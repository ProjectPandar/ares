"""Read-only effective MMU VD capture; verified seeded ELF ABI, never raw Boost.

MultiMaterialSegmentation.cpp:1555-1556; Geometry/Voronoi.cpp:28-73.
GDB owns this module. Native application data/arguments are never modified.
"""
import gdb,os,json,struct,hashlib,traceback
from pathlib import Path
OUT=Path(os.environ['ARES_VD_DIAG_DIR']);OUT.mkdir(parents=True,exist_ok=False)
NATIVE=os.environ['ARES_VD_NATIVE_ELF']
DEBUG=os.environ['ARES_VD_DEBUG_DIR']
CPU=os.environ['ORCA_REPLAY_CPU']
KIND=os.environ.get('ARES_VD_CONSTRUCTOR','mmu')
SEARCH=os.environ.get('ARES_VD_SEARCH')=='1'
LIMIT=int(os.environ.get('ARES_VD_CAPTURE_LIMIT','100000'))
assert KIND in ('mmu','lines') and LIMIT>0
records=[];errors=[];active=set();exits=[];entry_count=0;owners={};limit_reached=False

def sample():
 p=gdb.selected_inferior().pid;rows=[]
 for f in Path(f'/proc/{p}/task').glob('*/status'):
  s=f.read_text();mask=next(l.split(':',1)[1].strip() for l in s.splitlines() if l.startswith('Cpus_allowed_list:'));rows.append({'tid':int(f.parent.name),'mask':mask})
 assert rows and all(r['mask']==CPU for r in rows)
 return rows

def mem(v,n):return bytes(gdb.selected_inferior().read_memory(int(v),n))
def point(p):
 assert p.type.sizeof==16
 return list(struct.unpack('<qq',mem(p.address,16)))
def vector(v,element):
 impl=v['_M_impl'];dtype=gdb.lookup_type('boost::polygon::voronoi_'+element+'<double>');assert dtype.sizeof=={'cell':24,'edge':48,'vertex':32}[element];a=impl['_M_start'].cast(dtype.pointer());b=impl['_M_finish'].cast(dtype.pointer());n=int(b-a);assert 0<=n<100000
 return a,n,int(dtype.sizeof)
def idx(p,a,n,size):
 p=int(p)
 if p==0:return None
 delta=p-int(a);assert delta>=0 and delta%size==0 and delta//size<n,(p,int(a),n,size)
 return delta//size

def dump(this,folder):
 v=this.dereference();modified=bool(v['m_is_modified']);base=v if modified else v['m_voronoi_diagram']
 c=vector(base['m_cells' if modified else 'cells_'],'cell');e=vector(base['m_edges' if modified else 'edges_'],'edge');z=vector(base['m_vertices' if modified else 'vertices_'],'vertex')
 (folder/'layout.json').write_text(json.dumps({name:{'base':int(a),'count':n,'size':s} for name,(a,n,s) in [('cells',c),('edges',e),('vertices',z)]},indent=2))
 for name,(a,n,s) in [('cells',c),('edges',e),('vertices',z)]:
  (folder/(name+'.bin')).write_bytes(mem(a,n*s) if n else b'')
 cells=[];vertices=[];edges=[]
 for i in range(c[1]):
  x=c[0][i];col=int(x['color_']);cells.append({'source':int(x['source_index_']),'category':col&31,'color':col>>5,'incident':idx(x['incident_edge_'],*e)})
 for i in range(z[1]):
  x=z[0][i];vertices.append({'x_bits':mem(x['x_'].address,8).hex(),'y_bits':mem(x['y_'].address,8).hex(),'incident':idx(x['incident_edge_'],*e),'color':int(x['color_'])>>5})
 for i in range(e[1]):
  x=e[0][i];col=int(x['color_']);edges.append({'cell':idx(x['cell_'],*c),'v0':idx(x['vertex_'],*z),'twin':idx(x['twin_'],*e),'next':idx(x['next_'],*e),'prev':idx(x['prev_'],*e),'linear':bool(col&1),'primary':bool(col&2),'color':col>>5})
 for i,x in enumerate(edges):
  assert x['cell'] is not None and x['twin'] is not None and x['next'] is not None and x['prev'] is not None
  assert edges[x['twin']]['twin']==i and edges[x['next']]['prev']==i and edges[x['prev']]['next']==i
  x['v1']=edges[x['twin']]['v0'];x['finite']=x['v0'] is not None and x['v1'] is not None
 for x in cells:
  assert x['category'] in [0,1,2,8,9]
 for i,x in enumerate(vertices):assert x['incident'] is not None and edges[x['incident']]['v0']==i
 return {'modified':modified,'state':int(v['m_state']),'issue':int(v['m_issue_type']),'cells':cells,'vertices':vertices,'edges':edges}

