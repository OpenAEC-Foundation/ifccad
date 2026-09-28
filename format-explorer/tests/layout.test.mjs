import test from 'node:test';
import assert from 'node:assert/strict';
import { placeLabel, placeDetailNode, overlaps, routeEdge, wrapText } from '../src/layout.mjs';

test('a reference between nearly aligned nodes stays in their vertical gap', () => {
  const line={x:850,y:320,width:260,height:100};
  const scope={x:830,y:170,width:260,height:100};
  const route=routeEdge(line,scope);
  assert.equal(route.kind,'vertical');
  assert.deepEqual(route.start,{x:980,y:320});
  assert.deepEqual(route.end,{x:960,y:270});
  assert.ok(!route.path.includes(' 135'), 'the edge must not detour above the scope');
});

test('a target entirely to the left still uses the outer return corridor', () => {
  const route=routeEdge({x:850,y:320,width:260,height:100},{x:400,y:170,width:260,height:100});
  assert.equal(route.kind,'backward');
  assert.deepEqual(route.start,{x:850,y:370});
  assert.deepEqual(route.end,{x:660,y:220});
});

test('a short vertical reference may put its caption in the clear node gap', () => {
  const occupied=[{x:126,y:96,width:282,height:101},{x:144,y:221,width:282,height:101}];
  const label=placeLabel([{x:278,y:209}],75,24,occupied,0);
  assert.ok(label.y>=197&&label.y+label.height<=221);
  assert.ok(occupied.every(node=>!overlaps(label,node)));
});

test('new field nodes find space without moving reserved nodes', () => {
  const occupied = [{x:450,y:100,width:260,height:110},{x:450,y:250,width:260,height:110}];
  const original = structuredClone(occupied);
  const placed = placeDetailNode({x:450,y:100,width:260,height:93},occupied);
  assert.deepEqual(occupied, original);
  assert.equal(placed.x, 450);
  assert.ok(occupied.every(other => !overlaps(placed,other,32)));
});

test('a clear connection carries its label directly on the line', () => {
  const anchor = {x: 400, y: 325};
  const box = placeLabel([anchor], 110, 24, []);
  assert.equal(box.x + box.width / 2, anchor.x);
  assert.equal(box.y + box.height / 2, anchor.y);
});

test('a fanned-out polyline stream stays in its own branch corridor', () => {
  const occupied = [
    {x: 0, y: 0, width: 282, height: 101},
    {x: 450, y: 0, width: 282, height: 101},
    {x: 450, y: 130, width: 282, height: 101},
    {x: 450, y: 260, width: 282, height: 101},
    {x: 310, y: 140, width: 95, height: 26},
  ];
  const box = placeLabel([{x: 368, y: 260}, {x: 355, y: 205}, {x: 390, y: 290}], 110, 24, occupied);
  assert.ok(box.y >= 200, 'the label must stay with the lower branch');
  assert.ok(occupied.every(r => !overlaps(box, r, 6)));
});

test('relation labels avoid nodes, expansion controls and other labels', () => {
  const nodes = [
    {x: 40, y: 50, width: 316, height: 96},
    {x: 440, y: 50, width: 316, height: 96},
    {x: 440, y: 250, width: 316, height: 96},
  ];
  const occupied = [...nodes];
  for (const width of [182, 96, 72, 240, 60, 144]) {
    const box = placeLabel([{x: 400, y: 100}, {x: 420, y: 180}], width, 22, occupied);
    assert.ok(occupied.every(r => !overlaps(box, r, 6)));
    occupied.push(box);
  }
});

test('crowded label fallback always finds free space without losing text', () => {
  const occupied = [{x: -1000, y: -1000, width: 2000, height: 2000}];
  const box = placeLabel([{x: 0, y: 0}], 250, 24, occupied);
  assert.ok(!overlaps(box, occupied[0], 6));
  assert.equal(box.width, 250);
});

test('long names wrap at semantic boundaries and retain every character', () => {
  for (const value of ['PreservationRepresentation', 'preservation-golden-source', 'a'.repeat(120)]) {
    const lines = wrapText(value, 110, s => s.length * 8);
    assert.equal(lines.join(''), value);
    assert.ok(lines.every(s => s.length * 8 <= 110));
  }
});
