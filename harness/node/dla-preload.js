'use strict';

// DLA measurement agent for Node.js processes.
//
// Loaded with NODE_OPTIONS='--require <abs path>/dla-preload.js' by `dla harness run`. It counts
// live timer and event-listener registrations by the first stack frame inside DLA_PROJECT, exposes
// the top-level bindings of probed modules, and writes one JSON record per dump to the file named by
// DLA_HARNESS_OUT: an `exit` record from the process 'exit' event, `checkpoint` records when a
// driver calls checkpoint(), and light `tick` records every DLA_TICK_MS.
//
// Environment: DLA_HARNESS_OUT, DLA_HARNESS_PROBES (probes.json), DLA_PURGE (gc|none),
// DLA_PROJECT (scratch root), DLA_TICK_MS, DLA_ITER. The module never throws into the target.

const fs = require('fs');
const path = require('path');
const v8 = require('v8');
const vm = require('vm');
const util = require('util');
const Module = require('module');
const events = require('events');
const timers = require('timers');

const T0 = process.hrtime.bigint();
const MAX_OBJECTS = 2000000;
const PROBE_DEADLINE_MS = 60000;
const BREAKDOWN_CAP = 64;
const MODULE_NAME_CAP = 150;
const STACK_LIMIT = 64;
const PRUNE_FLOOR = 1024;
const SELF = __filename;

function env(name) {
  const value = process.env[name];
  return value ? value : null;
}

function isMainThread() {
  try {
    return require('worker_threads').isMainThread;
  } catch (_) {
    return true;
  }
}

// npm/yarn/pnpm/corepack inherit NODE_OPTIONS from the harness run; they are never measured.
function isToolProcess() {
  const script = process.argv[1] || '';
  if (/(^|[\\/])(npm|npx|yarn|pnpm|corepack)(-cli)?(\.c?js)?$/.test(script)) return true;
  return /[\\/](lib[\\/]node_modules[\\/]npm|node_modules[\\/]corepack|corepack)[\\/]/.test(script);
}

const ACTIVE = env('DLA_HARNESS_OUT') !== null && isMainThread() && !isToolProcess();

function realRoot() {
  const root = env('DLA_PROJECT');
  if (root === null) return null;
  try {
    return fs.realpathSync(root);
  } catch (_) {
    return path.resolve(root);
  }
}

const ROOT = realRoot();
const ROOT_PREFIX = ROOT === null ? null : ROOT + path.sep;
const FILE_PREFIX = ROOT === null ? null : 'file://' + ROOT + '/';

// --------------------------------------------------------------------------------------------
// call-site attribution
// --------------------------------------------------------------------------------------------

/** Return "relative/file.js:line" of the first stack frame inside the project, or null. */
function projectSite() {
  if (ROOT === null) return null;
  const holder = {};
  const saved = Error.stackTraceLimit;
  let stack;
  try {
    Error.stackTraceLimit = STACK_LIMIT;
    Error.captureStackTrace(holder, projectSite);
    stack = holder.stack;
  } catch (_) {
    return null;
  } finally {
    Error.stackTraceLimit = saved;
  }
  if (typeof stack !== 'string') return null;
  const lines = stack.split('\n');
  for (let i = 1; i < lines.length; i += 1) {
    const line = lines[i];
    let at = line.indexOf(ROOT_PREFIX);
    let skip = ROOT_PREFIX.length;
    if (at < 0) {
      at = line.indexOf(FILE_PREFIX);
      skip = FILE_PREFIX.length;
    }
    if (at < 0) continue;
    const rest = line.slice(at + skip);
    if (rest.startsWith('node_modules/') || rest.includes('/node_modules/')) continue;
    if (line.includes(SELF)) continue;
    const m = /^(.*?):(\d+):\d+\)?\s*$/.exec(rest);
    if (m) return `${m[1].split(path.sep).join('/')}:${m[2]}`;
  }
  return null;
}

// --------------------------------------------------------------------------------------------
// timers: live registrations (an unref()'d timer still counts)
// --------------------------------------------------------------------------------------------

// Records hang off the object they describe in a hidden symbol property (no WeakMap: the backing
// table of a WeakMap keeps its capacity after its entries die, which would look like growth).
const TIMER_REC = Symbol('dla.timer');
const LISTENER_REC = Symbol('dla.listeners');
const fallbackRecords = new WeakMap(); // for frozen/sealed objects only

