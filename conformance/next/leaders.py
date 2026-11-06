"""Independent literal Leader/MultiLeader/Tolerance cases for profile0.7/CBOR7.

Pins ordinary targets, local branch keys, style registry, labels and bounded
saved commands. It does not qualify any physical CAD ToleranceContent payload.
Only conformance/next is written; no production writer is called.
"""
from copy import deepcopy
from pathlib import Path
import json
from literal_cbor import decode, encode

ROOT=Path(__file__).resolve().parent
JSON_ROOT=ROOT/'ifccad'
BINARY_ROOT=ROOT/'ifccad-cbor'
plane={'origin':[0.,0.,0.],'xAxis':[1.,0.,0.],'yAxis':[0.,1.,0.]}
placement={0:[0.,0.,0.],1:[1.,0.,0.],2:[0.,1.,0.]}
common={
    'line':{'pathKind':'straightSegments','color':{'mode':'ByBlock'},'linePattern':{'mode':'ByLayer'},'lineWeight':{'kind':'byLayer'},'arrow':{'kind':'defaultClosedFilled'},'arrowSize':2.5},
    'landingEnabled':True,'doglegEnabled':True,'doglegLength':5.,'contentKind':'none',
    'text':{'color':{'mode':'ByBlock'},'height':2.5,'frame':False,
        'attachments':{'left':'middleOfText','right':'middleOfText','top':'centerOfText','bottom':'centerOfText'},
        'attachmentDirection':'horizontal','angle':'horizontal','alignment':'left'},
    'block':{'color':{'mode':'ByBlock'},'connection':'extents','rotation':0.,'scale':[1.,1.,1.]},'scaleFactor':1.,
}
literal_line={0:1,1:{0:1},2:{0:0},3:{0:0},4:{0:0},5:2.5}
literal_common={0:literal_line,1:True,2:True,3:5.,4:0,
    5:{1:{0:1},2:2.5,3:False,4:{0:2,1:2,2:9,3:9},5:0,6:1,7:0},
    6:{1:{0:1},2:0,3:0.,4:[1.,1.,1.]},7:1.}
settings={'common':common,'textAttachmentPoint':'center','textDirectionNegative':False,'extendLeaderToText':False}
literal_settings={0:literal_common,1:1,2:False,3:False}
default_text={'characterFormat':{},'paragraphFormat':{},'content':[{'paragraphFormat':{},'characterFormat':{},'inlines':[]}]}
style={'name':'leader conformance','description':'','common':common,'landingGap':0.625,'defaultMText':default_text,
    'textAlwaysLeft':False,'blockScaleEnabled':True,'blockRotationEnabled':True,'alignSpace':0.,'breakGapSize':0.,
    'maxLeaderPoints':0,'firstSegmentAngle':0.,'secondSegmentAngle':0.,'leaderDrawOrder':'headFirst','contentDrawOrder':'contentFirst','annotative':False}
literal_style={0:41,1:'leader conformance',2:'',3:literal_common,4:0.625,5:{0:{},1:{},2:[{0:{},1:{},2:[]}]},
    6:False,7:True,8:True,9:0.,10:0.,11:0,12:0.,13:0.,14:0,15:0,16:False}

source=json.loads((JSON_ROOT/'valid/dimension-radius.ifcx').read_text(encoding='utf-8'))
drawing=next(node for node in source['data'] if 'ifccad::drawing' in node['attributes'])
prefix=drawing['path'];model=next(node for node in source['data'] if node['path']==drawing['children']['model'])
layer=next(node['path'] for node in source['data'] if 'ifccad::layer' in node['attributes'])
dimension_style=next(node['path'] for node in source['data'] if 'ifccad::dimensionStyle' in node['attributes'])
next_id=max(900,drawing['attributes']['ifccad::drawing']['nextEntityId'])
tolerance_id,leader_id,multi_id=range(next_id,next_id+3)
def entity(identity,attributes,children=None):
    node={'path':f'{prefix}/e{identity}','attributes':{'ifccad::entity':{'layer':layer},**attributes}}
    if children is not None:node['children']=children
    model['children'][str(len(model['children']))]=node['path'];source['data'].append(node)
    return node
