import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { createInspection } from '../src/inspection-model.mjs';
import { graphNeighborhood, relationshipSequence } from '../src/inspection-graph.mjs';
import { createRelationshipHistory } from '../src/inspection-graph-navigation.mjs';
import { inspectionNavigation } from '../src/inspection-navigation.mjs';

function openFixture(path) {
  const text = readFileSync(new URL(path, import.meta.url), 'utf8');
  return createInspection({ presentation: { graph: JSON.parse(text) } }, text);
}

test('opaque roots link to one shared preservation package', () => {
  const model = openFixture('../../conformance/next/ifccad/valid/preservation-shared.ifcx');
  const backing = '/cad/d1/preservation/r3';
  const field = 'ifccad::preservationRecord.storageBacking';
  const incoming = model.nodes.get(backing).incoming.filter(edge => edge.field === field);
  assert.deepEqual(incoming.map(edge => edge.source).sort(), [
    '/cad/d1/preservation/r1', '/cad/d1/preservation/r2',
  ]);
});

test('an instance points to its owner, its layer and the shared block definition', () => {
  const model = openFixture('../../tests/fixtures/ifccad/hello-cad.ifcx');
  const instance = graphNeighborhood(model, '/cad/d1/e3');
  assert.deepEqual(instance.incoming.flatMap(group => group.items.map(item => item.key)), ['/cad/d1/layout/1']);
  assert.deepEqual(instance.outgoing.map(group => [group.kind, group.items[0].key]), [
    ['layer', '/cad/d1/layer/2'], ['definition', '/cad/d1/block/1'],
  ]);
  const block = graphNeighborhood(model, '/cad/d1/block/1');
  assert.deepEqual(block.incoming.find(group => group.kind === 'definition').items.map(item => item.key), ['/cad/d1/e3', '/cad/d1/e5']);
  assert.equal(block.outgoing[0].entityType, 'lineSegment');
  assert.equal(block.outgoing[0].items[0].field, 'children.0');
  assert.equal(block.outgoing[0].items[0].key, '/cad/d1/e6');
});

test('presentation groups and large layer collections keep their identities and order', () => {
  const model = openFixture('../../examples/ifccad/entities.ifcx');
  const layers = graphNeighborhood(model, 'group:layer');
  assert.deepEqual(layers.outgoing[0].items.map(item => item.key), model.nodes.get('group:layer').children);
  const layer = model.nodes.get('/cad/d1/layer/0');
  assert.ok(layer.incoming.filter(edge => edge.kind === 'layer').length > 10);
  const { incoming } = graphNeighborhood(model, layer.key, 'order');
  const members = incoming.find(group => group.kind === 'layer');
  const current = incoming.find(group => group.kind === 'currentLayer');
  assert.equal(members.title, 'graph.incoming.layer.entities');
  assert.deepEqual(current.items.map(item => item.key), [model.roots[0]]);
  assert.deepEqual(members.items.map(item => item.key),
    layer.incoming.filter(edge => edge.kind === 'layer' && edge.source !== model.roots[0]).map(edge => edge.source));
});

test('tree and graph use authored order and group drawable relationships by entity type', () => {
  const path = '../../tests/fixtures/ifccad/hello-cad.ifcx';
  const source = JSON.parse(readFileSync(new URL(path, import.meta.url), 'utf8'));
  const layout = '/cad/d1/layout/1';
  const ids = ['e5', 'e4', 'e1', 'e3', 'e2'].map(id => '/cad/d1/' + id);
  source.data.find(node => node.path === layout).children = Object.fromEntries(ids.entries());
  const model = createInspection({ presentation: { graph: source } });
  const tree = inspectionNavigation(model, { grouping: 'order' });
  const ordered = graphNeighborhood(tree, layout, 'order').outgoing.find(group => group.kind === 'child');
  assert.equal(ordered.title, 'graph.outgoing.child.entities');
  assert.deepEqual(tree.children.get(layout), ids);
  assert.deepEqual(ordered.items.map(item => item.key), ids);

  const grouped = graphNeighborhood(model, layout, 'type').outgoing.filter(group => group.kind === 'child');
  assert.deepEqual(grouped.map(group => [group.entityType, group.items.map(item => item.key)]), [
    ['blockInstance', [ids[0], ids[3]]],
    ['circle', [ids[1]]],
    ['lineSegment', [ids[2]]],
    ['planarPolyline', [ids[4]]],
  ]);
  assert.equal(grouped[0].items[1].field, 'children.3');

  const symbols = model.nodes.get('/cad/d1/layer/2');
  const layerLinks = graphNeighborhood(model, symbols.key, 'order').incoming.find(group => group.kind === 'layer');
  assert.deepEqual(layerLinks.items.map(item => item.key), [ids[0], ids[1], ids[3]]);
});