function attach(obj, key, value) {
  try {
    Object.defineProperty(obj, key, { value, configurable: true, enumerable: false, writable: true });
  } catch (_) {
    fallbackRecords.set(obj, value);
  }
}

function attached(obj, key) {
  if (obj === null || (typeof obj !== 'object' && typeof obj !== 'function')) return undefined;
  const value = obj[key];
  return value !== undefined ? value : fallbackRecords.get(obj);
}

const timerSites = new Map(); // site -> live count (project frames only)
let timersLive = 0;

function bump(map, key, delta) {
  const next = (map.get(key) || 0) + delta;
  if (next > 0) map.set(key, next);
  else map.delete(key);
}

function timerSet(rec) {
  if (rec.live) return;
  rec.live = true;
  timersLive += 1;
  if (rec.site !== null) bump(timerSites, rec.site, 1);
}

function timerEnd(rec) {
  if (!rec.live) return;
  rec.live = false;
  timersLive -= 1;
  if (rec.site !== null) bump(timerSites, rec.site, -1);
}

function installTimers() {
  const origSetInterval = timers.setInterval;
  const origSetTimeout = timers.setTimeout;
  const origClearInterval = timers.clearInterval;
  const origClearTimeout = timers.clearTimeout;

  function wrapSet(orig, repeat) {
    const wrapped = function (callback, ...rest) {
      if (typeof callback !== 'function') return orig.call(this, callback, ...rest);
      const rec = { site: null, live: false, repeat };
      let cb = callback;
      if (!repeat) {
        cb = function (...args) {
          timerEnd(rec); // a timeout that fires is no longer registered
          return callback.apply(this, args);
        };
      }
      const handle = orig.call(this, cb, ...rest);
      try {
        rec.site = projectSite();
        attach(handle, TIMER_REC, rec);
        timerSet(rec);
      } catch (_) {
        // counting must never break the target
      }
      return handle;
    };
    // keep util.promisify(setTimeout) and friends working
    for (const key of Reflect.ownKeys(orig)) {
      if (key === 'length' || key === 'name' || key === 'prototype') continue;
      try {
        Object.defineProperty(wrapped, key, Object.getOwnPropertyDescriptor(orig, key));
      } catch (_) {
        // ignore
      }
    }
    return wrapped;
  }

  function wrapClear(orig) {
    return function (handle) {
      try {
        const rec = attached(handle, TIMER_REC);
        if (rec) timerEnd(rec);
      } catch (_) {
        // ignore
      }
      return orig.apply(this, arguments);
    };
  }

  const setIntervalW = wrapSet(origSetInterval, true);
  const setTimeoutW = wrapSet(origSetTimeout, false);
  const clearIntervalW = wrapClear(origClearInterval);
  const clearTimeoutW = wrapClear(origClearTimeout);

  // Timeout.prototype.close / refresh / Symbol.dispose
  try {
    const probe = origSetTimeout(() => {}, 0);
    const proto = Object.getPrototypeOf(probe);
    origClearTimeout(probe);
    if (proto && typeof proto.close === 'function') {
      const close = proto.close;
      proto.close = function () {
        try {
          const rec = attached(this, TIMER_REC);
          if (rec) timerEnd(rec);
        } catch (_) {
          // ignore
        }
        return close.apply(this, arguments);
      };
    }
    if (proto && typeof proto.refresh === 'function') {
      const refresh = proto.refresh;
      proto.refresh = function () {
        const result = refresh.apply(this, arguments);
        try {
          const rec = attached(this, TIMER_REC);
          if (rec) timerSet(rec);
        } catch (_) {
          // ignore
        }
        return result;
      };
    }
    if (proto && typeof proto[Symbol.dispose] === 'function') {
      const dispose = proto[Symbol.dispose];
      proto[Symbol.dispose] = function () {
        try {
          const rec = attached(this, TIMER_REC);
          if (rec) timerEnd(rec);
        } catch (_) {
          // ignore
        }
        return dispose.apply(this, arguments);
      };
    }
  } catch (_) {
    // ignore
  }

  timers.setInterval = setIntervalW;
  timers.setTimeout = setTimeoutW;
  timers.clearInterval = clearIntervalW;
  timers.clearTimeout = clearTimeoutW;
  for (const [name, fn] of [
    ['setInterval', setIntervalW],
    ['setTimeout', setTimeoutW],
    ['clearInterval', clearIntervalW],
    ['clearTimeout', clearTimeoutW],
  ]) {
    try {
      globalThis[name] = fn;
    } catch (_) {
      // ignore
    }
  }
  return { origSetInterval, origClearTimeout };
}

