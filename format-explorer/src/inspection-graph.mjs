import { listWindow } from './inspection-navigation.mjs';
import { itemTitle } from './inspection-view.mjs';

const membershipKinds = new Set(['child', 'member']);
const workspaceLinks = new Map([
  ['ifccad::modelViewState.activeModelWindow', 'activeModelWindow'],
  ['ifccad::drawing.modelWindows', 'modelWindowList'],
  ['ifccad::drawingWorkspace.currentLayer', 'currentLayer'],
  ['ifccad::drawingWorkspace.activeLayout', 'activeLayout'],
  ['layout.annotationState.currentScale', 'currentAnnotationScale'],
]);
const directTextStyleGroups = new Map([
  ['text', 'textsUsingStyle'],
  ['mText', 'paragraphTextsUsingStyle'],
  ['blockAttribute', 'blockAttributesUsingStyle'],
  ['attributeDefinition', 'attributeDefinitionsUsingStyle'],
]);
const textStyleReferenceGroups = new Map([
  ['dimension', 'dimensionsUsingTextStyle'],
  ['dimensionStyle', 'dimensionStylesUsingTextStyle'],
  ['linePattern', 'linePatternsUsingTextStyle'],
  ['multiLeader', 'multiLeadersUsingTextStyle'],
  ['multiLeaderStyle', 'multiLeaderStylesUsingTextStyle'],
]);

function incomingTextStyleKind(kind, source, grouping) {
  if (kind === 'child' && source?.type === 'drawing') return 'drawingTextStyle';
  if (kind === 'textStyle') return textStyleReferenceGroups.get(source?.type) ?? kind;
  if (kind !== 'style') return kind;
  if (source?.values?.['ifccad::dimensionGraphicsPart']) return 'storedDimensionTextsUsingStyle';
  return grouping === 'order' ? kind : directTextStyleGroups.get(source?.type) ?? kind;
}

function inDrawOrder(items) {
  // Draw positions are only comparable within a single owner.
  const byOwner = new Map();
  for (const item of items) {
    const { owner, drawPosition } = item.node ?? {};
    if (owner == null || drawPosition == null) continue;
    if (!byOwner.has(owner)) byOwner.set(owner, []);
    byOwner.get(owner).push(item);
  }
  for (const owned of byOwner.values()) {
    owned.sort((a, b) => a.node.drawPosition - b.node.drawPosition).reverse();
  }
  return items.map(item => item.node?.drawPosition == null ? item : byOwner.get(item.node.owner)?.pop() ?? item);
}

export function graphNeighborhood({ nodes, roots }, key, grouping = 'type') {
  const node = nodes.get(key);
  if (!node) return null;

  function drawingChildGroups(group) {
    const byTarget = new Map();
    for (const item of group.items) {
      if (!byTarget.has(item.key)) byTarget.set(item.key, []);
      byTarget.get(item.key).push(item);
    }

    const groups = [];
    for (const root of roots) {
      const collection = nodes.get(root);
      if (collection?.type !== 'group') continue;
      const items = collection.children.flatMap(key => {
        const links = byTarget.get(key) ?? [];
        byTarget.delete(key);
        return links;
      });
      if (items.length) groups.push({ kind: group.kind, title: collection.title, items });
    }
    const remaining = group.items.filter(item => byTarget.has(item.key));
    if (remaining.length) groups.push({ ...group, items: remaining });
    return groups;
  }

  function groups(edges, endpoint) {
    const byKind = new Map();
    for (const edge of edges) {
      const { field } = edge;
      const key = edge[endpoint];
      const related = nodes.get(key);
      let kind = workspaceLinks.get(field) ?? edge.kind;
      if (endpoint === 'source' && node.type === 'textStyle') kind = incomingTextStyleKind(kind, related, grouping);
      if (kind === 'layer' && field?.startsWith('ifccad::viewport.layerOverrides.')) kind = 'viewportLayerOverride';
      if (!byKind.has(kind)) byKind.set(kind, { kind, items: [] });
      byKind.get(kind).items.push({ key, field, node: related });
    }
    return [...byKind.values()].flatMap(group => {
      if (endpoint === 'target' && node.type === 'drawing' && group.kind === 'child') return drawingChildGroups(group);
      const items = inDrawOrder(group.items);
      if (grouping === 'order') {
        if (endpoint === 'source' && group.kind === 'layer') return { ...group, items, title: 'graph.incoming.layer.entities' };
        if (endpoint === 'target' && membershipKinds.has(group.kind) && items.every(item => item.node?.drawPosition != null)) {
          return { ...group, items, title: 'graph.outgoing.child.entities' };
        }
        return { ...group, items };
      }

      const byType = new Map();
      for (const item of items) {
        const type = item.node?.drawPosition == null ? undefined : item.node.type;
        if (!byType.has(type)) byType.set(type, []);
        byType.get(type).push(item);
      }
      const showTypeForMembership = endpoint === 'target' && membershipKinds.has(group.kind) && !byType.has(undefined);
      return byType.size < 2 && !showTypeForMembership ? { ...group, items } : [...byType].map(([entityType, items]) => ({
        kind: group.kind, entityType, items,
      }));
    });
  }

  const members = node.type === 'group' ? node.children.map(target => ({ target, kind: 'member' })) : [];
  return {
    node,
    incoming: groups(node.incoming, 'source'),
    outgoing: groups(members.length ? members : node.outgoing, 'target'),
  };
}

