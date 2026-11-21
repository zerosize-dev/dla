#!/usr/bin/env python3
"""Timeline of a headless transcript: tool calls (dla commands shown), results trimmed."""
import json,sys
p=sys.argv[1]; n=int(sys.argv[2]) if len(sys.argv)>2 else 0
rows=[]
for line in open(p):
    try: e=json.loads(line)
    except Exception: continue
    t=e.get('type')
    if t=='system' and e.get('subtype')=='init':
        rows.append('INIT model=%s plugins=%s skills=%d agents=%s'%(e.get('model'),[x.get('name') for x in e.get('plugins',[])],len(e.get('skills',[])),e.get('agents')))
    elif t=='assistant':
        par='  ' if e.get('parent_tool_use_id') else ''
        for c in e.get('message',{}).get('content',[]):
            if c.get('type')=='text' and c['text'].strip(): rows.append(par+'SAY '+c['text'].strip().replace('\n',' ')[:300])
            elif c.get('type')=='tool_use':
                i=c.get('input',{})
                d=i.get('command') or i.get('file_path') or i.get('pattern') or i.get('skill') or i.get('description') or json.dumps(i)[:200]
                rows.append(par+'%s: %s'%(c['name'],str(d).replace('\n',' ; ')[:330]))
    elif t=='user':
        par='  ' if e.get('parent_tool_use_id') else ''
        for c in e.get('message',{}).get('content',[]) if isinstance(e.get('message',{}).get('content'),list) else []:
            if c.get('type')=='tool_result':
                x=c.get('content'); 
                if isinstance(x,list): x=' '.join(y.get('text','') for y in x if isinstance(y,dict))
                rows.append(par+'   -> '+str(x).replace('\n',' ')[:260])
    elif t=='result':
        rows.append('RESULT turns=%s cost=%s is_error=%s :: %s'%(e.get('num_turns'),e.get('total_cost_usd'),e.get('is_error'),str(e.get('result'))[:1500]))
for r in rows[-n:] if n else rows: print(r)
