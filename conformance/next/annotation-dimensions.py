"""Independent per-kind annotation contexts; no production writer calls."""
from pathlib import Path
from copy import deepcopy
import json,struct
ROOT=Path(__file__).resolve().parent
exec(compile((ROOT/'dimension-kinds.py').read_text(encoding='utf-8').split('# Explicit code/body literals')[0],str(ROOT/'dimension-kinds.py'),'exec'))
binary=ROOT/'ifccad-cbor';js=ROOT/'ifccad'
manifest=json.loads((binary/'cases.json').read_text(encoding='utf-8'));cases=json.loads((js/'cases.json').read_text(encoding='utf-8'))
files=['dimension-linear-aligned','dimension-angular-two-lines','dimension-angular-three-points','dimension-radius','dimension-diameter','dimension-ordinate-signed','dimension-arc-length-full','dimension-jogged-radius']
points=[['dimensionLinePoint'],['dimensionLinePoint'],['dimensionArcPoint'],['dimensionArcPoint'],['curvePoint'],['firstDiameterPoint','secondDiameterPoint'],['featurePoint','leaderEnd'],['dimensionArcPoint'],['displayCenter','jogPoint']]
layout={i:{0:i%2==1,1:i%2==0} for i in range(5)};layout.update({5:True,6:False,7:True})
def emit(name,document,valid=True,error=None):
 file=f"{'valid' if valid else 'invalid'}/{name}.ifccad.cbor";(binary/file).write_bytes(encode(document));(binary/(file+'.diag')).write_text(repr(document)+'\n',encoding='utf-8');c=dict(file=file,valid=valid,diagnostic=file+'.diag')
 if error:c['errorClass']=error
 manifest['cases']=[x for x in manifest['cases'] if x['file']!=file]+[c]
for filename in files:
 base=decode((binary/f"valid/{'graphics-empty' if filename=='dimension-linear-aligned' else filename}.ifccad.cbor").read_bytes());d=base[2];
 if filename=='dimension-linear-aligned':
  aligned=deepcopy(next(e for e in d[5][5] if e.get(0)==0 and e[1][2]==13));aligned[1][0]=d[2][0];d[2][0]+=1;aligned[1][3][0][2]={0:1,1:{0:[0.,0.,0.],1:[3.,4.,0.],2:[2.,2.]}};aligned[1][3][0][6]=[];d[5][5].append(aligned)
 d[2][11]=2;d[20]=[{0:1,1:'1:50',2:1.,3:50.}];d[5].pop(0,None);d[5].pop(4,None)
 for entity in d[5][5]:
  if entity.get(0)!=0 or entity[1][2]!=13:continue
  dimension=entity[1][3][0];kind=dimension[2][0];geometry={0:kind,1:{i:[10.+i,20.+i,30.+i] for i in range(len(points[kind]))}}
  dimension[8]={0:2,2:[{0:{0:1},1:{0:[100.,200.],1:False,2:0.5,3:deepcopy(layout),4:geometry,5:{0:0,1:[]}}}]}
 emit('annotation-'+filename,base)
 if filename=='dimension-linear-aligned':
  target=next(e[1][3][0] for e in base[2][5][5] if e.get(0)==0 and e[1][2]==13)
  bad=deepcopy(base);target_bad=next(e[1][3][0] for e in bad[2][5][5] if e.get(0)==0 and e[1][2]==13);target_bad[8][2][0][1][4]={0:5,1:{0:[1.,2.,3.],1:[4.,5.,6.]}};emit('annotation-dimension-kind-mismatch',bad,False,'InvalidDocument')
  bad=deepcopy(base);target_bad=next(e[1][3][0] for e in bad[2][5][5] if e.get(0)==0 and e[1][2]==13);target_bad[8][2][0][1][3][0].pop(1);emit('annotation-dimension-missing-dormant-value',bad,False,'Malformed')
 source=json.loads((js/f'valid/{filename}.ifcx').read_text(encoding='utf-8'));drawing=next(n for n in source['data'] if 'ifccad::drawing' in n.get('attributes',{}));prefix=drawing['path'];drawing['children']['annotationScale1']=prefix+'/annotationScale/1';drawing['attributes']['ifccad::drawing']['nextAnnotationScaleId']=2
 source['data'].append({'path':prefix+'/annotationScale/1','attributes':{'ifccad::annotationScale':{'name':'1:50','paperUnits':1.,'drawingUnits':50.}}})
 kinds=['linear','aligned','angularTwoLines','angularThreePoints','radius','diameter','ordinate','arcLength','joggedRadius']
 for n in source['data']:
  a=n.get('attributes',{})
  if 'ifccad::layout' in a:a['ifccad::layout'].pop('bounds',None);a['ifccad::layout'].pop('boundsQuality',None)
  if 'ifccad::dimension' in a:
   kind=a['ifccad::dimension']['definition']['kind'];index=kinds.index(kind);geometry={'kind':kind,**{f:[10.+i,20.+i,30.+i] for i,f in enumerate(points[index])}};jl={n:{'active':i%2==1,'value':i%2==0} for i,n in enumerate(['lineOutside','suppressArrowsOutside','fitSwitch','forceTextInside','textMovementSwitch'])};jl.update(secondArrowPresent=True,flipFirst=False,flipSecond=True)
   a['ifccad::annotation']={'mode':'byDefinition','contexts':[{'key':{'scaleId':prefix+'/annotationScale/1'},'presentation':{'textLocation':[100.,200.],'userTextLocation':False,'textRotation':0.5,'layout':jl,'geometry':geometry,'graphics':{'origin':'sourceSnapshot','parts':[]}}}]}
 file=f'valid/annotation-{filename}.ifcx';(js/file).write_text(json.dumps(source,indent=2)+'\n',encoding='utf-8');cases=[c for c in cases if c['file']!=file]+[dict(file=file,valid=True)]
(binary/'cases.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8');(js/'cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
