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

// The purge record's readings are the same measurement each column's series holds at the last
// point: `before` is that point's own (post-GC) `values`, `after` is taken with the same keys
// after one more forced collection. (A pre-GC reading as `before` would make every column with
// per-step garbage look released.) The `*.pre_gc` columns carry no purge pair.
const PURGE_KEYS = ['node.heap_used', 'node.heap_total', 'node.external', 'node.array_buffers', 'os.rss'];

function purgeRecord(kind, values, limitations) {
  if (kind === 'gc') {
    const before = {};
    for (const k of PURGE_KEYS) if (values[k] !== undefined) before[k] = values[k];
    if (gcFn === null) {
      limitations.push('gc() unavailable: the purge changed nothing');
    } else {
      try {
        gcFn();
      } catch (_) {
        limitations.push('forced gc failed: the purge changed nothing');
      }
    }
    const mem = memValues();
    const after = {};
    for (const k of PURGE_KEYS) if (mem[k] !== undefined) after[k] = mem[k];
    return { kind, before, after };
  }
  limitations.push(`purge '${kind}' is not supported by the node agent (only gc)`);
  return { kind };
}

// --------------------------------------------------------------------------------------------
// raw captures (DLA_CAPTURE): heap snapshots, sampling heap profiles, resource creation sites
// --------------------------------------------------------------------------------------------

// Files go to DLA_CAPTURE_DIR/<pid>/ and are listed in the "captures" array of the record that
// triggered them. DLA_CAPTURE is a comma list of heap, alloc and nonheap (histogram and jfr are
// JVM captures). Nothing here throws into the target: a failed capture becomes an entry with
// "error".
//
//   heap     heap-<n>.heapsnapshot  v8.writeHeapSnapshot at the checkpoints chosen by dumpHere()
//                                   (driver) or at exit (command); scripts.json maps script ids
//                                   to urls (Debugger.scriptParsed seen by this agent)
//   alloc    alloc-<n>.heapprofile  inspector sampling heap profile of the objects still alive
//   nonheap  resources.json         active resources by type and creation site, at the end
const OS = require('os');

const CAPTURE_KINDS = new Set();
for (const part of (env('DLA_CAPTURE') || '').split(',')) {
  const kind = part.trim().toLowerCase();
  if (kind) CAPTURE_KINDS.add(kind);
}
const CAPTURE_DIR = env('DLA_CAPTURE_DIR');
const CAPTURE_ON = ACTIVE && CAPTURE_DIR !== null && CAPTURE_KINDS.size > 0;
const HEAP_FACTOR = 12; // a snapshot needs about 12 times heapUsed of extra memory (docs/VERIFY.md)
const SNAPSHOT_BYTES_CAP = 3 * 1024 * 1024 * 1024;
const PROFILE_BYTES_CAP = 64 * 1024 * 1024;
const SCRIPTS_CAP = 5000;
const SITES_PER_TYPE = 40;
const TYPES_CAP = 40;
const TRACKED_CAP = 500000;
const SAMPLING_INTERVAL = (() => {
  const n = Number.parseInt(env('DLA_HEAPPROF_INTERVAL') || '', 10);
  return Number.isFinite(n) && n >= 256 ? n : 8192;
})();

const captureNotes = []; // limitations found while starting the captures
let capturesTaken = 0; // checkpoints seen
let heapSeq = 0;
let allocSeq = 0;
let snapshotBytes = 0;
let finalCaptured = false;
let session = null;
let sessionFailed = false;
let sampling = false;
const scriptUrls = new Map(); // script id -> {url, map}

function intEnv(name, fallback) {
  const n = Number.parseInt(env(name) || '', 10);
  return Number.isFinite(n) && n >= 0 ? n : fallback;
}

/** Whether the checkpoint with this 1-based number gets a heavy capture (same rule as the JVM). */
function dumpHere(seq, warmup, rounds, dumps) {
  const total = warmup + rounds;
  if (total <= 0 || total <= dumps) return true;
  const first = Math.max(warmup, 1);
  if (seq === first || seq === total) return true;
  const interior = dumps - 2;
  if (interior <= 0 || seq < first || seq > total) return false;
  for (let j = 1; j <= interior; j += 1) {
    if (seq === first + Math.round(((total - first) * j) / (interior + 1))) return true;
  }
  return false;
}