// --------------------------------------------------------------------------------------------
// listeners: EventEmitter and EventTarget
// --------------------------------------------------------------------------------------------

// A record only lives as long as its emitter, so dead emitters drop out of the counts.
const listenerRefs = new Set(); // WeakRef(emitter), swept now and then
let nextSweepAt = PRUNE_FLOOR;

function recordFor(target) {
  let rec = attached(target, LISTENER_REC);
  if (rec === undefined) {
    rec = new Map(); // type -> [{fn, ref, site, capture}]
    attach(target, LISTENER_REC, rec);
    listenerRefs.add(new WeakRef(target));
    if (listenerRefs.size >= nextSweepAt) sweepListenerRefs();
  }
  return rec;
}

/** Drop the weak references of emitters that were collected; returns how many went. */
function sweepListenerRefs() {
  const before = listenerRefs.size;
  for (const ref of listenerRefs) if (ref.deref() === undefined) listenerRefs.delete(ref);
  nextSweepAt = Math.max(PRUNE_FLOOR, listenerRefs.size * 2);
  return before - listenerRefs.size;
}

function typeKey(type) {
  return typeof type === 'symbol' ? type : String(type);
}

function noteAdd(target, type, fn, prepend) {
  try {
    if (typeof fn !== 'function' && !(typeof fn === 'object' && fn !== null)) return;
    const rec = recordFor(target);
    const key = typeKey(type);
    let list = rec.get(key);
    if (list === undefined) {
      list = [];
      rec.set(key, list);
    }
    const entry = { fn, site: projectSite(), capture: false };
    if (prepend) list.unshift(entry);
    else list.push(entry);
  } catch (_) {
    // ignore
  }
}

function noteRemove(target, type, fn) {
  try {
    const rec = attached(target, LISTENER_REC);
    if (rec === undefined) return;
    const list = rec.get(typeKey(type));
    if (list === undefined) return;
    for (let i = list.length - 1; i >= 0; i -= 1) {
      const held = list[i].fn;
      if (held === fn || (fn !== null && fn !== undefined && fn.listener === held)) {
        list.splice(i, 1);
        break;
      }
    }
    if (list.length === 0) rec.delete(typeKey(type));
  } catch (_) {
    // ignore
  }
}

function listenerTotal(target, type) {
  try {
    if (typeof target.listenerCount === 'function') return target.listenerCount(type);
  } catch (_) {
    // ignore
  }
  return -1;
}

const ONCE_WRAPPER = /onceWrapper/;

function installEmitter() {
  const proto = events.EventEmitter.prototype;
  const add = (name, prepend) => {
    const orig = proto[name];
    if (typeof orig !== 'function') return;
    proto[name] = function (type, listener) {
      const result = orig.apply(this, arguments);
      // once()/prependOnceListener() call on()/prependListener() with an internal wrapper that
      // carries .listener; the outer call records the user's function, so skip the inner one
      if (!(typeof listener === 'function' && typeof listener.listener === 'function' && ONCE_WRAPPER.test(listener.name))) {
        noteAdd(this, type, listener, prepend);
      }
      return result;
    };
  };
  const addOnce = (name, prepend) => {
    const orig = proto[name];
    if (typeof orig !== 'function') return;
    proto[name] = function (type, listener) {
      const result = orig.apply(this, arguments);
      noteAdd(this, type, listener, prepend);
      return result;
    };
  };
  add('on', false);
  add('addListener', false);
  add('prependListener', true);
  addOnce('once', false);
  addOnce('prependOnceListener', true);

  const remove = (name) => {
    const orig = proto[name];
    if (typeof orig !== 'function') return;
    proto[name] = function (type, listener) {
      const before = listenerTotal(this, type);
      const result = orig.apply(this, arguments);
      const after = listenerTotal(this, type);
      if (before < 0 || after < before) noteRemove(this, type, listener);
      return result;
    };
  };
  remove('removeListener');
  remove('off');

  const origAll = proto.removeAllListeners;
  if (typeof origAll === 'function') {
    proto.removeAllListeners = function (type) {
      const result = origAll.apply(this, arguments);
      try {
        const rec = attached(this, LISTENER_REC);
        if (rec !== undefined) {
          if (arguments.length === 0 || type === undefined) rec.clear();
          else rec.delete(typeKey(type));
        }
      } catch (_) {
        // ignore
      }
      return result;
    };
  }
}