test('drawing children follow the file tree collections and active fields have distinct links', () => {
  const model = openFixture('../../examples/ifccad/entities.ifcx');
  const drawing = model.roots[0];
  for (const grouping of ['type', 'order']) {
    const tree = inspectionNavigation(model, { grouping });
    const { outgoing } = graphNeighborhood(tree, drawing, grouping);
    const collections = outgoing.filter(group => group.kind === 'child');
    assert.deepEqual(collections.map(group => [group.title, group.items.length]), [
      ['layouts', 2], ['layers', 5], ['linePatterns', 4], ['symbolSources', 1], ['blocks', 1],
      ['modelWindows', 1], ['annotationScales', 2], ['textStyles', 1], ['dimensionStyles', 1],
      ['multiLeaderStyles', 1],
    ]);
    assert.deepEqual(collections[0].items.map(item => item.field), ['children.model', 'children.paper2']);
    assert.deepEqual(outgoing.filter(group => group.kind !== 'child').map(group => [
      group.kind, group.items[0].key, group.items[0].field,
    ]), [
      ['activeModelWindow', '/cad/d1/modelWindow/1', 'ifccad::modelViewState.activeModelWindow'],
      ['modelWindowList', '/cad/d1/modelWindow/1', 'ifccad::drawing.modelWindows'],
      ['currentLayer', '/cad/d1/layer/0', 'ifccad::drawingWorkspace.currentLayer'],
      ['activeLayout', '/cad/d1/layout/1', 'ifccad::drawingWorkspace.activeLayout'],
    ]);
  }
  const layout = graphNeighborhood(model, '/cad/d1/layout/1');
  assert.ok(layout.incoming.some(group => group.kind === 'activeLayout' && group.items[0].key === drawing));
  assert.ok(layout.outgoing.some(group => group.kind === 'currentAnnotationScale' &&
    group.items[0].field === 'layout.annotationState.currentScale'));
});

test('incoming text style links distinguish saved graphics, effective settings and direct text', () => {
  const text = readFileSync(new URL('../../examples/ifccad/entities.ifcx', import.meta.url), 'utf8');
  const style = '/cad/d1/textStyle/0';
  const graph = JSON.parse(text);
  const dimension = graph.data.find(node => node.attributes?.['ifccad::dimension'] && node.children?.graphics);
  const model = createInspection({ presentation: {
    graph,
    dimensions: [{ path: dimension.path, geometry: { effectiveSettings: { text: { style } } } }],
  } }, text);

  const { incoming } = graphNeighborhood(model, style);
  assert.deepEqual(incoming.map(group => group.kind).toSorted(), [
    'drawingTextStyle', 'dimensionStylesUsingTextStyle', 'linePatternsUsingTextStyle',
    'dimensionsUsingTextStyle', 'textsUsingStyle', 'paragraphTextsUsingStyle',
    'blockAttributesUsingStyle', 'storedDimensionTextsUsingStyle', 'attributeDefinitionsUsingStyle',
    'multiLeadersUsingTextStyle', 'multiLeaderStylesUsingTextStyle',
  ].toSorted());
  const links = incoming.flatMap(group => group.items.map(item => [item.key, item.field]));
  assert.deepEqual(links.toSorted(), model.nodes.get(style).incoming.map(edge => [edge.source, edge.field]).toSorted());
  assert.ok(incoming.find(group => group.kind === 'storedDimensionTextsUsingStyle')
    .items.some(item => item.key.startsWith(dimension.children.graphics + '/')));
  assert.deepEqual(incoming.find(group => group.kind === 'dimensionsUsingTextStyle')
    .items.map(item => item.field), ['derived.effectiveSettings.text.style']);
});

test('entity sequences follow the origin of the clicked relationship across type groups', () => {
  const model = openFixture('../../tests/fixtures/ifccad/hello-cad.ifcx');
  const layout = '/cad/d1/layout/1', layer = '/cad/d1/layer/2';
  const target = '/cad/d1/e3';
  const layoutGroup = graphNeighborhood(model, layout).outgoing.find(group => group.entityType === 'blockInstance');
  const layerGroup = graphNeighborhood(model, layer).incoming.find(group => group.entityType === 'blockInstance');
  assert.deepEqual(relationshipSequence(model, layout, 'outgoing', layoutGroup, target), {
    source: layout, keys: ['/cad/d1/e1', '/cad/d1/e2', target, '/cad/d1/e4', '/cad/d1/e5'],
  });
  assert.deepEqual(relationshipSequence(model, layer, 'incoming', layerGroup, target), {
    source: layer, keys: [target, '/cad/d1/e4', '/cad/d1/e5'],
  });
});

test('back and forward restore the sequence and a new visit clears the forward path', () => {
  const history = createRelationshipHistory();
  const sequence = { source: 'layer', keys: ['a', 'b', 'c'] };
  history.visit('layer');
  history.visit('a', sequence);
  assert.equal(history.adjacent(-1), undefined);
  assert.equal(history.adjacent(1), 'b');
  history.visit('b', sequence);
  history.visit('drawing');
  assert.equal(history.move(-1), 'b');
  assert.equal(history.current.sequence, sequence);
  assert.equal(history.adjacent(-1), 'a');
  assert.equal(history.peek(1), 'drawing');
  history.visit('c', sequence);
  assert.equal(history.peek(1), undefined);
  assert.equal(history.adjacent(1), undefined);
  assert.equal(history.move(-1), 'b');
  assert.equal(history.move(1), 'c');
  history.reset();
  assert.equal(history.current, undefined);
  assert.equal(history.move(-1), undefined);
});