class Finished(gdb.FinishBreakpoint):
 def __init__(self,this,info,folder):
  super().__init__(gdb.newest_frame(),internal=True);self.this=this;self.info=info;self.folder=folder;active.add(self)
 def stop(self):
  try:
   row={**self.info,'diagram':dump(self.this,self.folder),'finish_affinity':sample()};assert all(c['source']<len(row['lines']) for c in row['diagram']['cells']);(self.folder/'record.json').write_text(json.dumps(row,indent=2));records.append(row['call_id']);del owners[int(self.this)];active.remove(self);return False
  except Exception:
   errors.append(traceback.format_exc());return True
 def out_of_scope(self):errors.append('FinishBreakpoint out_of_scope call='+str(self.info['call_id']))

class Enter(gdb.Breakpoint):
 def stop(self):
  global entry_count,limit_reached
  try:
   if entry_count==LIMIT:
    limit_reached=True;self.enabled=False;return False
   entry_count+=1;folder=OUT/f'call-{entry_count:04d}';folder.mkdir();t=gdb.lookup_type('Slic3r::Geometry::VoronoiDiagram').pointer();this=gdb.parse_and_eval('$rdi').cast(t);lt=gdb.lookup_type('Slic3r::ColoredLine' if KIND=='mmu' else 'Slic3r::Line');a=gdb.parse_and_eval('$rsi').cast(lt.pointer());b=gdb.parse_and_eval('$rdx').cast(lt.pointer());n=int(b-a);assert 0<n<100000;assert lt.sizeof==(48 if KIND=='mmu' else 32) and t.target().sizeof==176
   lines=[]
   for i in range(n):
    x=a[i];line=x['line'] if KIND=='mmu' else x;row={'a':point(line['a']),'b':point(line['b'])}
    if KIND=='mmu':row.update(color=int(x['color']),poly=int(x['poly_idx']),local_line=int(x['local_line_idx']))
    lines.append(row)
   raw=mem(a,n*lt.sizeof);(folder/('colored-lines.bin' if KIND=='mmu' else 'input-lines.bin')).write_bytes(raw);repair=bool(int(gdb.parse_and_eval('$rcx'))&255);assert repair
   info={'call_id':entry_count,'pid':gdb.selected_inferior().pid,'thread':list(gdb.selected_thread().ptid),'constructor':KIND,'caller_stack':gdb.execute('bt 24',to_string=True),'repair_enabled':repair,'lines':lines,'events':[],'entry_affinity':sample()}
   (folder/'entry.json').write_text(json.dumps(info,indent=2))
   if KIND=='lines':assert 'Slic3r::Geometry::MedialAxis::build' in info['caller_stack'],info['caller_stack']
   owners[int(this)]=info;Finished(this,info,folder);return False
  except Exception:
   errors.append(traceback.format_exc());return True