function installEventTarget() {
  if (typeof EventTarget === 'undefined') return;
  const proto = EventTarget.prototype;
  const origAdd = proto.addEventListener;
  const origRemove = proto.removeEventListener;
  if (typeof origAdd === 'function') {
    proto.addEventListener = function (type, listener, options) {
      const result = origAdd.apply(this, arguments);
      try {
        if (listener !== null && listener !== undefined && !(options && options.signal && options.signal.aborted)) {
          const capture = typeof options === 'boolean' ? options : Boolean(options && options.capture);
          const rec = recordFor(this);
          const key = typeKey(type);
          let list = rec.get(key);
          if (list === undefined) {
            list = [];
            rec.set(key, list);
          }
          // the same (type, listener, capture) is registered once
          if (!list.some((e) => e.fn === listener && e.capture === capture)) {
            const auto = typeof options === 'object' && options !== null && (options.once || options.signal);
            // listeners that remove themselves are held weakly so the record never pins them
            const entry = auto
              ? { fn: undefined, ref: new WeakRef(listener), site: projectSite(), capture }
              : { fn: listener, site: projectSite(), capture };
            list.push(entry);
          }
        }
      } catch (_) {
        // ignore
      }
      return result;
    };
  }
  if (typeof origRemove === 'function') {
    proto.removeEventListener = function (type, listener, options) {
      const result = origRemove.apply(this, arguments);
      try {
        const capture = typeof options === 'boolean' ? options : Boolean(options && options.capture);
        const rec = attached(this, LISTENER_REC);
        const list = rec === undefined ? undefined : rec.get(typeKey(type));
        if (list !== undefined) {
          const at = list.findIndex((e) => (e.fn !== undefined ? e.fn : e.ref.deref()) === listener && e.capture === capture);
          if (at >= 0) list.splice(at, 1);
          if (list.length === 0) rec.delete(typeKey(type));
        }
      } catch (_) {
        // ignore
      }
      return result;
    };
  }
}

/** Count live listeners (total and by project site); call after a forced collection. */
function countListeners() {
  const sites = new Map();
  let total = 0;
  for (const ref of [...listenerRefs]) {
    const target = ref.deref();
    if (target === undefined) {
      listenerRefs.delete(ref);
      continue;
    }
    const rec = attached(target, LISTENER_REC);
    if (rec === undefined) continue;
    for (const [type, list] of rec) {
      let current = null;
      try {
        current = events.getEventListeners(target, type);
      } catch (_) {
        current = null;
      }
      // reconcile with what the target really holds (once listeners that fired, aborted signals)
      const remaining = current === null ? null : new Map();
      if (remaining !== null) for (const fn of current) remaining.set(fn, (remaining.get(fn) || 0) + 1);
      const keep = [];
      for (const entry of list) {
        const fn = entry.fn !== undefined ? entry.fn : entry.ref.deref();
        if (fn === undefined) continue;
        if (remaining !== null) {
          const left = remaining.get(fn) || 0;
          if (left <= 0) continue;
          remaining.set(fn, left - 1);
        }
        keep.push(entry);
        total += 1;
        if (entry.site !== null) bump(sites, entry.site, 1);
      }
      if (keep.length !== list.length) {
        list.length = 0;
        for (const entry of keep) list.push(entry);
      }
    }
  }
  return { total, sites };
}

// --------------------------------------------------------------------------------------------
// probes: the module wrapper accessor and the measurement of a binding
// --------------------------------------------------------------------------------------------

const IDENT = /^[A-Za-z_$][A-Za-z0-9_$]*$/;
const probeFiles = new Map(); // abs file -> {names: Set, scan: bool}
let probeSpecs = [];
let probeLimitations = [];

function normaliseKind(raw) {
  return String(raw || '').replace(/[_-]/g, '').toLowerCase();
}

