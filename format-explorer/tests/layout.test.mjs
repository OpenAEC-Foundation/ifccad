import test from 'node:test';
import assert from 'node:assert/strict';
import { placeLabel, placeDetailNode, overlaps, wrapText } from '../src/layout.mjs';

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