function inspectorSession() {
  if (session !== null || sessionFailed) return session;
  try {
    const inspector = require('inspector');
    session = new inspector.Session();
    session.connect();
  } catch (err) {
    session = null;
    sessionFailed = true;
    captureNotes.push(`inspector session unavailable: ${err && err.message}`);
  }
  return session;
}

/** Post one inspector command; the callback of an in-process session runs synchronously. */
function post(method, params) {
  const s = inspectorSession();
  if (s === null) throw new Error('inspector session unavailable');
  let error = null;
  let result;
  let called = false;
  s.post(method, params || {}, (err, res) => {
    called = true;
    error = err;
    result = res;
  });
  if (!called) throw new Error(`${method} did not answer synchronously`);
  if (error) throw new Error(`${method}: ${error.message || error}`);
  return result;
}

function startScriptWatch() {
  try {
    const s = inspectorSession();
    if (s === null) return;
    s.on('Debugger.scriptParsed', (message) => {
      const p = message && message.params;
      if (!p || !p.url || p.url.startsWith('node:') || p.url.startsWith('internal/')) return;
      if (scriptUrls.size >= SCRIPTS_CAP * 2) return;
      let map = p.sourceMapURL || '';
      if (map.startsWith('data:') || map.length > 512) map = map ? 'inline' : '';
      scriptUrls.set(String(p.scriptId), { url: p.url, map });
    });
    post('Debugger.enable');
    captureNotes.push('--capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead');
  } catch (err) {
    captureNotes.push(`script ids could not be recorded: ${err && err.message}`);
  }
}

function startSamplingNow() {
  post('HeapProfiler.startSampling', {
    samplingInterval: SAMPLING_INTERVAL,
    includeObjectsCollectedByMajorGC: false,
    includeObjectsCollectedByMinorGC: false,
  });
  sampling = true;
}

// Resource creation sites. async_hooks reports every resource; the stack is read only for the
// types that can leak (handles and watchers), not for promises, ticks or request objects.
const RESOURCE_SKIP = new Set([
  'PROMISE', 'Microtask', 'TickObject', 'Immediate', 'Timeout', 'TIMERWRAP',
  'FSREQCALLBACK', 'FSREQPROMISE', 'FSREQWRAP', 'WRITEWRAP', 'SHUTDOWNWRAP', 'CONNECTWRAP',
  'TCPCONNECTWRAP', 'PIPECONNECTWRAP', 'GETADDRINFOREQWRAP', 'GETNAMEINFOREQWRAP',
  'QUERYWRAP', 'HTTPINCOMINGMESSAGE', 'HTTPCLIENTREQUEST', 'RANDOMBYTESREQUEST',
  'ZLIB', 'PERFORMANCEOBSERVER', 'SIGINTWATCHDOG',
]);
// async_hooks type names to the names process.getActiveResourcesInfo() uses for the same handles.
const RESOURCE_ALIAS = {
  TCPWRAP: 'TCPSocketWrap',
  TCPSERVERWRAP: 'TCPServerWrap',
  PIPEWRAP: 'PipeWrap',
  TTYWRAP: 'TTYWrap',
  UDPWRAP: 'UDPWrap',
  FSEVENTWRAP: 'FSEvent',
  PROCESSWRAP: 'ProcessWrap',
  SIGNALWRAP: 'SignalWrap',
};
const tracked = new Map(); // async id -> {type, site}
const resourceCreated = new Map(); // type -> count
let trackedOverflow = 0;
let trackerOn = false;

function startResourceTracker() {
  try {
    const hook = require('async_hooks').createHook({
      init(id, rawType, trigger) {
        if (RESOURCE_SKIP.has(rawType)) return;
        const type = RESOURCE_ALIAS[rawType] || rawType;
        resourceCreated.set(type, (resourceCreated.get(type) || 0) + 1);
        if (tracked.size >= TRACKED_CAP) {
          trackedOverflow += 1;
          return;
        }
        // A socket a server accepted has no project frame of its own: it takes its server's site.
        const parent = tracked.get(trigger);
        tracked.set(id, { type, site: projectSite() || (parent ? parent.site : null) });
      },
      destroy(id) {
        tracked.delete(id);
      },
    });
    hook.enable();
    trackerOn = true;
    captureNotes.push('--capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture');
  } catch (err) {
    captureNotes.push(`resource tracker unavailable: ${err && err.message}`);
  }
}

