"""Independent Hatch scale/view annotation literals; never calls a production writer."""
from pathlib import Path
from copy import deepcopy
import json,struct
ROOT=Path(__file__).resolve().parent
exec(compile((ROOT/'dimension-kinds.py').read_text(encoding='utf-8').split('# Explicit code/body literals')[0],str(ROOT/'dimension-kinds.py'),'exec'))
binary=ROOT/'ifccad-cbor';js=ROOT/'ifccad'
base=decode((binary/'valid/hatch-editing.ifccad.cbor').read_bytes());base[2][2][11]=2;base[2][20]=[{0:1,1:'1:20',2:1.,3:20.}]
h=base[2][5][5][0][1][3][0]
def presentation(scale):return {0:{0:[],1:scale,2:[4.,5.,6.]},1:[{0:0,1:{0:0,1:{3:True,4:True,9:True,10:True}}},{0:1,1:{0:{},1:{0:{0:1,1:{0:[100.,0.],1:5.}}}}}]}
h[9]={0:0,1:{0:1},2:[{0:{0:1},1:presentation(2.)}]}
h[10]={0:5,1:2,2:[{0:i,1:1,3:presentation(scale),4:{0:[0.,0.,2.],1:0.25,2:False}} for i,scale in [(1,3.),(2,4.)]]}
h[10][2][1][3][1][0][1][1].update({11:True,12:True,13:True})
manifest=json.loads((binary/'cases.json').read_text(encoding='utf-8'))
def emit(name,document,valid=True,error=None):
 file=f"{'valid' if valid else 'invalid'}/{name}.ifccad.cbor";(binary/file).write_bytes(encode(document));(binary/(file+'.diag')).write_text(repr(document)+'\n',encoding='utf-8');c=dict(file=file,valid=valid,diagnostic=file+'.diag')
 if error:c['errorClass']=error
 manifest['cases']=[x for x in manifest['cases'] if x['file']!=file]+[c]
emit('annotation-hatch',base)
for name,change,error in [
 ('annotation-hatch-duplicate-view',lambda h:h[10][2][1].__setitem__(0,1),'InvalidDocument'),
 ('annotation-hatch-null-target',lambda h:h[10][2][0].__setitem__(2,None),'Malformed'),
 ('annotation-hatch-shared-index',lambda h:h[9][2][0][1][1][0][1].__setitem__(0,4294967295),'InvalidDocument'),
 ('annotation-hatch-local-polyline-flag',lambda h:h[9][2][0][1][1][1][1][0].__setitem__(1,True),'InvalidDocument'),
]:
 bad=deepcopy(base);change(bad[2][5][5][0][1][3][0]);emit(name,bad,False,error)
(binary/'cases.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
source=json.loads((ROOT.parent.parent/'tests/fixtures/ifccad/hello-hatch-solid.ifcx').read_text(encoding='utf-8'))
drawing=next(n for n in source['data'] if 'ifccad::drawing' in n.get('attributes',{}));prefix=drawing['path'];drawing['children']['annotationScale1']=prefix+'/annotationScale/1';drawing['attributes']['ifccad::drawing']['nextAnnotationScaleId']=2
source['data'].append({'path':prefix+'/annotationScale/1','attributes':{'ifccad::annotationScale':{'name':'1:20','paperUnits':1.,'drawingUnits':20.}}})
def json_presentation(scale):return {'pattern':{'families':[],'scale':scale,'orientationReference':[4.,5.,6.]},'loops':[{'kind':'shared','baseLoopIndex':0,'classification':{'textBox':True,'outermost':True,'isAnnotative':True,'doesNotSupportScale':True}},{'kind':'local','classification':{},'boundary':{'geometry':{'kind':'circle','center':[100.,0.],'radius':5.},'sourceEntityIds':[]}}]}
for n in source['data']:
 a=n.get('attributes',{})
 if 'ifccad::layout' in a:
  a['ifccad::layout'].pop('bounds',None);a['ifccad::layout'].pop('boundsQuality',None)
 if 'ifccad::hatch' in a:
  a['ifccad::annotation']={'mode':'disabled','reference':{'scaleId':prefix+'/annotationScale/1'},'contexts':[{'key':{'scaleId':prefix+'/annotationScale/1'},'presentation':json_presentation(2.)}]}
  a['ifccad::hatchAnnotationViews']={'nextContextId':5,'reference':2,'contexts':[{'id':i,'scaleId':prefix+'/annotationScale/1','presentation':json_presentation(scale),'viewState':{'normal':[0.,0.,2.],'rotation':0.25,'evaluate':False}} for i,scale in [(1,3.),(2,4.)]]}
  a['ifccad::hatchAnnotationViews']['contexts'][1]['presentation']['loops'][0]['classification'].update({'forceAnnotationVisible':True,'orientToPaper':True,'isAnnotativeBlock':True})
file='valid/annotation-hatch.ifcx';(js/file).write_text(json.dumps(source,indent=2)+'\n',encoding='utf-8');cases=json.loads((js/'cases.json').read_text(encoding='utf-8'));cases=[c for c in cases if c['file']!=file]+[dict(file=file,valid=True)];(js/'cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
