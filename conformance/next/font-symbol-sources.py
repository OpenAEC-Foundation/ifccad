"""Independent SHX source field literals; no production writer is invoked."""
from copy import deepcopy
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parent
helpers = (ROOT / 'dimension-kinds.py').read_text().split('# Explicit code/body literals')[0]
exec(compile(helpers, str(ROOT / 'dimension-kinds.py'), 'exec'))
binary = ROOT / 'ifccad-cbor'
wire = ROOT / 'ifccad'
base = decode((binary / 'valid/empty.ifccad.cbor').read_bytes())
assert base[1] == 5
base[2][2][6] = 9007199254740994
base[2][11] = [
    {0: 0, 1: {0: 0, 1: {0: 'missing.shx', 1: r'..\project\missing.shx'}}},
    {0: 9007199254740993, 1: {0: 0, 1: {0: 'missing.shx', 1: 'missing.shx'}}},
]
manifest = json.loads((binary / 'cases.json').read_text())
name = 'font-symbol-source-requests'
cases = [(name, base, True, None)]
for suffix, value, error in [
    ('null', None, 'Malformed'), ('wrong-type', 1, 'Malformed'),
    ('mismatch', 'other.shx', 'InvalidDocument'),
    ('empty', '', 'InvalidDocument'), ('trailing', 'project/', 'InvalidDocument'),
]:
    bad = deepcopy(base)
    bad[2][11][0][1][1][1] = value
    cases.append((name + '-' + suffix, bad, False, error))
bad = deepcopy(base)
bad[2][11][0][1][1][2] = 'unknown'
cases.append((name + '-unknown', bad, False, 'Malformed'))
for case, document, valid, error in cases:
    file = f"{'valid' if valid else 'invalid'}/{case}.ifccad.cbor"
    (binary / file).write_bytes(encode(document))
    (binary / (file + '.diag')).write_text(repr(document) + '\n')
    entry = dict(file=file, valid=valid, diagnostic=file + '.diag')
    if error:
        entry['errorClass'] = error
    manifest['cases'] = [c for c in manifest['cases'] if c['file'] != file] + [entry]
duplicate = deepcopy(base)
sentinel = '__SHX_LITERAL_BODY__'
duplicate[2][11][0][1][1] = sentinel
body = (b'\xa3' + encode(0) + encode('missing.shx') + encode(1)
        + encode('missing.shx') + encode(1) + encode('missing.shx'))
payload = encode(duplicate)
assert payload.count(encode(sentinel)) == 1
file = f'invalid/{name}-duplicate.ifccad.cbor'
(binary / file).write_bytes(payload.replace(encode(sentinel), body))
(binary / (file + '.diag')).write_text('Literal SHX body: map(3), 0:"missing.shx", 1:"missing.shx", 1:"missing.shx"\n')
manifest['cases'] = [c for c in manifest['cases'] if c['file'] != file] + [
    dict(file=file, valid=False, diagnostic=file + '.diag', errorClass='Malformed')]
(binary / 'cases.json').write_text(json.dumps(manifest, indent=2) + '\n')

base_json = json.loads((wire / 'valid/annotation-text.ifcx').read_text())
drawing = next(n for n in base_json['data'] if n['path'] == '/cad/d1')
drawing['attributes']['ifccad::drawing']['nextSymbolSourceId'] = 9007199254740994
for identifier, request in [(0, r'..\project\missing.shx'), (9007199254740993, 'missing.shx')]:
    path = f'/cad/d1/symbolSource/{identifier}'
    drawing['children'][f'source{identifier}'] = path
    base_json['data'].append(dict(path=path, attributes={'ifccad::symbolSource':
        dict(kind='shx', fileName='missing.shx', cadFileName=request)}))
json_manifest = json.loads((wire / 'cases.json').read_text())
json_cases = [(name, base_json, True)]
for suffix, value in [('null', None), ('wrong-type', 1), ('mismatch', 'other.shx'),
                      ('empty', ''), ('trailing', 'project/')]:
    bad = deepcopy(base_json)
    bad['data'][-2]['attributes']['ifccad::symbolSource']['cadFileName'] = value
    json_cases.append((name + '-' + suffix, bad, False))
bad = deepcopy(base_json)
bad['data'][-2]['attributes']['ifccad::symbolSource']['unknown'] = True
json_cases.append((name + '-unknown', bad, False))
for case, document, valid in json_cases:
    file = f"{'valid' if valid else 'invalid'}/{case}.ifcx"
    (wire / file).write_text(json.dumps(document, indent=2) + '\n')
    json_manifest = [c for c in json_manifest if c['file'] != file] + [dict(file=file, valid=valid)]
(wire / 'cases.json').write_text(json.dumps(json_manifest, indent=2) + '\n')
