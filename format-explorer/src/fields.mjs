/** Presentation of typed entity values; never written into the package. */
export function isVector(value) {
  return Array.isArray(value) ? value.length >= 2 && value.length <= 3 && value.every(v => typeof v === 'number')
    : value && typeof value === 'object' && Object.keys(value).length === 3 && ['x','y','z'].every(k => typeof value[k] === 'number');
}

export function fieldSummary(value) {
  if (isVector(value)) return '(' + (Array.isArray(value) ? value : [value.x,value.y,value.z]).join(', ') + ')';
  if (Array.isArray(value)) return value.length + ' punten';
  if (value && typeof value === 'object') return Object.keys(value).join(' · ');
  if (typeof value === 'string' && value.length > 28) return 'Verwijzing';
  return String(value);
}

export function addEntityFields(entities, {add, edge, byId, defaultCollapsed,register}) {
  for (const entity of entities) {
    defaultCollapsed.add(entity.id);
    function visit(parentId, key, value, path, index, implicit = false) {
      const parent = byId.get(parentId), id = `field:${entity.id}:${path.join('.')}`;
      const label = /^\d+$/.test(key) ? `punt ${key}` : key;
      const field = add({id, label, subtitle:implicit && path.length === 1 ? 'Standaard XY-vlak · impliciet' : fieldSummary(value),
        kind:'field', domain:'ifcdr', concept:Boolean(entity.concept), ownerId:entity.id,
        parentId, fieldPath:path, fieldIndex:index, implicit, resourceId:entity.resourceId,
        x:parent.x+270, y:parent.y+index*140, raw:value});
      edge(parentId,id,key,true,Boolean(entity.concept));
      parent.expandable=true;
      if (value && typeof value === 'object' && !isVector(value)) {
        defaultCollapsed.add(id);
        const entries=Object.entries(value),create=([k,v],i)=>visit(id,k,v,[...path,k],i,implicit);
        if(register)register(id,entries,create,([k])=>`field:${entity.id}:${[...path,k].join('.')}`);
        else entries.forEach(create);
      }
      if (path[0] === 'styleRef' && byId.has(value)) edge(id,value,'verwijzing',false,Boolean(entity.concept));
      return field;
    }
    Object.entries(entity.geometry).forEach(([key,value],i) => {
      const implicit = key === 'placement' && value === 'identity';
      const resolved = implicit ? {origin:{x:0,y:0,z:0},X:{x:1,y:0,z:0},Y:{x:0,y:1,z:0}} : value;
      visit(entity.id,key,resolved,[key],i,implicit);
    });
  }
}