function loadProbeSpecs() {
  const file = env('DLA_HARNESS_PROBES');
  if (file === null || ROOT === null) return;
  let specs;
  try {
    specs = JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch (err) {
    probeLimitations.push(`probes.json unreadable: ${err && err.message}`);
    return;
  }
  if (specs && !Array.isArray(specs)) specs = specs.probes;
  if (!Array.isArray(specs)) return;
  for (const spec of specs) {
    try {
      const target = spec.target && typeof spec.target === 'object' ? spec.target : spec;
      const kind = normaliseKind(target.kind);
      if (kind !== 'nodemodule' && kind !== 'nodebinding') continue;
      if (typeof target.file !== 'string') continue;
      const abs = path.resolve(ROOT, target.file);
      let entry = probeFiles.get(abs);
      if (entry === undefined) {
        entry = { names: new Set(), scan: false };
        probeFiles.set(abs, entry);
      }
      const names = kind === 'nodebinding' ? [target.name] : Array.isArray(target.names) ? target.names : [];
      for (const name of names) if (typeof name === 'string' && IDENT.test(name)) entry.names.add(name);
      if (kind === 'nodemodule' && names.length === 0) entry.scan = true;
      probeSpecs.push({ id: spec.id === undefined ? null : spec.id, kind, file: target.file, abs, names, name: target.name });
    } catch (_) {
      // a malformed spec is skipped
    }
  }
}

// Fallback when dla passed no names: top-level (column 0) declarations of the module source.
function scanTopLevel(source) {
  const names = new Set();
  const re = /^(?:export\s+)?(?:const|let|var|class|function\*?|async\s+function\*?)\s+([A-Za-z_$][A-Za-z0-9_$]*)/gm;
  let m;
  while ((m = re.exec(source)) !== null && names.size < MODULE_NAME_CAP) names.add(m[1]);
  return names;
}

function installProbes() {
  if (probeFiles.size === 0) return;
  const origCompile = Module.prototype._compile;
  if (typeof origCompile !== 'function') return;
  Module.prototype._compile = function (content, filename, ...rest) {
    let source = content;
    try {
      const entry = probeFiles.get(filename);
      if (entry !== undefined && typeof content === 'string') {
        const names = new Set(entry.names);
        if (entry.scan) for (const name of scanTopLevel(content)) names.add(name);
        if (names.size > 0) {
          const cases = [...names].map((n) => `case ${JSON.stringify(n)}: return ${n};`).join(' ');
          source =
            content +
            `\n;(function () { try { Object.defineProperty(module, '__dlaProbe', { configurable: true, enumerable: false, ` +
            `value: function (n) { switch (n) { ${cases} } } }); } catch (e) {} })();\n`;
        }
      }
    } catch (_) {
      source = content;
    }
    return origCompile.call(this, source, filename, ...rest);
  };
}

// ---- bounded reachable walk -------------------------------------------------------------------

const OPAQUE_CONSTRUCTORS = new Set([
  'Timeout', 'Immediate', 'TimersList', 'TCP', 'TTY', 'Pipe', 'Socket', 'Server', 'Agent', 'IncomingMessage',
  'ServerResponse', 'ClientRequest', 'ChildProcess', 'Worker', 'MessagePort', 'Console', 'Module', 'AsyncResource',
  'AbortSignal', 'Performance',
]);

function constructorName(obj) {
  try {
    const proto = Object.getPrototypeOf(obj);
    const ctor = proto && Object.getOwnPropertyDescriptor(proto, 'constructor');
    return ctor && typeof ctor.value === 'function' ? ctor.value.name : '';
  } catch (_) {
    return '';
  }
}

function isLeaf(obj) {
  return (
    util.types.isTypedArray(obj) ||
    util.types.isArrayBuffer(obj) ||
    util.types.isSharedArrayBuffer(obj) ||
    util.types.isDataView(obj) ||
    util.types.isWeakMap(obj) ||
    util.types.isWeakSet(obj) ||
    util.types.isPromise(obj) ||
    util.types.isDate(obj) ||
    util.types.isRegExp(obj) ||
    util.types.isNativeError(obj) ||
    util.types.isProxy(obj) ||
    obj instanceof WeakRef
  );
}

function isOpaque(obj) {
  if (obj === globalThis || obj === process || obj instanceof Module) return true;
  return OPAQUE_CONSTRUCTORS.has(constructorName(obj));
}

function pushObject(stack, value) {
  if ((typeof value === 'object' && value !== null) || typeof value === 'function') stack.push(value);
}

/** Objects reachable from `root` (the root included); iterative, bounded by MAX_OBJECTS. */
function walk(root, deadline) {
  const seen = new WeakSet();
  const stack = [];
  pushObject(stack, root);
  let count = 0;
  let steps = 0;
  while (stack.length > 0) {
    const obj = stack.pop();
    if (seen.has(obj)) continue;
    seen.add(obj);
    steps += 1;
    if ((steps & 0xfff) === 0 && Date.now() > deadline) return { count, truncated: 'time' };
    if (count >= MAX_OBJECTS) return { count, truncated: 'objects' };
    count += 1;
    if (typeof obj === 'function') continue; // closure variables cannot be inspected
    if (isLeaf(obj) || isOpaque(obj)) continue;
    try {
      if (Array.isArray(obj)) {
        for (let i = 0; i < obj.length; i += 1) pushObject(stack, obj[i]);
      } else if (util.types.isMap(obj)) {
        Map.prototype.forEach.call(obj, (value, key) => {
          pushObject(stack, key);
          pushObject(stack, value);
        });
      } else if (util.types.isSet(obj)) {
        Set.prototype.forEach.call(obj, (value) => pushObject(stack, value));
      }
      if (!Array.isArray(obj)) {
        for (const key of Reflect.ownKeys(obj)) {
          const desc = Object.getOwnPropertyDescriptor(obj, key);
          if (desc && 'value' in desc) pushObject(stack, desc.value);
        }
      }
    } catch (_) {
      // an exotic object is counted but not followed
    }
  }
  return { count, truncated: null };
}

function sizeOf(value) {
  try {
    if (util.types.isMap(value) || util.types.isSet(value)) return value.size;
    if (Array.isArray(value)) return value.length;
  } catch (_) {
    // ignore
  }
  return null;
}

function probeRecord(id, name, pathName, value, deadline) {
  const rec = { id, name, path: pathName, size: null, reachable: null, bytes: null, error: null };
  if ((typeof value !== 'object' || value === null) && typeof value !== 'function') {
    rec.reachable = 0;
    if (typeof value === 'number' && Number.isFinite(value)) rec.size = value; // a counter variable
    return rec;
  }
  rec.size = sizeOf(value);
  const result = walk(value, deadline);
  rec.reachable = result.count;
  if (result.truncated === 'objects') rec.error = `walk truncated at ${MAX_OBJECTS} objects`;
  else if (result.truncated === 'time') rec.error = 'walk stopped at the time budget';
  return rec;
}

const FUNCTION_BUILTINS = new Set(['length', 'name', 'prototype', 'caller', 'arguments']);

/** One-level breakdown: own data properties of an object (or static fields of a class). */
function breakdown(id, name, pathName, value, deadline) {
  const out = [];
  const isFn = typeof value === 'function';
  if (!isFn && (typeof value !== 'object' || value === null)) return out;
  if (!isFn && (Array.isArray(value) || util.types.isMap(value) || util.types.isSet(value) || isLeaf(value))) return out;
  if (!isFn && isOpaque(value)) return out;
  let keys;
  try {
    keys = Object.getOwnPropertyNames(value);
  } catch (_) {
    return out;
  }
  for (const key of keys) {
    if (out.length >= BREAKDOWN_CAP) break;
    if (isFn && FUNCTION_BUILTINS.has(key)) continue;
    const desc = Object.getOwnPropertyDescriptor(value, key);
    if (!desc || !('value' in desc)) continue;
    const child = desc.value;
    if (typeof child !== 'object' || child === null) continue; // methods and primitives
    if (Date.now() > deadline) {
      out.push({ id, name: `${name}.${key}`, path: `${pathName}.${key}`, size: null, reachable: null, bytes: null, error: 'probe time budget exhausted' });
      break;
    }
    out.push(probeRecord(id, `${name}.${key}`, `${pathName}.${key}`, child, deadline));
  }
  return out;
}

function reportBinding(out, spec, name, value, deadline, explicit) {
  const pathName = `${spec.file}:${name}`;
  if (!explicit) {
    // a module probe reports data holders, not functions or plain primitives
    if (typeof value === 'function') {
      const statics = breakdown(spec.id, name, pathName, value, deadline);
      out.push(...statics);
      return;
    }
    if (typeof value !== 'object' || value === null) return;
  }
  out.push(probeRecord(spec.id, name, pathName, value, deadline));
  out.push(...breakdown(spec.id, name, pathName, value, deadline));
}

function runProbes(limitations) {
  const out = [];
  const deadline = Date.now() + PROBE_DEADLINE_MS;
  for (const spec of probeSpecs) {
    try {
      const mod = Module._cache[spec.abs];
      if (mod === undefined) {
        out.push({ id: spec.id, name: spec.name || spec.file, path: spec.file, size: null, reachable: null, bytes: null, error: 'module not loaded (or loaded as an ES module)' });
        continue;
      }
      if (typeof mod.__dlaProbe !== 'function') {
        out.push({ id: spec.id, name: spec.name || spec.file, path: spec.file, size: null, reachable: null, bytes: null, error: 'module has no probe accessor (loaded before the agent, or as an ES module)' });
        continue;
      }
      const names = spec.kind === 'nodebinding' ? [spec.name] : [...probeFiles.get(spec.abs).names];
      for (const name of names) {
        if (typeof name !== 'string' || !IDENT.test(name)) continue;
        if (out.length >= MODULE_NAME_CAP * 4) break;
        let value;
        try {
          value = mod.__dlaProbe(name);
        } catch (err) {
          if (spec.kind === 'nodebinding') {
            out.push({ id: spec.id, name, path: `${spec.file}:${name}`, size: null, reachable: null, bytes: null, error: String(err && err.message) });
          }
          continue;
        }
        if (value === undefined && spec.kind === 'nodebinding') {
          out.push({ id: spec.id, name, path: `${spec.file}:${name}`, size: null, reachable: null, bytes: null, error: 'binding not found at module top level' });
          continue;
        }
        reportBinding(out, spec, name, value, deadline, spec.kind === 'nodebinding');
      }
    } catch (err) {
      out.push({ id: spec.id, name: spec.name || null, path: spec.file, size: null, reachable: null, bytes: null, error: String(err && err.message) });
    }
  }
  if (Date.now() > deadline) limitations.push(`probe walk stopped after ${PROBE_DEADLINE_MS / 1000} s`);
  return out;
}

// --------------------------------------------------------------------------------------------
// reading values
// --------------------------------------------------------------------------------------------

let gcFn = typeof globalThis.gc === 'function' ? globalThis.gc : null;
if (gcFn === null) {
  try {
    v8.setFlagsFromString('--expose-gc');
    gcFn = vm.runInNewContext('gc');
  } catch (_) {
    gcFn = null;
  }
}

function osValues() {
  const out = {};
  const keys = {
    VmRSS: 'os.rss',
    RssAnon: 'os.rss_anon',
    RssFile: 'os.rss_file',
    RssShmem: 'os.rss_shmem',
    VmSwap: 'os.swap',
    VmData: 'os.vm_data',
    VmHWM: 'os.vm_hwm',
  };
  try {
    for (const line of fs.readFileSync('/proc/self/status', 'latin1').split('\n')) {
      const at = line.indexOf(':');
      if (at < 0) continue;
      const key = line.slice(0, at);
      const num = Number.parseInt(line.slice(at + 1).trim(), 10);
      if (key in keys) out[keys[key]] = num * 1024;
      else if (key === 'Threads') out['os.threads'] = num;
    }
  } catch (_) {
    // not Linux or /proc unreadable: memoryUsage().rss below
  }
  return out;
}

function memValues() {
  const out = {};
  try {
    const mem = process.memoryUsage();
    out['node.heap_used'] = mem.heapUsed;
    out['node.heap_total'] = mem.heapTotal;
    out['node.external'] = mem.external;
    out['node.array_buffers'] = mem.arrayBuffers;
    out['os.rss'] = mem.rss;
  } catch (_) {
    // ignore
  }
  return out;
}

function orderedGroup(map) {
  const out = {};
  for (const [site, count] of [...map].sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1))) out[site] = count;
  return out;
}

