"""Independent persisted Dimension sizing literals; no production writer."""
from pathlib import Path
from copy import deepcopy
import json,struct
ROOT=Path(__file__).resolve().parent
exec(compile((ROOT/'dimension-kinds.py').read_text(encoding='utf-8').split('# Explicit code/body literals')[0],str(ROOT/'dimension-kinds.py'),'exec'))
binary=ROOT/'ifccad-cbor';js=ROOT/'ifccad'
base=decode((binary/'valid/graphics-empty.ifccad.cbor').read_bytes());doc=base[2]
styles=doc[13];style=deepcopy(styles[0]);doc[13]=[]
forms=[({0:0,1:{0:2.}},False),({0:1},False),({0:2,1:{}},True),({0:2,1:{0:3.}},True)]
for i,(sizing,annotative) in enumerate(forms):
 s=deepcopy(style);s[0]=i;s[1]=f'Sizing {i}';s[2][6]=sizing;s[4]=annotative;doc[13].append(s)
doc[2][10]=4
manifest=json.loads((binary/'cases.json').read_text(encoding='utf-8'))
def emit(name,document,valid=True,error=None):
 file=f"{'valid' if valid else 'invalid'}/{name}.ifccad.cbor";(binary/file).write_bytes(encode(document));(binary/(file+'.diag')).write_text(repr(document)+'\n',encoding='utf-8')
 c=dict(file=file,valid=valid,diagnostic=file+'.diag');
 if error:c['errorClass']=error
 manifest['cases']=[x for x in manifest['cases'] if x['file']!=file]+[c]
emit('dimension-sizing',base)
bad=deepcopy(base);bad[2][13][3][2][6][1][0]=0.;emit('dimension-sizing-zero-dormant',bad,False,'InvalidDocument')
bad=deepcopy(base);bad[2][13][2][2][6][1][0]=None;emit('dimension-sizing-null-dormant',bad,False,'Malformed')
bad=deepcopy(base);bad[2][13][1][2][6][1]={};emit('dimension-sizing-viewport-body',bad,False,'Malformed')
(binary/'cases.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
source=json.loads((js/'valid/dimension-linear-aligned.ifcx').read_text(encoding='utf-8'));original=next(n for n in source['data'] if 'ifccad::dimensionStyle' in n.get('attributes',{}));drawing=next(n for n in source['data'] if 'ifccad::drawing' in n.get('attributes',{}));drawing['attributes']['ifccad::drawing']['nextDimensionStyleId']=4
forms=[({'kind':'fixed','factor':2.},False),({'kind':'byViewport'},False),({'kind':'byAnnotationScale'},True),({'kind':'byAnnotationScale','dormantFactor':3.},True)]
for i,(sizing,annotative) in enumerate(forms):
 n=original if i==0 else deepcopy(original);n['path']=f'/cad/d1/dimensionStyle/{i}';s=n['attributes']['ifccad::dimensionStyle'];s['name']=f'Sizing {i}';s['settings']['presentationScale']=sizing;s['annotative']=annotative
 if i:source['data'].append(n)
 drawing['children'][f'dimensionStyle{i}']=n['path']
file='valid/dimension-sizing.ifcx';(js/file).write_text(json.dumps(source,indent=2)+'\n',encoding='utf-8');cases=json.loads((js/'cases.json').read_text(encoding='utf-8'));cases=[c for c in cases if c['file']!=file]+[dict(file=file,valid=True)];(js/'cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
