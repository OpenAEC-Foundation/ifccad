"""Independent Block annotation literals; no production writer calls."""
from pathlib import Path
from copy import deepcopy
import json,struct
ROOT=Path(__file__).resolve().parent
exec(compile((ROOT/'dimension-kinds.py').read_text(encoding='utf-8').split('# Explicit code/body literals')[0],str(ROOT/'dimension-kinds.py'),'exec'))
binary=ROOT/'ifccad-cbor';js=ROOT/'ifccad'
base=decode((binary/'valid/hatch-editing.ifccad.cbor').read_bytes());d=base[2]
d[2][0]=3;d[2][3]=2;d[2][11]=2;d[20]=[{0:1,1:'1:50',2:1.,3:50.}]
frame={0:[0.,0.,0.],1:[1.,0.,0.],2:[0.,1.,0.]}
transform={0:deepcopy(frame)};p={0:deepcopy(frame),1:0.5,2:[-2.,3.,4.]};p[0][0]=[300.,400.,500.]
annotation={0:2,1:{0:1},2:[{0:{0:1},1:{0:p}}],3:0}
d[5].pop(0,None);d[5].pop(4,None);d[5][5]=[{0:0,1:{0:1,1:0,2:8,3:{0:1,1:transform,2:annotation}}}]
d[7]=[{0:'',1:False,2:True,3:False,5:1,6:'Marker',7:[2.,0.,0.],8:'cm',10:[{0:0,1:{0:2,1:0,2:1,3:{0:deepcopy(frame)}}}]}]
manifest=json.loads((binary/'cases.json').read_text(encoding='utf-8'))
def emit(name,document,valid=True,error=None):
 file=f"{'valid' if valid else 'invalid'}/{name}.ifccad.cbor";(binary/file).write_bytes(encode(document));(binary/(file+'.diag')).write_text(repr(document)+'\n',encoding='utf-8');c=dict(file=file,valid=valid,diagnostic=file+'.diag')
 if error:c['errorClass']=error
 manifest['cases']=[x for x in manifest['cases'] if x['file']!=file]+[c]
emit('annotation-blocks',base)
for name,change in [('annotation-block-zero-scale',lambda x:x[2][5][5][0][1][3][2][2][0][1][0][2].__setitem__(0,0.)),('annotation-block-uniform',lambda x:x[2][7][0].__setitem__(3,True))]:
 bad=deepcopy(base);change(bad);emit(name,bad,False,'InvalidDocument')
(binary/'cases.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
source=json.loads((ROOT.parent.parent/'tests/fixtures/ifccad/hello-nested-blocks.ifcx').read_text(encoding='utf-8'))
drawing=next(n for n in source['data'] if 'ifccad::drawing' in n.get('attributes',{}));prefix=drawing['path'];drawing['children']['annotationScale1']=prefix+'/annotationScale/1';drawing['attributes']['ifccad::drawing']['nextAnnotationScaleId']=2
source['data'].append({'path':prefix+'/annotationScale/1','attributes':{'ifccad::annotationScale':{'name':'1:50','paperUnits':1.,'drawingUnits':50.}}})
for n in source['data']:
 a=n.get('attributes',{})
 for owner in ['ifccad::layout','ifccad::blockDefinition']:
  if owner in a:a[owner].pop('bounds',None);a[owner].pop('boundsQuality',None)
 if 'ifccad::blockInstance' in a:
  a['ifccad::annotation']={'mode':'byDefinition','orientation':'fixed','reference':{'scaleId':prefix+'/annotationScale/1'},'contexts':[{'key':{'scaleId':prefix+'/annotationScale/1'},'presentation':{'transform':{'placement':{'origin':[300.,400.,500.],'xAxis':[1.,0.,0.],'yAxis':[0.,1.,0.]},'rotation':0.5,'scale':[-2.,3.,4.]}}}]}
file='valid/annotation-blocks.ifcx';(js/file).write_text(json.dumps(source,indent=2)+'\n',encoding='utf-8');cases=json.loads((js/'cases.json').read_text(encoding='utf-8'));cases=[c for c in cases if c['file']!=file]+[dict(file=file,valid=True)];(js/'cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