export function relationshipSequence(navigation, source, direction, group, selection) {
  const origin = navigation.nodes.get(source);
  const selected = navigation.nodes.get(selection);
  let keys;

  if (direction === 'incoming' && origin?.type === 'layer' && group?.kind === 'layer') {
    const members = graphNeighborhood(navigation, source, 'order').incoming.find(entry => entry.kind === 'layer');
    keys = members?.items.filter(item => item.node).map(item => item.key) ?? [];
  } else if (origin && direction === 'outgoing' && (!group || group.kind === 'child') &&
      selected?.drawPosition != null && selected.owner === source) {
    keys = origin.children.filter(key => navigation.nodes.has(key));
  } else {
    keys = group?.items.filter(item => item.node).map(item => item.key) ?? [];
  }

  const unique = [...new Set(keys)];
  return unique.length > 1 && unique.includes(selection) ? { source, keys: unique } : undefined;
}

const rowHeight = 54;
const visibleRows = 4;
const create = (tag, className, text) => {
  const element = document.createElement(tag);
  if (className) element.className = className;
  if (text != null) element.textContent = text;
  return element;
};

export function initializeInspectionGraph(container, t) {
  let previous, centeredAt;
  const track = container.querySelector('.graph-track');
  const focus = track.querySelector('.graph-focus');
  const focusTitle = focus.querySelector('strong');
  const focusPath = focus.querySelector('small');
  const incoming = track.querySelector('.graph-incoming');
  const outgoing = track.querySelector('.graph-outgoing');
  const groupContexts = new WeakMap();

  function positionFocus() {
    if (track.hidden || !container.clientHeight) return;
    const recenter = centeredAt == null || Math.abs(container.scrollTop - centeredAt) < 2;
    const centerSpace = (container.clientHeight - focus.offsetHeight) / 2;
    track.style.setProperty('--graph-center-space', `${Math.max(0, centerSpace)}px`);
    centeredAt = Math.max(0, focus.offsetTop - centerSpace);
    if (recenter) container.scrollTop = centeredAt;
  }

  const resize = new ResizeObserver(positionFocus);
  resize.observe(container);
  resize.observe(focus);

  function clearGraph() {
    centeredAt = null;
    track.hidden = true;
    incoming.replaceChildren();
    outgoing.replaceChildren();
    focusTitle.textContent = '';
    focusPath.textContent = '';
  }

  function relationTitle(group, direction) {
    const key = `graph.${direction}.${group.kind}`;
    const label = t(key);
    if (label !== key) return label;
    const fallback = `graph.kind.${group.kind}`;
    const kind = t(fallback);
    return kind === fallback ? group.kind : kind;
  }

  function groupTitle(group, direction) {
    if (group.title) return t(group.title);
    const relation = relationTitle(group, direction);
    if (!group.entityType) return relation;
    const type = t(`plural.${group.entityType}`);
    if (type.startsWith('plural.')) return relation;
    if (direction === 'outgoing' && membershipKinds.has(group.kind)) return type;
    if (direction === 'incoming' && group.kind === 'layer') return t('graph.incoming.layer.type', { type });
    return `${relation} · ${type}`;
  }

  function graphRow(item, language, index, total) {
    const title = item.node ? itemTitle(item.node, language) : t('unresolved');
    const row = create('li', 'graph-row');
    row.setAttribute('aria-posinset', index + 1);
    row.setAttribute('aria-setsize', total);
    const link = create(item.node ? 'button' : 'div', 'graph-node');
    if (item.node) {
      link.type = 'button';
      link.dataset.select = item.key;
    }
    const field = item.field;
    const id = item.key.split('/').at(-1);
    link.title = [title, item.key, item.field].filter(Boolean).join('\n');
    link.append(create('strong', '', title), create('small', '', [title.endsWith(` · ${id}`) && field ? '' : id, field].filter(Boolean).join(' · ')));
    row.append(link);
    return row;
  }

  function drawGroup(group, direction, language) {
    const { items } = group;
    const label = groupTitle(group, direction);
    const box = create('details', 'graph-group');
    box.dataset.kind = group.kind;
    groupContexts.set(box, { group, direction });
    const heading = create('summary', 'graph-group-heading');
    heading.append(create('span', '', label), create('span', 'graph-count', items.length));
    const list = create('ul', 'graph-items');
    list.setAttribute('aria-label', label);
    box.append(heading);

    if (items.length > visibleRows) {
      const viewport = create('div', 'graph-items-scroll');
      viewport.tabIndex = 0;
      viewport.setAttribute('role', 'region');
      viewport.setAttribute('aria-label', label);
      viewport.append(list);
      box.append(viewport);
      function draw() {
        if (!box.open) return;
        const { start, end, top, bottom } = listWindow(items.length, viewport.scrollTop, viewport.clientHeight || visibleRows * rowHeight, rowHeight, 2);
        const active = list.contains(document.activeElement) && document.activeElement.dataset.select;
        list.style.paddingBlock = `${top}px ${bottom}px`;
        list.replaceChildren(...items.slice(start, end).map((item, index) => graphRow(item, language, start + index, items.length)));
        if (active) ([...list.querySelectorAll('[data-select]')].find(button => button.dataset.select === active) || viewport).focus({ preventScroll: true });
      }
      box.addEventListener('toggle', draw);
      viewport.addEventListener('scroll', draw, { passive: true });
    } else {
      list.append(...items.map((item, index) => graphRow(item, language, index, items.length)));
      box.append(list);
    }
    return box;
  }

  function drawSide(side, groups, direction, language) {
    if (!groups.length) {
      side.replaceChildren();
      return;
    }

    const label = t(`graph.direction.${direction}`);
    const section = create('section', 'graph-groups');
    section.setAttribute('aria-label', label);
    for (const group of groups) section.append(drawGroup(group, direction, language));

    const connector = create('div', 'graph-connector');
    const arrow = create('span', 'graph-arrow', '↓');
    arrow.setAttribute('aria-hidden', 'true');
    connector.append(arrow, create('span', 'graph-connector-label', label));
    if (direction === 'incoming') side.replaceChildren(section, connector);
    else side.replaceChildren(connector, section);
  }

  return {
    show(navigation, key, language, grouping) {
      if (previous?.navigation === navigation && previous.key === key && previous.language === language && previous.grouping === grouping) return;
      previous = { navigation, key, language, grouping };
      const graph = graphNeighborhood(navigation, key, grouping);
      if (!graph) {
        clearGraph();
        return;
      }

      centeredAt = null;
      focusTitle.textContent = itemTitle(graph.node, language);
      focusPath.textContent = key;
      drawSide(incoming, graph.incoming, 'incoming', language);
      drawSide(outgoing, graph.outgoing, 'outgoing', language);
      if (!graph.incoming.length && !graph.outgoing.length) {
        outgoing.append(create('p', 'graph-empty secondary', t('noRelations')));
      }
      track.hidden = false;
      positionFocus();
    },
    sequenceFor(button) {
      const context = groupContexts.get(button.closest('.graph-group'));
      if (!context) return undefined;
      return relationshipSequence(previous.navigation, previous.key, context.direction, context.group, button.dataset.select);
    },
    focus() { focus.focus({ preventScroll: true }); },
    reset() { previous = null; clearGraph(); },
  };
}
