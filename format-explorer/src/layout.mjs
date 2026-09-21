// Geometry in graph coordinates, independent of the current pan/zoom transform.
export function overlaps(a, b, gap = 0) {
  return a.x < b.x + b.width + gap && a.x + a.width + gap > b.x
    && a.y < b.y + b.height + gap && a.y + a.height + gap > b.y;
}

// New detail nodes find space without moving any already visible graph nodes.
export function placeDetailNode(preferred, occupied) {
  const box = {...preferred};
  while (occupied.some(other => overlaps(box, other, 32))) box.y += 36;
  return box;
}

export function wrapText(value, maxWidth, measure) {
  const lines = [];
  let rest = String(value);
  while (rest && measure(rest) > maxWidth) {
    let end = 1;
    while (end < rest.length && measure(rest.slice(0, end + 1)) <= maxWidth) end++;
    // Prefer word, identifier or camel-case boundaries without removing characters.
    const prefix = rest.slice(0, end + 1);
    const boundaries = [...prefix.matchAll(/[-_ /]|(?<=[a-z])(?=[A-Z])/g)]
      .map(m => m.index + (m[0] ? 1 : 0)).filter(i => i > end / 2 && i <= end);
    end = boundaries.at(-1) || end;
    lines.push(rest.slice(0, end));
    rest = rest.slice(end);
  }
  if (rest) lines.push(rest);
  return lines;
}

export function placeLabel(points, width, height, occupied) {
  const free = box => occupied.every(other => !overlaps(box, other, 6));
  const candidates = [];
  // Keep the caption on its connection whenever possible. Compare all nearby
  // candidates before accepting an offset; first-free searches can jump to a
  // different branch just because an earlier sample point was obstructed.
  points.forEach((point, index) => {
    const offsets = [[0, 0]];
    for (const distance of [height / 2 + 8, height + 16, height + 32])
      offsets.push([0, -distance], [0, distance], [-distance, 0], [distance, 0]);
    for (const [dx, dy] of offsets) {
      const box = {x: point.x + dx - width / 2, y: point.y + dy - height / 2,
        width, height, anchor: point};
      if (free(box)) candidates.push({box, score: Math.hypot(dx, dy) + index * 12});
    }
  });
  if (candidates.length) return candidates.sort((a, b) => a.score - b.score)[0].box;
  // Dense reference graphs still keep every label. Search nearby rows, with a
  // guaranteed free final row above all existing nodes and labels.
  const point = points[0];
  const ceiling = Math.min(point.y, ...occupied.map(r => r.y)) - height - 12;
  for (let y = point.y - height - 8; y > ceiling; y -= height + 12) {
    for (const dx of [0, -width - 12, width + 12]) {
      const box = {x: point.x - width / 2 + dx, y, width, height, anchor: point};
      if (free(box)) return box;
    }
  }
  return {x: point.x - width / 2, y: ceiling, width, height, anchor: point};
}