class EventFinished(gdb.FinishBreakpoint):
 def __init__(self,event):
  super().__init__(gdb.newest_frame(),internal=True);self.event=event;active.add(self)
 def stop(self):
  try:self.event['returned_issue']=int(gdb.parse_and_eval('$eax'));active.remove(self);return False
  except Exception:errors.append(traceback.format_exc());return True
 def out_of_scope(self):errors.append('detector/rotation FinishBreakpoint out_of_scope')

class EventEnter(gdb.Breakpoint):
 def __init__(self,address,kind):super().__init__('*'+hex(address),internal=True);self.kind=kind
 def stop(self):
  try:
   owner=owners.get(int(gdb.parse_and_eval('$rdi')))
   if owner is not None:
    event={'kind':self.kind}
    if self.kind=='rotation':event['angle_bits']=struct.pack('<d',float(gdb.parse_and_eval('$xmm0.v2_double[0]'))).hex()
    owner['events'].append(event);EventFinished(event)
   return False
  except Exception:errors.append(traceback.format_exc());return True

def on_exit(event):exits.append(getattr(event,'exit_code',None))
gdb.events.exited.connect(on_exit)
try:
 gdb.execute('set pagination off');gdb.execute('set confirm off');gdb.execute('set debuginfod enabled off');gdb.execute('set print thread-events off');gdb.execute('set debug-file-directory '+DEBUG);gdb.execute('set disable-randomization on')
 gdb.execute('file '+json.dumps(NATIVE));gdb.execute('starti');pid=gdb.selected_inferior().pid;actual=os.readlink(f'/proc/{pid}/exe');assert actual==NATIVE,actual;assert hashlib.sha256(Path(actual).read_bytes()).hexdigest()==os.environ['ARES_VD_NATIVE_SHA']
 maps=Path(f'/proc/{pid}/maps').read_text();(OUT/'maps.txt').write_text(maps);base=int(next(l.split('-')[0] for l in maps.splitlines() if l.endswith(NATIVE) and l.split()[2]=='00000000'),16);relative=0x9a7590 if KIND=='mmu' else 0x9a63f0;entry=base+relative
 (OUT/'observer.json').write_text(json.dumps({'actual_ELF':actual,'pid':pid,'base':base,'entry':entry,'entry_relative':relative,'constructor':KIND,'ELF_SHA256':hashlib.sha256(Path(actual).read_bytes()).hexdigest(),'debug_directory':DEBUG,'CPU':[int(CPU)],'vd_size':gdb.lookup_type('Slic3r::Geometry::VoronoiDiagram').sizeof,'coloredline_size':gdb.lookup_type('Slic3r::ColoredLine').sizeof,'initial_affinity':sample(),'mode':'read-only types/arguments/effective VD memory; no inferior calls/writes'},indent=2))
 Enter('*'+hex(entry),internal=True)
 EventEnter(base+(0x9a8560 if KIND=='mmu' else 0x9a73c0),'detect_original')
 EventEnter(base+0x9d0fe0,'detect_rotated')
 EventEnter(base+(0x9d14b0 if KIND=='mmu' else 0x9d06f0),'rotation')
 gdb.execute('continue');assert not errors and not active and not owners and exits==[0],(errors,len(active),exits);assert entry_count==len(records) and (entry_count>0 or SEARCH)
 (OUT/'outcome.json').write_text(json.dumps({'status':'COMPLETE_NATIVE_VD_CAPTURE','actual_ELF':actual,'entries':entry_count,'finishes':len(records),'capture_limit':LIMIT,'limit_reached':limit_reached,'exit_codes':exits,'errors':errors,'note':'Complete effective diagrams at actual calls; bounded search is not exhaustive absence.'},indent=2));gdb.execute('quit 0')
except Exception:
 errors.append(traceback.format_exc());(OUT/'OBSERVER-NON-GATE-FAILURE.json').write_text(json.dumps({'errors':errors,'entries':entry_count,'finishes':len(records),'exits':exits,'active':len(active),'not_backend_RED':True},indent=2));gdb.execute('quit 2')