tolerance={'dimensionStyle':dimension_style,'overrides':{},'notation':{'kind':'cadFeatureControlTextV1','value':{'code':'{\\Fgdt;p}%%v0.1%%vA'}}}
entity(tolerance_id,{'ifccad::tolerance':tolerance,'ifccad::geom::placement':plane})
leader={'path':{'kind':'straightSegments','value':{'vertices':[[0.,0.,0.],[5.,5.,0.]]}},
    'annotationKind':'tolerance','annotationEntity':f'{prefix}/e{tolerance_id}','dimensionStyle':dimension_style,'overrides':{},
    'arrowEnabled':True,'hookEnabled':False,'hookDirection':'alongHorizontal','horizontalDirection':[1.,0.,0.],
    'blockOffset':[0.,0.,0.],'annotationOffset':[0.,0.,0.],'origin':[0.,0.,0.]}
entity(leader_id,{'ifccad::leader':leader,'ifccad::geom::placement':plane})
owner=f'{prefix}/e{multi_id}'
multi={'style':f'{prefix}/multiLeaderStyle/41','settings':settings,'overriddenProperties':['arrowSize'],
    'nextLabelId':4,'legacyArrowOverrides':[],
    'presentation':{'plane':plane,'normalReversed':False,'contentBasePoint':[0.,0.,0.],'scaleFactor':1.,'textHeight':2.5,'arrowSize':2.5,'landingGap':0.625,'nextPartId':10}}
entity(multi_id,{'ifccad::multiLeader':multi},{'leaders':owner+'/leaders','blockLabels':owner+'/labels','graphics':owner+'/graphics'})
line_settings=deepcopy(common['line']);line_settings['pathKind']='splineThroughVertices'
line={'key':9,'sourceIndex':900,'settings':line_settings,'overriddenProperties':['pathKind'],
    'vertices':[[0.,0.,0.]],'connection':'groupConnection','breaks':[{'segmentIndex':0,'intervals':[{'start':[2.,2.,0.],'end':[3.,3.,0.]}]}]}
group={'key':4,'sourceIndex':900,'hasConnectionPoint':True,'connectionPoint':[5.,5.,0.],
    'hasDoglegDirection':False,'doglegDirection':[0.,0.,0.],'doglegLength':0.,'textAttachmentDirection':'horizontal','breaks':[]}
def part(path,role,body,children=None):
    node={'path':path,'attributes':{role:body}}
    if children is not None:node['children']=children
    source['data'].append(node)
part(owner+'/leaders','ifccad::multiLeaderBranches',{}, {'0':owner+'/leaders/r4'})
part(owner+'/leaders/r4','ifccad::multiLeaderBranch',group,{'0':owner+'/leaders/r4/l9'})
part(owner+'/leaders/r4/l9','ifccad::multiLeaderLine',line)
part(owner+'/labels','ifccad::multiLeaderBlockLabels',{}, {'0':owner+'/labels/a3'})
part(owner+'/labels/a3','ifccad::multiLeaderBlockLabel',{'key':3,'sourceIndex':30,'width':0.,'text':'A-12'})
part(owner+'/graphics','ifccad::leaderGraphics',{'origin':'sourceSnapshot','placement':plane},{'0':owner+'/graphics/op0'})
part(owner+'/graphics/op0','ifccad::leaderGraphicsCommand',{'kind':'cadCompatibility51Zero'})
part(prefix+'/multiLeaderStyle/41','ifccad::multiLeaderStyle',style)
drawing['children']['multiLeaderStyle41']=prefix+'/multiLeaderStyle/41'
drawing['attributes']['ifccad::drawing']['nextMultiLeaderStyleId']=42
drawing['attributes']['ifccad::drawing']['nextEntityId']=next_id+3
# Saved bounds are not a regenerated evaluation of the new leader geometry.
model['attributes']['ifccad::layout'].pop('bounds',None)
model['attributes']['ifccad::layout'].pop('boundsQuality',None)

json_cases=json.loads((JSON_ROOT/'cases.json').read_text(encoding='utf-8'))
def emit_json(name,data,valid):
    file=f'{"valid" if valid else "invalid"}/{name}.ifcx'
    (JSON_ROOT/file).write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')
    json_cases[:]=[case for case in json_cases if case['file']!=file]
    json_cases.append({'file':file,'valid':valid})
