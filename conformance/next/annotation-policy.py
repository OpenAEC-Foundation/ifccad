"""Independent saved annotation policy literals, using existing workspace geometry."""
from pathlib import Path
from copy import deepcopy
import json,struct
ROOT=Path(__file__).resolve().parent
exec(compile((ROOT/'dimension-kinds.py').read_text(encoding='utf-8').split('# Explicit code/body literals')[0],str(ROOT/'dimension-kinds.py'),'exec'))
binary=ROOT/'ifccad-cbor';js=ROOT/'ifccad'
base=decode((binary/'valid/workspace-state.ifccad.cbor').read_bytes());d=base[2];d[2][11]=3;d[20]=[{0:1,1:'1:20',2:1.,3:20.},{0:2,1:'1:50',2:1.,3:50.}];d[22]={0:False,1:1};d[5][6]={0:1,1:0};d[5].pop(0,None);d[5].pop(4,None)
viewport=None
for paper in d[8]:
 paper[8]={0:2,1:1};paper.pop(0,None);paper.pop(6,None)
 for entity in paper[7]:
  if entity.get(0)==0 and entity[1][2]==9:viewport=entity[1][3][0];viewport[10]=1
assert viewport is not None
d[21]=[{0:0,1:'Saved annotation camera',2:0,3:24.,4:deepcopy(viewport[3]),5:2}]
manifest=json.loads((binary/'cases.json').read_text(encoding='utf-8'))
def emit(name,document,valid=True,error=None):
 file=f"{'valid' if valid else 'invalid'}/{name}.ifccad.cbor";(binary/file).write_bytes(encode(document));(binary/(file+'.diag')).write_text(repr(document)+'\n',encoding='utf-8');c=dict(file=file,valid=valid,diagnostic=file+'.diag')
 if error:c['errorClass']=error
 manifest['cases']=[x for x in manifest['cases'] if x['file']!=file]+[c]
emit('annotation-policy',base)
for name,change,error in [('annotation-policy-empty',lambda x:x[2].__setitem__(22,{}),'InvalidDocument'),('annotation-policy-null-current',lambda x:x[2][5][6].__setitem__(0,None),'Malformed'),('annotation-policy-unresolved-scale',lambda x:x[2][21][0].__setitem__(5,100),'InvalidDocument'),('annotation-policy-model-window-choice',lambda x:x[2][15][0].__setitem__(9,1),'Malformed')]:
 bad=deepcopy(base);change(bad);emit(name,bad,False,error)
(binary/'cases.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
source=json.loads((js/'valid/workspace-state.ifcx').read_text(encoding='utf-8'));drawing=next(n for n in source['data'] if 'ifccad::drawing' in n.get('attributes',{}));prefix=drawing['path'];drawing['attributes']['ifccad::drawing']['annotationSettings']={'insertAsAnnotative':False,'modelLinetypeScaling':'byAnnotationScale'};drawing['attributes']['ifccad::drawing']['nextAnnotationScaleId']=3
for i,den in [(1,20.),(2,50.)]:drawing['children'][f'annotationScale{i}']=f'{prefix}/annotationScale/{i}';source['data'].append({'path':f'{prefix}/annotationScale/{i}','attributes':{'ifccad::annotationScale':{'name':f'1:{den:g}','paperUnits':1.,'drawingUnits':den}}})
view=None
for node in source['data']:
 a=node.get('attributes',{})
 if 'ifccad::layout' in a:
  v=a['ifccad::layout'];v['annotationState']={'currentScale':prefix+'/annotationScale/'+('1' if v['kind']=='Model' else '2'),'visibility':'currentScaleOnly' if v['kind']=='Model' else 'all'};v.pop('bounds',None);v.pop('boundsQuality',None)
 if 'ifccad::viewport' in a:view=a['ifccad::viewport']['view'];a['ifccad::viewport']['annotationScale']=prefix+'/annotationScale/1'
assert view is not None
drawing['children']['annotationView0']=prefix+'/annotationView/0';source['data'].append({'path':prefix+'/annotationView/0','attributes':{'ifccad::annotationView':{'name':'Saved annotation camera','space':'model','width':24.,'view':view,'annotationScale':prefix+'/annotationScale/2'}}})
file='valid/annotation-policy.ifcx';(js/file).write_text(json.dumps(source,indent=2)+'\n',encoding='utf-8');cases=json.loads((js/'cases.json').read_text(encoding='utf-8'));cases=[c for c in cases if c['file']!=file]+[dict(file=file,valid=True)];(js/'cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
