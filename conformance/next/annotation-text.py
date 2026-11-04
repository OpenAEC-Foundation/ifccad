"""Independent annotation contract literals. No native writer is invoked."""
from pathlib import Path
from copy import deepcopy
import json, struct
ROOT=Path(__file__).resolve().parent
# Reuse only the independent minimal CBOR helpers, not the Dimension generator.
helpers=(ROOT/'dimension-kinds.py').read_text(encoding='utf-8').split('# Explicit code/body literals')[0]
exec(compile(helpers,str(ROOT/'dimension-kinds.py'),'exec'))
binary=ROOT/'ifccad-cbor';js=ROOT/'ifccad'
manifest=json.loads((binary/'cases.json').read_text());cases=json.loads((js/'cases.json').read_text())
base=decode((binary/'valid/empty.ifccad.cbor').read_bytes());d=base[2]
d[2][0]=2;d[2][6]=1;d[2][11]=4
# Two independently named scales with equal ratio; third unused extreme finite scale.
d[20]=[{0:1,1:'1:50',2:1.,3:50.,4:True},{0:2,1:'2:100',2:2.,3:100.},{0:3,1:'Extreme unused',2:5e-324,3:1.7976931348623157e308}]
d[12]=[{0:0,1:'Label',2:{0:{0:'Requested'}},3:{0:False,1:1}}]
d[7]=[{0:'Unused saved definition',1:False,2:True,3:False,5:1,6:'Unused',7:[0.,0.,0.],8:'mm',10:[],11:{0:False,1:1}}];d[2][3]=2
text={0:0,1:{0:[0.,0.,7.],1:[1.,0.,0.],2:[0.,1.,0.]},5:{0:0,1:{0:0,1:0,2:2.,3:1.}},8:[{0:'Saved',1:False,2:False,3:False}],9:{0:0,1:{0:1},2:[{0:{0:1},1:{0:[100.,-0.],1:[105.,3.],2:-0.,3:5}},{0:{0:2}}]}}
d[5][5]=[{0:0,1:{0:1,1:0,2:10,3:{0:text}}}]
def emit(name,doc,valid=True,error=None):
 file=f"{'valid' if valid else 'invalid'}/{name}.ifccad.cbor";(binary/file).write_bytes(encode(doc));(binary/(file+'.diag')).write_text(repr(doc)+'\n',encoding='utf-8')
 c=dict(file=file,valid=valid,diagnostic=file+'.diag');
 if error:c['errorClass']=error
 manifest['cases']=[x for x in manifest['cases'] if x['file']!=file]+[c]
emit('annotation-text',base)
for name,change,error in [
 ('annotation-stale-watermark',lambda x:x[2][2].__setitem__(11,3),'InvalidDocument'),
 ('annotation-unresolved-scale',lambda x:x[2][5][5][0][1][3][0][9][2][0][0].__setitem__(0,99),'InvalidDocument'),
 ('annotation-text-inheritance',lambda x:x[2][5][5][0][1][3][0][9].__setitem__(0,2),'InvalidDocument'),
 ('annotation-null-presentation',lambda x:x[2][5][5][0][1][3][0][9][2][0].__setitem__(1,None),'Malformed'),
]:
 bad=deepcopy(base);change(bad);emit(name,bad,False,error)
(binary/'cases.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
source=json.loads((js/'valid/text-rich.ifcx').read_text(encoding='utf-8'))
for n in source['data']:
 a=n.get('attributes',{})
 for policy_owner in ['ifccad::textStyle','ifccad::blockDefinition']:
  if policy_owner in a:a[policy_owner]['annotation']={'enabled':False,'orientation':'matchLayout'}
 for k in ['ifccad::layout','ifccad::blockDefinition']:
  if k in a:
   for f in ['bounds','boundsQuality']:a[k].pop(f,None)
 if 'ifccad::drawing' in a:a['ifccad::drawing']['nextAnnotationScaleId']=3;n['children']['annotationScale1']='/cad/d1/annotationScale/1';n['children']['annotationScale2']='/cad/d1/annotationScale/2'
 if 'ifccad::text' in a:a['ifccad::annotation']={'mode':'disabled','reference':{'scaleId':'/cad/d1/annotationScale/1'},'contexts':[{'key':{'scaleId':'/cad/d1/annotationScale/1'},'presentation':{'insertion':[101.,-0.],'alignment':[105.,3.],'rotation':-0.,'horizontal':'fit'}},{'key':{'scaleId':'/cad/d1/annotationScale/2'}}]}
 if 'ifccad::mText' in a:a['ifccad::annotation']={'mode':'enabled','contexts':[{'key':{'scaleId':'/cad/d1/annotationScale/1'},'presentation':{'insertion':[2.,3.,19.],'xDirection':[0.,2.,1.],'attachment':'bottomRight','rectangleWidth':0.,'rectangleHeight':17.,'columns':{'kind':'static','count':2,'width':0.,'gutter':0.,'autoHeight':True,'flowReversed':True,'heights':[1.,2.,3.]}}}]}
source['data'] += [{'path':f'/cad/d1/annotationScale/{i}','attributes':{'ifccad::annotationScale':{'name':name,'paperUnits':paper,'drawingUnits':drawing}}} for i,name,paper,drawing in [(1,'1:50',1.,50.),(2,'2:100',2.,100.)]]
file='valid/annotation-text.ifcx';(js/file).write_text(json.dumps(source,indent=2,ensure_ascii=False)+'\n',encoding='utf-8');cases=[x for x in cases if x['file']!=file]+[dict(file=file,valid=True)]
(js/'cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
