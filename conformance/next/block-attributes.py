"""Independent JSON attribute fixtures for the unpublished active contract."""
import copy,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
OUT=Path(__file__).parent/'ifccad'
v=json.loads((ROOT/'examples/ifccad/text.ifcx').read_text(encoding='utf-8'))
definition=next(n for n in v['data'] if 'ifccad::text' in n.get('attributes',{}))
a=definition['attributes'];text=a.pop('ifccad::text');a['ifccad::attributeDefinition']={'tag':'CODE','prompt':'Code','text':text,'flags':{'verify':True},'lockPosition':True}
defpath=definition['path'];ins='/cad/d1/e9007199254740992';child='/cad/d1/e9007199254740993'
for n in v['data']:
 attrs=n.get('attributes',{})
 if 'ifccad::drawing' in attrs:
  attrs['ifccad::drawing'].update(nextEntityId=9007199254740994,nextBlockId=3);n['children']['block2']='/cad/d1/block/2'
 if 'ifccad::layout' in attrs:
  for k,p in n.get('children',{}).items():
   if p==defpath:n['children'][k]=ins
 for role in ['ifccad::layout','ifccad::blockDefinition']:
  if role in attrs:attrs[role].pop('bounds',None);attrs[role].pop('boundsQuality',None)
v['data'] += [
 {'path':'/cad/d1/block/2','children':{'0':defpath},'attributes':{'ifccad::blockDefinition':{'name':'Attribute case','basePoint':[7,9,0],'insertionUnit':'mm'}}},
 {'path':ins,'children':{'0':child},'attributes':{'ifccad::entity':copy.deepcopy(a['ifccad::entity']),'ifccad::blockInstance':{'definition':'/cad/d1/block/2','transform':{'placement':{'origin':[100,200,0],'xAxis':[1,0,0],'yAxis':[0,1,0]},'scale':[-2,3,1]}}}},
 {'path':child,'attributes':{'ifccad::entity':copy.deepcopy(a['ifccad::entity']),'ifccad::geom::placement':{'origin':[101,202,0],'xAxis':[1,0,0],'yAxis':[0,1,0]},'ifccad::blockAttribute':{'tag':'RENAMED','definition':defpath,'text':copy.deepcopy(text),'secondaryAttributes':[defpath]}}}]
cases=[]
def emit(name,value,valid):
 file=('valid/' if valid else 'invalid/')+'attributes-'+name+'.ifcx';p=OUT/file;p.parent.mkdir(exist_ok=True);p.write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8');cases.append({'file':file,'valid':valid})
emit('ownership-and-strong-link',v,True)
m=copy.deepcopy(v);payload=m['data'][-1]['attributes']['ifccad::blockAttribute'];payload['mText']={'layer':a['ifccad::entity']['layer'],'placement':m['data'][-1]['attributes']['ifccad::geom::placement'],'text':{'style':text['style'],'height':2.5,'content':[{'inlines':[]}]}}
emit('empty-active-mtext',m,True)
f=copy.deepcopy(v)
def record(i,evaluator,code):return {'id':i,'evaluator':evaluator,'code':code,'format':'','cache':{'value':{'kind':'Text','value':'cached'},'display':'stale'},'evaluationOptions':0,'filingOptions':0,'state':0,'evaluationStatus':0,'errorCode':0,'errorMessage':''}
f['data'][-1]['attributes']['ifccad::blockAttribute']['fields']={'root':0,'nextId':2,'records':[record(0,'TextTemplate',[{'kind':'Literal','value':'File: '},{'kind':'Child','value':1}]),record(1,'DrawingVariable',[{'kind':'Literal','value':'\\AcVar Filename'}])]}
emit('field-graph',f,True)
for name,key,value in [('wrong-definition','definition',ins),('wrong-secondary-target','secondaryAttributes',[ins]),('null-field','fields',None),('unknown-field','invented',True)]:
 bad=copy.deepcopy(v);bad['data'][-1]['attributes']['ifccad::blockAttribute'][key]=value;emit(name,bad,False)
bad=copy.deepcopy(f);bad['data'][-1]['attributes']['ifccad::blockAttribute']['fields']['records'][1]['code']=[{'kind':'Child','value':0}];emit('field-cycle',bad,False)
p=OUT/'cases.json';old=json.loads(p.read_text(encoding='utf-8'));old=[c for c in old if not c['file'].startswith(('valid/attributes-','invalid/attributes-'))];p.write_text(json.dumps(old+cases,indent=2)+'\n',encoding='utf-8')
