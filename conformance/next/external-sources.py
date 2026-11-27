"""Independent literal external-source/XREF contract cases; no production writer."""
from pathlib import Path
import copy
import json
from literal_cbor import decode, encode

ROOT = Path(__file__).resolve().parent
GUID = 'e5b4a9af-cf20-4fc9-a30f-ffb20fd987e1'
SOURCE = '/cad/d1/externalSource/1'
PLACEMENT = {'origin':[10000.,5000.,0.], 'xAxis':[0.,1.,0.], 'yAxis':[-1.,0.,0.]}

def write(path, value):
    path.write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')

root=ROOT/'ifccad'
graph=json.loads((root/'valid-circle.ifcx').read_bytes())
nodes={n['path']:n for n in graph['data']}
nodes['/cad/d1']['attributes']['ifccad::drawing']['nextExternalSourceId']=2
nodes['/cad/d1']['children']['externalSource1']=SOURCE
nodes['/cad/d1/layout/1']['children']={'0':'/cad/d1/e1000'}
nodes['/cad/d1/layout/42']['children']={'0':'/cad/d1/e1001','1':'/cad/d1/e1002'}
source={'name':'grondplan','location':{'kind':'uriReference','uri':'../Referenties/grondplan.dwg'},'source':{'format':'dwg','basePoint':[10.,20.,0.],'insertionUnit':'m','fingerprintGuid':GUID}}
graph['data'].append({'path':SOURCE,'attributes':{'ifccad::externalSource':source}})
attrs=nodes['/cad/d1/e1000']['attributes']
for key in list(attrs):
    if key.startswith('ifccad::geom::'): del attrs[key]
attrs.update({'ifccad::xref':{'source':SOURCE,'transform':{'placement':PLACEMENT,'scale':[500.,500.,500.]}}})
# External full extents cannot be derived from local insertion coordinates.
for n in graph['data']:
    for key in ['ifccad::layout','ifccad::drawing']:
        if key in n['attributes']:
            n['attributes'][key].pop('bounds',None)
            n['attributes'][key].pop('boundsQuality',None)

cases=json.loads((root/'cases.json').read_bytes())
def emit_json(name,valid,change=None):
    v=copy.deepcopy(graph)
    if change: change(v)
    name=f'{"valid" if valid else "invalid"}-xref-{name}.ifcx'
    write(root/name,v)
    if not any(c['file']==name for c in cases): cases.append({'file':name,'valid':valid})
def src(v): return v['data'][-1]['attributes']['ifccad::externalSource']
def xref(v): return next(n['attributes']['ifccad::xref'] for n in v['data'] if 'ifccad::xref' in n['attributes'])
emit_json('relative',True)
emit_json('search',True,lambda v:src(v).update(location={'kind':'searchFileName','fileName':'grondplan.dwg'}))
emit_json('unsupported-scheme',True,lambda v:src(v)['location'].update(uri='custom:grondplan'))
def json_format(v, name):
    src(v)['source']['format']=name
    src(v)['location']['uri']=f'../Referenties/grondplan.{name}'
    if name=='ifccad':
        src(v)['source'].pop('fingerprintGuid',None)
        src(v)['source']['basePoint']=[0.,0.,0.]
emit_json('dxf',True,lambda v:json_format(v,'dxf'))
emit_json('ifccad',True,lambda v:json_format(v,'ifccad'))
def native_guid(v):
    json_format(v,'ifccad')
    src(v)['source']['fingerprintGuid']=GUID
emit_json('ifccad-guid',False,native_guid)
emit_json('unknown-format',False,lambda v:json_format(v,'unknown'))
emit_json('mixed-location',False,lambda v:src(v)['location'].update(fileName='grondplan.dwg'))
emit_json('percent-escape',False,lambda v:src(v)['location'].update(uri='bad%GG'))
emit_json('guid',False,lambda v:src(v)['source'].update(fingerprintGuid='not-a-guid'))
emit_json('source-ref',False,lambda v:xref(v).update(source='/cad/d2/externalSource/1'))
emit_json('scale',False,lambda v:xref(v)['transform'].update(scale=[0.,1.,1.]))
def paper_owner(v):
    n={n['path']:n for n in v['data']}
    n['/cad/d1/layout/1']['children']={}
    n['/cad/d1/layout/42']['children']['2']='/cad/d1/e1000'