function resourceCounts(values) {
  try {
    if (typeof process.getActiveResourcesInfo !== 'function') return;
    for (const type of process.getActiveResourcesInfo()) {
      const key = `node.active.${type}`;
      values[key] = (values[key] || 0) + 1;
    }
  } catch (_) {
    // ignore
  }
}

function heapStats(values) {
  try {
    const stats = v8.getHeapStatistics();
    values['node.native_contexts'] = stats.number_of_native_contexts;
    values['node.detached_contexts'] = stats.number_of_detached_contexts;
    values['node.malloced_memory'] = stats.malloced_memory;
  } catch (_) {
    // ignore
  }
}

function purgeRecord(kind, preGc, limitations) {
  const snapshot = () => {
    const mem = memValues();
    return { 'os.rss': mem['os.rss'], 'node.heap_used': mem['node.heap_used'] };
  };
  if (kind === 'gc') {
    const after = snapshot();
    return { kind, before: preGc, after };
  }
  limitations.push(`purge '${kind}' is not supported by the node agent (only gc)`);
  return { kind };
}

let checkpointWritten = false;
const detectors = [];

function build(phase, x, last) {
  const limitations = [];
  const tick = phase === 'tick';
  const rec = {
    dla_harness: 1,
    phase,
    pid: process.pid,
    argv: [process.argv[0], ...process.execArgv, ...process.argv.slice(1)],
    exe: process.execPath,
    runtime: 'node',
    x,
    t_ms: Number((process.hrtime.bigint() - T0) / 1000000n),
    gc_forced: false,
  };
  let preGc = null;
  if (!tick) {
    const mem = memValues();
    preGc = {
      'node.heap_used': mem['node.heap_used'],
      'node.heap_total': mem['node.heap_total'],
      'node.external': mem['node.external'],
      'node.array_buffers': mem['node.array_buffers'],
      'os.rss': mem['os.rss'],
    };
    if (gcFn !== null) {
      try {
        gcFn();
        gcFn();
        // the weak references of dead emitters are garbage the agent itself leaves behind
        if (sweepListenerRefs() > 0) gcFn();
        rec.gc_forced = true;
      } catch (_) {
        limitations.push('forced gc failed: values are not after a collection');
      }
    } else {
      limitations.push('gc() unavailable: values are not after a collection');
    }
  }
  const values = { ...osValues(), ...memValues() };
  heapStats(values);
  resourceCounts(values);
  const listeners = countListeners();
  values['node.timers'] = timersLive;
  values['node.listeners'] = listeners.total;
  rec.values = values;
  if (preGc !== null) rec.values_pre_gc = preGc;
  rec.groups = {
    'node.timers': orderedGroup(timerSites),
    'node.listeners': orderedGroup(listeners.sites),
  };
  const purge = env('DLA_PURGE');
  if (last && !tick && purge !== null && purge !== 'none') rec.purge = purgeRecord(purge, preGc, limitations);
  if (!tick) {
    rec.probes = runProbes(limitations);
    if (detectors.length > 0) rec.detectors = detectors.slice();
    if (env('NODE_V8_COVERAGE') !== null) limitations.push('function coverage is written by node itself to NODE_V8_COVERAGE');
  }
  for (const note of probeLimitations) limitations.push(note);
  limitations.push('timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted');
  rec.limitations = limitations;
  return rec;
}