emit_json('leaders-family',source,True)
bad=deepcopy(source)
next(node for node in bad['data'] if node['path']==f'{prefix}/e{leader_id}')['attributes']['ifccad::leader']['annotationEntity']=owner
emit_json('leader-wrong-target-kind',bad,False)
bad=deepcopy(source)
next(node for node in bad['data'] if node['path']==owner)['attributes']['ifccad::multiLeader']['style']=prefix+'/multiLeaderStyle/999'
emit_json('multileader-missing-style',bad,False)
bad=deepcopy(source);bad['data'].append({'path':owner+'/graphics/op99','attributes':{'ifccad::leaderGraphicsCommand':{'kind':'cadCompatibility51Zero'}}})
emit_json('leader-unowned-command',bad,False)
(JSON_ROOT/'cases.json').write_text(json.dumps(json_cases,indent=2)+'\n',encoding='utf-8')

binary=decode((BINARY_ROOT/'valid/dimension-radius.ifccad.cbor').read_bytes())
assert binary[1]==7
doc=binary[2];layer_id=doc[6][0][0];dim_id=doc[13][0][0];start=max(900,doc[2][0])
register=json.loads((ROOT.parent.parent/'schemas/ifccad/cbor-mapping-7.json').read_text(encoding='utf-8'))['types']
def required_empty_record(name):
    """The existing Dimension override groups remain required empty records.

    Read only that shared legacy topology from the language-neutral register;
    every new leader-family field/variant code below is independently literal.
    """
    record=register[name]
    assert record['kind']=='record',name
    return {field['code']:required_empty_record(field['type']) for field in record['fields'] if field['presence']=='required'}
empty_overrides=required_empty_record('DimensionStyleOverrides<IfccadTextStyleId,u64,IfccadColor,IfccadLinePatternId>')
literal_tolerance={0:placement,1:dim_id,2:empty_overrides,3:{0:0,1:{0:'{\\Fgdt;p}%%v0.1%%vA'}}}
literal_leader={0:placement,1:{0:0,1:{0:[[0.,0.,0.],[5.,5.,0.]]}},2:3,3:start,4:dim_id,5:empty_overrides,7:True,8:False,9:1,11:[1.,0.,0.],12:[0.,0.,0.],13:[0.,0.,0.],14:[0.,0.,0.]}
literal_spline=deepcopy(literal_line);literal_spline[0]=2
literal_branch_line={0:9,1:900,2:literal_spline,3:[0],4:[[0.,0.,0.]],5:0,6:[{0:0,1:[{0:[2.,2.,0.],1:[3.,3.,0.]}]}]}
literal_group={0:4,1:900,2:True,3:[5.,5.,0.],4:False,5:[0.,0.,0.],6:0.,7:0,8:[literal_branch_line],9:[]}
literal_multi={0:41,1:literal_settings,2:[9],3:[{0:3,2:30,3:0.,4:'A-12'}],4:4,5:[],
    6:{0:placement,1:False,2:[0.,0.,0.],3:1.,4:2.5,5:2.5,6:0.625,7:10,8:[literal_group],9:{}},
    8:{0:0,1:placement,2:[{0:12}]}}
for offset,kind,body in [(0,19,literal_tolerance),(1,17,literal_leader),(2,18,literal_multi)]:
    doc[5][5].append({0:0,1:{0:start+offset,1:layer_id,2:kind,3:{0:body}}})
doc[2][0]=start+3;doc[2][13]=42;doc[24]=[literal_style]
doc[5].pop(4,None);doc[5].pop(0,None)
manifest=json.loads((BINARY_ROOT/'cases.json').read_text(encoding='utf-8'))
def emit_binary(name,data,valid,error=None):
    file=f'{"valid" if valid else "invalid"}/{name}.ifccad.cbor'
    (BINARY_ROOT/file).write_bytes(encode(data))
    diagnostic=file+'.diag'
    (BINARY_ROOT/diagnostic).write_text('Independent literal profile0.7/CBOR7 leader-family map.\n'+repr(data)+'\n',encoding='utf-8')
    manifest['cases'][:]=[case for case in manifest['cases'] if case['file']!=file]
    row={'file':file,'valid':valid,'diagnostic':diagnostic}
    if error:row['errorClass']=error
    manifest['cases'].append(row)
emit_binary('leaders-family',binary,True)
bad=deepcopy(binary);bad[2][5][5][-2][1][3][0][3]=start+2
emit_binary('leader-wrong-target-kind',bad,False,'InvalidDocument')
bad=deepcopy(binary);bad[2][5][5][-1][1][3][0][2]=[0,0]
emit_binary('multileader-duplicate-override',bad,False,'Malformed')
bad=deepcopy(binary);bad[2][5][5][-1][1][3][0][8][2]=[{0:12,1:{}}]
emit_binary('leader-command-unit-body',bad,False,'Malformed')
(BINARY_ROOT/'cases.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