emit_json('paper-owner',True,paper_owner)
emit_json('overlay',True,lambda v:xref(v).update(referenceType='overlay'))
emit_json('attach-explicit',True,lambda v:xref(v).update(referenceType='attach'))
emit_json('reference-type',False,lambda v:xref(v).update(referenceType='other'))
emit_json('reference-type-null',False,lambda v:xref(v).update(referenceType=None))
write(root/'cases.json',cases)

root=ROOT/'ifccad-cbor'
v=decode((root/'valid/line-xy.ifccad.cbor').read_bytes())
d=v[2]; d[2][17]=2
d[29]=[{0:1,1:'grondplan',2:{0:0,1:{0:'../Referenties/grondplan.dwg'}},3:{0:0,1:{0:[10.,20.,0.],1:'m',2:GUID}}}]
d[5].pop(4,None); d[5].pop(0,None)
d[5][5][0][1][2]=22
d[5][5][0][1][3]={0:{0:1,1:{0:{0:[10000.,5000.,0.],1:[0.,1.,0.],2:[-1.,0.,0.]},2:[500.,500.,500.]}}}
manifest=json.loads((root/'cases.json').read_bytes())
def emit_cbor(name,valid,change=None):
    value=copy.deepcopy(v)
    if change: change(value[2])
    file=f'{"valid" if valid else "invalid"}/xref-{name}.ifccad.cbor'
    (root/file).write_bytes(encode(value))
    diagnostic=file+'.diag'
    (root/diagnostic).write_text(repr(value).replace('False','false').replace('True','true').replace("'",'"')+'\n',encoding='utf-8')
    case={'file':file,'valid':valid,'diagnostic':diagnostic}
    if not valid: case['errorClass']='Malformed' if name in ('mixed-location','ifccad-guid','unknown-format','reference-type','reference-type-null','legacy-scale') else 'InvalidDocument'
    old=next((c for c in manifest['cases'] if c['file']==file),None)
    if old is not None: old.update(case)
    else: manifest['cases'].append(case)
emit_cbor('relative',True)
emit_cbor('search',True,lambda d:d[29][0].update({2:{0:1,1:{0:'grondplan.dwg'}}}))
def cbor_format(d, name, code):
    d[29][0][2][1][0]=f'../Referenties/grondplan.{name}'
    d[29][0][3][0]=code
    if code==2:
        d[29][0][3][1].pop(2,None)
        d[29][0][3][1][0]=[0.,0.,0.]
emit_cbor('dxf',True,lambda d:cbor_format(d,'dxf',1))
emit_cbor('ifccad',True,lambda d:cbor_format(d,'ifccad',2))
def cbor_native_guid(d):
    cbor_format(d,'ifccad',2)
    d[29][0][3][1][2]=GUID
emit_cbor('ifccad-guid',False,cbor_native_guid)
emit_cbor('unknown-format',False,lambda d:cbor_format(d,'unknown',3))
emit_cbor('mixed-location',False,lambda d:d[29][0][2][1].update({1:'bad'}))
emit_cbor('guid',False,lambda d:d[29][0][3][1].update({2:'bad'}))
emit_cbor('source-ref',False,lambda d:d[5][5][0][1][3][0].update({0:2}))
emit_cbor('scale',False,lambda d:d[5][5][0][1][3][0][1].update({2:[0.,1.,1.]}))
emit_cbor('legacy-scale',False,lambda d:d[5][5][0][1][3][0].update({2:1.}))
emit_cbor('overlay',True,lambda d:d[5][5][0][1][3][0].update({3:1}))
emit_cbor('attach-explicit',True,lambda d:d[5][5][0][1][3][0].update({3:0}))
emit_cbor('reference-type',False,lambda d:d[5][5][0][1][3][0].update({3:2}))
emit_cbor('reference-type-null',False,lambda d:d[5][5][0][1][3][0].update({3:None}))
write(root/'cases.json',manifest)