function captureDir() {
  const dir = path.join(CAPTURE_DIR, String(process.pid));
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

function startCaptures() {
  if (!CAPTURE_ON) return;
  for (const kind of CAPTURE_KINDS) {
    if (kind === 'histogram' || kind === 'jfr') {
      captureNotes.push(`--capture ${kind} is a jvm capture; for node use heap (type-histogram-diff reads the snapshots)`);
    } else if (kind !== 'heap' && kind !== 'alloc' && kind !== 'nonheap') {
      captureNotes.push(`unknown capture kind ${kind}`);
    }
  }
  if (CAPTURE_KINDS.has('heap')) startScriptWatch();
  if (CAPTURE_KINDS.has('alloc')) {
    try {
      post('HeapProfiler.enable');
      startSamplingNow();
    } catch (err) {
      captureNotes.push(`sampling heap profiler could not be started: ${err && err.message}`);
    }
  }
  if (CAPTURE_KINDS.has('nonheap')) startResourceTracker();
}

/**
 * Reclaimable memory from `vm_stat` output: free + inactive + speculative pages. On macOS
 * os.freemem() counts only the free pages, a small part of what the system can give back.
 * Returns null when the text has no page size or no free count.
 */
function parseVmStat(text) {
  const size = /page size of (\d+) bytes/.exec(text);
  if (size === null) return null;
  let pages = 0;
  let seen = false;
  for (const line of text.split('\n')) {
    const m = /^Pages (free|inactive|speculative):\s+(\d+)\./.exec(line);
    if (m === null) continue;
    pages += Number.parseInt(m[2], 10);
    if (m[1] === 'free') seen = true;
  }
  return seen ? pages * Number.parseInt(size[1], 10) : null;
}

/** Available memory on macOS (or from the vm_stat text in DLA_VM_STAT_FILE, for tests); null elsewhere or on failure. */
function macAvailable() {
  try {
    const file = env('DLA_VM_STAT_FILE');
    if (file !== null) return parseVmStat(fs.readFileSync(file, 'utf8'));
    if (process.platform !== 'darwin') return null;
    const text = require('child_process').execFileSync('vm_stat', [], { encoding: 'utf8', timeout: 2000, stdio: ['ignore', 'pipe', 'ignore'] });
    return parseVmStat(text);
  } catch (_) {
    return null;
  }
}

/** Memory the process may still use: the smaller of MemAvailable, the free memory and the cgroup headroom. */
function availableMemory() {
  const forced = Number.parseInt(env('DLA_CAPTURE_MEM_BYTES') || '', 10); // override, for tests and tight hosts
  if (Number.isFinite(forced) && forced >= 0) return forced;
  let avail = OS.freemem();
  try {
    for (const line of fs.readFileSync('/proc/meminfo', 'latin1').split('\n')) {
      if (line.startsWith('MemAvailable:')) avail = Number.parseInt(line.slice(13), 10) * 1024;
    }
  } catch (_) {
    // not Linux
  }
  const mac = macAvailable();
  if (mac !== null) avail = mac;
  const cgroup = [
    ['/sys/fs/cgroup/memory.max', '/sys/fs/cgroup/memory.current'],
    ['/sys/fs/cgroup/memory/memory.limit_in_bytes', '/sys/fs/cgroup/memory/memory.usage_in_bytes'],
  ];
  for (const [limitFile, usedFile] of cgroup) {
    try {
      const limit = Number(fs.readFileSync(limitFile, 'latin1').trim());
      const used = Number(fs.readFileSync(usedFile, 'latin1').trim());
      if (Number.isFinite(limit) && Number.isFinite(used) && limit > 0 && limit < 2 ** 60) {
        avail = Math.min(avail, Math.max(limit - used, 0));
        break;
      }
    } catch (_) {
      // next layout
    }
  }
  return avail;
}

function entry(kind, role, file, ms, error) {
  const e = { kind, role };
  if (file !== null) e.file = path.basename(file);
  if (error !== null) {
    e.error = error;
  } else {
    let bytes = 0;
    try {
      bytes = fs.statSync(file).size;
    } catch (_) {
      // reported as 0
    }
    e.bytes = bytes;
  }
  e.ms = ms;
  return e;
}

const nowMs = () => Number(process.hrtime.bigint() / 1000000n);

function writeScripts(dir) {
  const project = [];
  const other = [];
  for (const [id, s] of scriptUrls) {
    const inProject = ROOT !== null && (s.url.startsWith(FILE_PREFIX) || s.url.startsWith(ROOT_PREFIX));
    (inProject ? project : other).push([id, s]);
  }
  const keep = project.slice(0, SCRIPTS_CAP).concat(other.slice(0, Math.max(0, SCRIPTS_CAP - project.length)));
  const scripts = {};
  const maps = {};
  for (const [id, s] of keep) {
    scripts[id] = s.url;
    if (s.map) maps[id] = s.map;
  }
  const doc = { dla_scripts: 1, pid: process.pid, root: ROOT, scripts, maps, dropped: scriptUrls.size - keep.length };
  const file = path.join(dir, 'scripts.json');
  fs.writeFileSync(file, JSON.stringify(doc) + '\n');
  return file;
}

function captureHeap(out, dir, heapUsed, limitations) {
  const t0 = nowMs();
  const role = 'heap snapshot (v8)';
  try {
    const need = (heapUsed || 0) * HEAP_FACTOR;
    const avail = availableMemory();
    if (need > avail) {
      throw new Error(`preflight: a snapshot needs about ${Math.round(need / 1048576)} MB (${HEAP_FACTOR}x heapUsed) and ${Math.round(avail / 1048576)} MB are available`);
    }
    if (snapshotBytes + (heapUsed || 0) > SNAPSHOT_BYTES_CAP) {
      throw new Error(`the snapshots of this process already take ${Math.round(snapshotBytes / 1048576)} MB (cap ${Math.round(SNAPSHOT_BYTES_CAP / 1048576)} MB)`);
    }
    heapSeq += 1;
    const file = path.join(dir, `heap-${heapSeq}.heapsnapshot`);
    fs.rmSync(file, { force: true });
    v8.writeHeapSnapshot(file);
    const e = entry('heap', role, file, nowMs() - t0, null);
    snapshotBytes += e.bytes;
    out.push(e);
    try {
      const sf = writeScripts(dir);
      out.push(entry('nonheap', 'script id to url map (sidecar of the heap snapshots)', sf, 0, null));
    } catch (err) {
      limitations.push(`scripts.json was not written: ${err && err.message}`);
    }
  } catch (err) {
    out.push(entry('heap', role, null, nowMs() - t0, String(err && err.message ? err.message : err)));
  }
}

function captureAlloc(out, dir, restart, limitations) {
  const t0 = nowMs();
  const role = 'sampling heap profile (live objects)';
  if (!sampling) {
    out.push(entry('alloc', role, null, 0, 'the sampling heap profiler is not running'));
    return;
  }
  try {
    if (gcFn !== null) {
      try {
        gcFn();
        gcFn();
      } catch (_) {
        limitations.push('gc before the heap profile failed: the profile may hold dead objects');
      }
    }
    const res = post('HeapProfiler.stopSampling');
    sampling = false;
    const text = JSON.stringify(res && res.profile);
    if (!text || text.length > PROFILE_BYTES_CAP) {
      throw new Error(text ? `the heap profile is over ${PROFILE_BYTES_CAP} bytes` : 'stopSampling returned no profile');
    }
    allocSeq += 1;
    const file = path.join(dir, `alloc-${allocSeq}.heapprofile`);
    fs.writeFileSync(file, text);
    out.push(entry('alloc', role, file, nowMs() - t0, null));
  } catch (err) {
    out.push(entry('alloc', role, null, nowMs() - t0, String(err && err.message ? err.message : err)));
  }
  if (restart) {
    try {
      startSamplingNow();
    } catch (err) {
      limitations.push(`the sampling heap profiler could not be restarted: ${err && err.message}`);
    }
  }
}

function captureResources(out, dir, x, limitations) {
  const t0 = nowMs();
  const role = 'active resources by type and creation site';
  try {
    const active = {};
    if (typeof process.getActiveResourcesInfo === 'function') {
      for (const type of process.getActiveResourcesInfo()) active[type] = (active[type] || 0) + 1;
    }
    const byType = new Map();
    const slot = (type) => {
      let s = byType.get(type);
      if (!s) {
        s = { active: active[type] || 0, tracked: 0, source: 'async_hooks', sites: new Map(), unattributed: 0 };
        byType.set(type, s);
      }
      return s;
    };
    for (const type of Object.keys(active)) slot(type);
    for (const { type, site } of tracked.values()) {
      const s = slot(type);
      s.tracked += 1;
      if (site === null) s.unattributed += 1;
      else s.sites.set(site, (s.sites.get(site) || 0) + 1);
    }
    // Timers are exact: the timer wrappers see every clear and every fire at once, while the
    // async_hooks destroy event of a timer reaches the tracker only after the current tick.
    const timerSlot = slot('Timeout');
    timerSlot.source = 'timers';
    timerSlot.tracked = timersLive;
    timerSlot.sites = new Map(timerSites);
    timerSlot.unattributed = Math.max(0, timersLive - [...timerSites.values()].reduce((a, b) => a + b, 0));
    const types = {};
    const ordered = [...byType].sort((a, b) => b[1].tracked + b[1].active - (a[1].tracked + a[1].active) || (a[0] < b[0] ? -1 : 1));
    for (const [type, s] of ordered.slice(0, TYPES_CAP)) {
      const sites = [...s.sites].sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1));
      const row = {
        active: s.active,
        tracked: s.tracked,
        created: resourceCreated.get(type) || null,
        source: s.source,
        sites: Object.fromEntries(sites.slice(0, SITES_PER_TYPE)),
        unattributed: s.unattributed,
      };
      if (sites.length > SITES_PER_TYPE) row.sites_dropped = sites.length - SITES_PER_TYPE;
      types[type] = row;
    }
    const notes = [
      'active counts the ref\'d handles node reports; tracked counts resources whose async_hooks destroy has not run, which can include resources closed in the current tick',
    ];
    if (!trackerOn) notes.push('the async_hooks tracker did not start: only the active counts and the timer sites are present');
    if (trackedOverflow > 0) notes.push(`${trackedOverflow} resources were not tracked after the first ${TRACKED_CAP}`);
    if (ordered.length > TYPES_CAP) notes.push(`${ordered.length - TYPES_CAP} resource types were left out`);
    const doc = {
      dla_resources: 1,
      pid: process.pid,
      runtime: 'node',
      x,
      t_ms: Number((process.hrtime.bigint() - T0) / 1000000n),
      active,
      types,
      limitations: notes,
    };
    const file = path.join(dir, 'resources.json');
    fs.writeFileSync(file, JSON.stringify(doc) + '\n');
    out.push(entry('nonheap', role, file, nowMs() - t0, null));
  } catch (err) {
    limitations.push(`resources.json was not written: ${err && err.message}`);
    out.push(entry('nonheap', role, null, nowMs() - t0, String(err && err.message ? err.message : err)));
  }
}

