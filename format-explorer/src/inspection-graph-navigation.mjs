export function createRelationshipHistory() {
  const entries = [];
  let index = -1;
  const peek = step => entries[index + step]?.key;

  return {
    get current() { return entries[index]; },
    peek,
    visit(key, sequence) {
      if (!key) return;
      if (entries[index]?.key === key) {
        if (sequence) entries[index] = { key, sequence };
        return;
      }
      entries.splice(index + 1);
      entries.push({ key, sequence });
      index = entries.length - 1;
    },
    move(step) {
      const key = peek(step);
      if (key) index += step;
      return key;
    },
    adjacent(step) {
      const entry = entries[index];
      if (!entry?.sequence) return undefined;
      const { keys } = entry.sequence;
      const position = keys.indexOf(entry.key);
      return position === -1 ? undefined : keys[position + step];
    },
    reset() {
      entries.length = 0;
      index = -1;
    },
  };
}