function writeLine(out, rec) {
  const line = JSON.stringify(rec, (_, value) => (typeof value === 'bigint' ? Number(value) : value)) + '\n';
  fs.mkdirSync(path.dirname(out), { recursive: true });
  const fd = fs.openSync(out, 'a', 0o644);
  try {
    fs.writeSync(fd, line);
  } finally {
    fs.closeSync(fd);
  }
}

/** Write one record to DLA_HARNESS_OUT synchronously. Never throws. */
function dump(phase = 'exit', x = null, last = true) {
  const out = env('DLA_HARNESS_OUT');
  if (out === null || !ACTIVE) return;
  let rec;
  try {
    rec = build(phase, x, last);
  } catch (err) {
    rec = { dla_harness: 1, phase, pid: process.pid, runtime: 'node', x, limitations: [`dump failed: ${err && err.message}`] };
  }
  try {
    writeLine(out, rec);
  } catch (_) {
    // nowhere to report it
  }
}

/** Driver entry: write a checkpoint record; x is the number of units completed. */
function checkpoint(x, last = false) {
  checkpointWritten = true;
  dump('checkpoint', x, last);
}

/** Record a library detector hit (for example a MaxListenersExceededWarning) in the next dump. */
function detector(lib, text, site = null) {
  detectors.push({ lib, text: String(text).slice(0, 400), site });
}

function iterFromEnv() {
  const raw = env('DLA_ITER');
  if (raw === null) return null;
  const n = Number.parseInt(raw, 10);
  return Number.isFinite(n) ? n : null;
}

function init() {
  if (!ACTIVE) return;
  loadProbeSpecs();
  const { origSetInterval } = installTimers();
  installEmitter();
  installEventTarget();
  installProbes();
  process.on('warning', (warning) => {
    try {
      if (warning && warning.name === 'MaxListenersExceededWarning') detector('node', `${warning.name}: ${warning.message}`);
    } catch (_) {
      // ignore
    }
  });
  process.on('exit', () => {
    try {
      if (!checkpointWritten) dump('exit', iterFromEnv(), true);
    } catch (_) {
      // never throw into the target
    }
  });
  const tickMs = Number.parseInt(env('DLA_TICK_MS') || '', 10);
  if (Number.isFinite(tickMs) && tickMs > 0) {
    const timer = origSetInterval(() => dump('tick', null, false), tickMs);
    if (timer && typeof timer.unref === 'function') timer.unref();
  }
}

try {
  init();
} catch (_) {
  // the target runs unmeasured rather than broken
}

module.exports = { checkpoint, dump, detector, active: ACTIVE };