/**
 * Take the captures due at this record. phase is 'checkpoint' (driver) or 'exit' (command mode);
 * last marks the final round. Returns the entries for the record's "captures" array.
 */
function captureNow(phase, x, last, heapUsed, limitations) {
  const out = [];
  if (!CAPTURE_ON || finalCaptured) return out;
  const driver = phase === 'checkpoint';
  const fin = !driver || last;
  let seq = 0;
  if (driver) {
    capturesTaken += 1;
    seq = capturesTaken;
  }
  const here = !driver || dumpHere(seq, intEnv('DLA_WARMUP', 0), intEnv('DLA_ROUNDS', 0), intEnv('DLA_CAPTURE_DUMPS', 4));
  let dir;
  try {
    dir = captureDir();
  } catch (err) {
    limitations.push(`capture directory unusable: ${err && err.message}`);
    return out;
  }
  // The profile first: the snapshot's own work should not be in it.
  if (CAPTURE_KINDS.has('alloc') && (here || fin)) captureAlloc(out, dir, !fin, limitations);
  if (CAPTURE_KINDS.has('heap') && here) captureHeap(out, dir, heapUsed, limitations);
  if (fin) {
    finalCaptured = true;
    if (CAPTURE_KINDS.has('nonheap')) captureResources(out, dir, x, limitations);
  }
  return out;
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
  if (last && !tick && purge !== null && purge !== 'none') rec.purge = purgeRecord(purge, values, limitations);
  if (!tick) {
    rec.probes = runProbes(limitations);
    if (detectors.length > 0) rec.detectors = detectors.slice();
    if (env('NODE_V8_COVERAGE') !== null) limitations.push('function coverage is written by node itself to NODE_V8_COVERAGE');
  }
  for (const note of probeLimitations) limitations.push(note);
  for (const note of captureNotes) limitations.push(note);
  if (!tick && CAPTURE_ON) {
    const captures = captureNow(phase, x, last, values['node.heap_used'], limitations);
    if (captures.length > 0) rec.captures = captures;
  }
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
  startCaptures();
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
