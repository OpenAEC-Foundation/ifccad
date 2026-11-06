"""Independent literal Dimension cases; never calls a production writer.

Uses the existing graphics-empty CBOR envelope and the existing
linear/aligned IFCX fixture. Replaces only the Dimension definition/references.
Run from any directory. Numbered conformance collections are never touched.
"""
from pathlib import Path
from copy import deepcopy
import json
from literal_cbor import decode, encode

ROOT = Path(__file__).resolve().parent






# Explicit code/body literals, independent of the Rust enum or mapping writer.
CASES = [
    ('radius', {0: 4, 1: {0: [0., 0., 7.], 1: [3., 4., 9.], 2: 0.}},
     dict(kind='radius', center=[0., 0., 7.], curvePoint=[3., 4., 9.], leaderLength=0.)),
    ('diameter', {0: 5, 1: {0: [-3., -4., 7.], 1: [3., 4., 9.], 2: 0.}},
     dict(kind='diameter', firstDiameterPoint=[-3., -4., 7.], secondDiameterPoint=[3., 4., 9.], leaderLength=0.)),
    ('angular-two-lines', {0: 2, 1: {0: [0., 0., 7.], 1: [1., 0., 8.], 2: [0., 0., 9.], 3: [0., 1., 10.], 4: 0, 5: 0, 6: 0, 7: [5., 5.]}},
     dict(kind='angularTwoLines', line1Start=[0., 0., 7.], line1End=[1., 0., 8.], line2Start=[0., 0., 9.], line2End=[0., 1., 10.], ray1Sign='positive', ray2Sign='positive', traversal='counterclockwise', dimensionArcPoint=[5., 5.])),
    ('angular-three-points', {0: 3, 1: {0: [0., 0., 7.], 1: [1., 0., 8.], 2: [0., 1., 9.], 3: 1, 4: [5., 5.]}},
     dict(kind='angularThreePoints', vertex=[0., 0., 7.], firstPoint=[1., 0., 8.], secondPoint=[0., 1., 9.], traversal='clockwise', dimensionArcPoint=[5., 5.])),
    ('ordinate-signed', {0: 6, 1: {0: [10., 20., 3.], 1: [7., 24., 8.], 2: 0., 3: [20., 40.], 4: 1}},
     dict(kind='ordinate', origin=[10., 20., 3.], featurePoint=[7., 24., 8.], axisAngle=0., leaderEnd=[20., 40.], valueMode='signed')),
    ('arc-length-full', {0: 7, 1: {0: [0., 0., 7.], 1: 100., 2: 0., 3: {0: 1, 1: {0: 1}}, 4: [120., 0.], 5: {0: [100., 0.], 1: [120., 0.]}}},
     dict(kind='arcLength', center=[0., 0., 7.], radius=100., startParameter=0., sweep=dict(kind='full', traversal='clockwise'), dimensionArcPoint=[120., 0.], leader=dict(start=[100., 0.], end=[120., 0.]))),
    ('jogged-radius', {0: 8, 1: {0: [0., 0., 7.], 1: [3., 4., 9.], 2: [100., 200.], 3: [50., 50.], 4: .5}},
     dict(kind='joggedRadius', center=[0., 0., 7.], curvePoint=[3., 4., 9.], displayCenter=[100., 200.], jogPoint=[50., 50.], jogAngle=.5)),
]

binary_root = ROOT/'ifccad-cbor'
base = decode((binary_root/'valid/graphics-empty.ifccad.cbor').read_bytes())
assert base[1] == 5
manifest = json.loads((binary_root/'cases.json').read_text())
json_root = ROOT/'ifccad'
json_base = json.loads((json_root/'valid/dimension-linear-aligned.ifcx').read_text())
json_cases = json.loads((json_root/'cases.json').read_text())


def emit(name, definition, valid=True, error=None):
    document = deepcopy(base)
    dimension = document[2][5][5][0][1][3][0]
    dimension[2] = definition
    dimension[6] = []
    file = f"{'valid' if valid else 'invalid'}/dimension-{name}.ifccad.cbor"
    (binary_root/file).write_bytes(encode(document))
    (binary_root/(file+'.diag')).write_text(
        'Independent literal Dimension definition: '+repr(definition)+'\n'
        'Envelope: existing graphics-empty fixture; mapping 4. Definitions above are independently authored.\n',
        encoding='utf-8', newline='\n')
    case = dict(file=file, diagnostic=file+'.diag', valid=valid)
    if error:
        case['errorClass'] = error
    manifest['cases'] = [x for x in manifest['cases'] if x['file'] != file]+[case]


for name, wire, logical in CASES:
    emit(name, wire)
    drawing = deepcopy(json_base)
    dimension = next(node['attributes']['ifccad::dimension'] for node in drawing['data'] if 'ifccad::dimension' in node.get('attributes', {}))
    dimension['definition'] = logical
    dimension['associations'] = []
    file = f'valid/dimension-{name}.ifcx'
    (json_root/file).write_text(json.dumps(drawing, indent=2, ensure_ascii=False)+'\n', encoding='utf-8', newline='\n')
    json_cases = [x for x in json_cases if x['file'] != file]+[dict(file=file, valid=True)]

bad = deepcopy(CASES[0][1]); bad[1][2] = -1.
emit('negative-leader', bad, False, 'InvalidDocument')
bad = deepcopy(CASES[0][1]); bad[1][0] = [0., 0.]
emit('short-center', bad, False, 'Malformed')
bad = deepcopy(CASES[2][1]); bad[1][4] = 2
emit('unknown-ray-sign', bad, False, 'Malformed')
bad = deepcopy(CASES[5][1]); bad[1][3] = {0: 0, 1: {0: 6.283185307179586}}
emit('partial-full-turn', bad, False, 'InvalidDocument')
bad = deepcopy(CASES[5][1]); bad[1][3][1][1] = 0.
emit('full-span-extra-field', bad, False, 'Malformed')
bad = deepcopy(CASES[6][1]); bad[1][2] = None
emit('null-display-center', bad, False, 'Malformed')
(binary_root/'cases.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8', newline='\n')
(json_root/'cases.json').write_text(json.dumps(json_cases, indent=2)+'\n', encoding='utf-8', newline='\n')
