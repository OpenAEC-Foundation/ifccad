/** Read-only view model for curated repository examples, not a package validator. */
import { addEntityFields } from './fields.mjs';
import { createCollections } from './collections.mjs';
export function buildModel(fixture, { concepts = false } = {}) {
  const nodes = [], edges = [], entities = [], resources = [], preservations = [];
  const byId = new Map(), defaultCollapsed = new Set(), roots = [];
  const add = n => { if (!byId.has(n.id)) { nodes.push(n); byId.set(n.id, n); } return byId.get(n.id); };
  const edge = (source, target, relation, structural = false, concept = false) => edges.push({source,target,relation,structural,concept});
  const paging=createCollections({add,edge,byId,defaultCollapsed});
  const fieldContext={add,edge,byId,defaultCollapsed,register:paging.register};
  const drawings = fixture.ifcx.data.filter(n => n.type === 'openaec:Drawing');
  const definitionsY = 560 + Math.max(0, drawings.length - 1) * 460;
  let layerIndex=0, appearanceIndex=0;
  const pagedDefinitions=new Map(['Layer','Appearance'].map(kind=>[kind,fixture.ifcx.data.filter(n=>n.type==='openaec:'+kind)]).filter(([,items])=>items.length>paging.pageSize));
  for(const [kind,items]of pagedDefinitions){
    const id='view:definitions:'+kind;
    add({id,label:kind==='Layer'?'Lagen':'Appearances',subtitle:items.length+' definities',kind:'group',domain:'ifcx',x:kind==='Layer'?40:580,y:definitionsY,raw:null});roots.push(id);
    paging.register(id,items,(raw,i)=>{
      const key='ifcx:'+raw.path;
      add({id:key,label:kind,subtitle:raw.attributes?.name||raw.path,kind,domain:'ifcx',x:310,y:definitionsY+(i%paging.pageSize)*150,raw});
      edge(id,key,'definitie',true);
      for(const [relation,children]of Object.entries(raw.children||{}))for(const path of Array.isArray(children)?children:[children])edge(key,'ifcx:'+path,relation,true);
      if(raw.attributes?.appearance)edge(key,'ifcx:'+raw.attributes.appearance,'appearance');
    },raw=>'ifcx:'+raw.path);
  }
  for (const raw of fixture.ifcx.data) {
    const kind=typeof raw.type==='string'?raw.type.replace('openaec:',''):'IFCX';
    if(pagedDefinitions.has(kind))continue;
    let i=drawings.findIndex(d => d.path===raw.path || d.children?.Representation===raw.path || d.children?.Layouts?.includes(raw.path));
    i=Math.max(0,i);
    const positions={DrawingSet:[40,50],Drawing:[40,215+i*460],DrawingLayout:[40,365+i*460],DrawingRepresentation:[310,290+i*460],PreservationRepresentation:[310,50],Layer:[40,definitionsY+layerIndex*150],Appearance:[310,definitionsY+appearanceIndex*150]};
    if(kind==='Layer')layerIndex++;
    if(kind==='Appearance')appearanceIndex++;
    const [x,y]=Object.hasOwn(positions,kind)?positions[kind]:[40,definitionsY+230];
    const n=add({id:'ifcx:'+raw.path,label:kind,subtitle:raw.attributes?.name||raw.path,kind,domain:'ifcx',x,y,raw});
    if(kind==='DrawingSet')roots.unshift(n.id);
    if(['Layer','Appearance'].includes(kind))roots.push(n.id);
    for(const [relation,children]of Object.entries(raw.children||{}))for(const path of Array.isArray(children)?children:[children])edge(n.id,'ifcx:'+path,relation,true);
    if(raw.attributes?.appearance)edge(n.id,'ifcx:'+raw.attributes.appearance,'appearance');
    for(const key of ['resource','preservation']) {
      if(key==='resource'?kind!=='DrawingRepresentation':kind!=='PreservationRepresentation')continue;
      const descriptor=raw.attributes?.[key];if(!descriptor)continue;
      const body=descriptor.content||fixture.files[descriptor.uri];
      if(!body)throw new Error('Ontbrekende voorbeeldresource: '+descriptor.resourceId);
      const isDrawing=key==='resource', rid=body.header.resourceId, id='resource:'+rid;
      const item={id,resourceId:rid,body,descriptor,storage:descriptor.content?'inline':'extern',source:descriptor.uri||'package.ifcx.json · content',node:n.id};
      const list=isDrawing?resources:preservations;
      if(!list.some(r=>r.resourceId===rid))list.push(item);
      add({id,label:isDrawing?'IFCDR':'IFCPR',subtitle:rid,kind:isDrawing?'drawing-resource':'preservation-resource',domain:isDrawing?'ifcdr':'ifcpr',x:580,y:isDrawing?290+i*460:50,raw:body,resourceId:rid,item});
      edge(n.id,id,descriptor.content?'content · inline':'uri · extern',true);
      defaultCollapsed.add(id);
    }
  }
  for(const resource of resources) {
    const {body:b,resourceId:rid,id}=resource, y=byId.get(id).y;
    for(const [j,scope]of b.scopeTable.entries()) {
      const sid=`scope:${rid}:${scope.id}`;
      add({id:sid,label:scope.name,subtitle:'scopeId '+scope.id,kind:'scope',domain:'ifcdr',resourceId:rid,x:850+j*260,y,raw:scope});
      edge(id,sid,'scopeTable',true);
    }
    for(const [j,[streamName,kind]]of [['lineStream','line'],['polylineStream','polyline']].entries()) {
      const stream=b.streams[streamName]; if(!stream)continue;
      const cid=`collection:${rid}:${kind}`;
      add({id:cid,label:kind==='line'?'Lijnen':'Polylijnen',subtitle:stream.count+' entiteiten · '+streamName,kind:'collection',domain:'ifcdr',resourceId:rid,x:850,y:y+120+j*124,raw:stream});
      edge(id,cid,streamName,true);defaultCollapsed.add(cid);
      let entitySlot=0;
      paging.register(cid,stream.entityId,(_,row)=> {
        const eid=stream.entityId[row], entityKey=`entity:${rid}:${eid}`;
        const layer=b.layerBindings.find(l=>String(l.id)===String(stream.layerId[row]));
        const appearance=b.appearanceBindings.find(a=>String(a.id)===String(stream.appearanceId[row]));
        let points, geometry;
        if(kind==='line') {
          points=[[stream.x1[row],stream.y1[row],stream.z1?.[row]??0],[stream.x2[row],stream.y2[row],stream.z2?.[row]??0]];
          geometry={start:points[0],end:points[1]};
        } else {
          const start=Number(stream.vertexOffset[row]),count=Number(stream.vertexCount[row]),placement=stream.placement?.[row];
          const local=Array.from({length:count},(_,i)=>[stream.x[start+i],stream.y[start+i]]);
          points=local.map(([x,y])=>placement?['x','y','z'].map(k=>Number(placement.origin[k])+Number(x)*Number(placement.X[k])+Number(y)*Number(placement.Y[k])):[x,y,0]);
          geometry={vertices:local,closed:stream.closed[row],placement:placement||'identity'};
        }
        const entity={id:entityKey,entityId:eid,resourceId:rid,kind,points,closed:kind==='polyline'&&stream.closed[row],layer:'ifcx:'+layer.ifcxLayer,layerId:layer.id,appearanceId:appearance.id,appearance,scopeId:stream.scopeId[row],visible:stream.visible?.[row]??true,geometry,row,stream:streamName};
        entities.push(entity);
        add({id:entityKey,label:kind+' #'+eid,subtitle:'scope '+entity.scopeId+' · laag '+layer.id,kind:'entity',domain:'ifcdr',resourceId:rid,x:1120+j*260,y:y+120+entitySlot++*150,raw:{entityId:eid,scopeId:entity.scopeId,layerId:layer.id,appearanceId:appearance.id,...geometry},entity});
        edge(cid,entityKey,'rij '+row,true);
        edge(entityKey,entity.layer,'layerBinding');
        edge(`scope:${rid}:${entity.scopeId}`,entityKey,'scope');
        if(appearance.ifcxAppearance)edge(entityKey,'ifcx:'+appearance.ifcxAppearance,'appearanceBinding');
        addEntityFields([entity],fieldContext);
      },eid=>'entity:'+rid+':'+eid);
    }
  }
  for(const preservation of preservations) {
    const {body:b,resourceId:rid,id}=preservation;
    const register=(name,items,create,idFor)=>{
      if(items.length<=paging.pageSize){items.forEach(create);return;}
      const cid=`collection:${rid}:${name}`,parent=byId.get(id);
      add({id:cid,label:name,subtitle:items.length+' records',kind:'group',domain:'ifcpr',resourceId:rid,x:parent.x+270,y:parent.y+['records','projectionBindings','structuredAttachments'].indexOf(name)*170,raw:null});
      edge(id,cid,name,true);defaultCollapsed.add(cid);
      let slot=0;
      paging.register(cid,items,(item,i)=>{create(item,slot++);const key=idFor(item,i);const link=edges.find(e=>e.source===id&&e.target===key&&e.structural);if(link)link.source=cid;},idFor);
    };
    for(const target of b.linkedDrawingResources)edge(id,'resource:'+target,'linkedDrawingResources');
    register('records',b.records,(record,i)=> {
      const key=`record:${rid}:${record.recordId}`;
      add({id:key,label:'Record '+record.recordId,subtitle:record.sourceType,kind:'record',domain:'ifcpr',x:850,y:-220+i*112,raw:record,preservation});
      edge(id,key,'records',true);
    },record=>`record:${rid}:${record.recordId}`);
    register('projectionBindings',b.projectionBindings,(binding,i)=> {
      const key=`projection:${rid}:${binding.projectionId}`;
      add({id:key,label:'Projection '+binding.projectionId,subtitle:binding.targetKind+' · '+binding.coverage,kind:'projection',domain:'ifcpr',x:1120,y:-220+i*112,raw:binding,preservation});
      edge(id,key,'projectionBindings',true);
      for(const r of binding.sourceRecords)edge(`record:${rid}:${r.recordId}`,key,'sourceRecord');
      for(const t of binding.modelTargets)edge(key,binding.targetKind==='ifcdrEntity'?`entity:${t.resourceId}:${t.targetId}`:binding.targetKind==='ifcxNode'?'ifcx:'+t.targetId:'resource:'+t.targetId,'modelTarget');
    },binding=>`projection:${rid}:${binding.projectionId}`);
    for(const [i,blob]of b.blobs.entries()) {
      const key=`blob:${rid}:${blob.id}`;
      add({id:key,label:'Bronblob',subtitle:blob.byteLength+' bytes · '+blob.compression,kind:'blob',domain:'ifcpr',x:1410,y:-220+i*112,raw:blob,preservation});
      edge(id,key,'blobs',true);
      for(const r of b.records)if(r.payload?.blobId===blob.id)edge(`record:${rid}:${r.recordId}`,key,'payload · bytebereik');
    }
    for(const d of b.dependencyEdges)edge(`record:${rid}:${d.sourceRecordId}`,`record:${rid}:${d.targetRecordId}`,d.role+' · '+d.strength);
    register('structuredAttachments',b.structuredAttachments,(a,i)=> {
      const key=`attachment:${rid}:${a.attachmentId}`;
      add({id:key,label:a.kind,subtitle:'Attachment '+a.attachmentId,kind:'attachment',domain:'ifcpr',x:1410,y:4+i*112,raw:a,preservation});
      edge(id,key,'structuredAttachments',true);
      edge(`record:${rid}:${a.parentRecordId}`,key,'parentRecord');
      if(a.payload)edge(key,`blob:${rid}:${a.payload.blobId}`,'payload');
    },a=>`attachment:${rid}:${a.attachmentId}`);
  }
  if(concepts) {
    const rid=resources[0].resourceId, y=byId.get(resources[0].id).y;
    const circle={id:`entity:${rid}:5`,entityId:5,resourceId:rid,kind:'circle',concept:true,scopeId:0,layerId:0,layer:'ifcx:layer-0',appearanceId:0,visible:true,center:[19,5,0],radius:3,geometry:{center:[19,5],radius:3,plane:{origin:[0,0,0],X:[1,0,0],Y:[0,1,0]}},points:Array.from({length:65},(_,i)=>[19+3*Math.cos(i*Math.PI/32),5+3*Math.sin(i*Math.PI/32),0])};
    const dimension={id:`entity:${rid}:6`,entityId:6,resourceId:rid,kind:'dimension',concept:true,scopeId:0,layerId:1,layer:'ifcx:layer-a-wall',appearanceId:0,visible:true,points:[[0,-3,0],[10,-3,0]],geometry:{kind:'linear',definitionPoints:[[0,0,0],[10,0,0]],dimensionLinePoint:[0,-3,0],styleRef:'concept:dimension-style',textOverride:null}};
    for(const [i,e]of [circle,dimension].entries()) {
      const cid='concept:collection:'+e.kind, branchY=y+380+i*150;
      add({id:cid,label:e.kind==='circle'?'Circles':'Dimensions',subtitle:'Voorbeeld van een eigen collectie',family:e.kind,resourceId:rid,kind:'collection',domain:'ifcdr',concept:true,x:850,y:branchY,raw:null});
      edge(resources[0].id,cid,e.kind+' · concept',true,true);defaultCollapsed.add(cid);
      entities.push(e);add({id:e.id,label:e.kind+' #'+e.entityId,subtitle:'Illustratief logisch record',kind:'entity',domain:'ifcdr',concept:true,x:1120,y:branchY,raw:e.geometry,entity:e,resourceId:rid});edge(cid,e.id,'voorbeeldentiteit',true,true);edge(e.id,e.layer,'layerBinding',false,true);
      const recordId='concept:source-'+e.kind,projectionId='concept:projection-'+e.kind;
      add({id:recordId,label:'Bron-'+e.kind.toUpperCase(),subtitle:'Aanvullende brongegevens',kind:'concept-record',domain:'ifcpr',concept:true,x:1410,y:branchY,raw:{sourceType:e.kind.toUpperCase(),note:'Brongebonden extra velden; illustratief, geen IFCPR-record.'}});
      add({id:projectionId,label:'Conceptbinding',subtitle:e.kind+' · gedeeltelijke dekking',kind:'concept-projection',domain:'ifcpr',concept:true,x:1680,y:branchY,raw:{source:'bron-'+e.kind,target:{resourceId:rid,entityId:e.entityId},coverage:'partial'}});
      edge(preservations[0].id,recordId,'records · concept',true,true);
      edge(preservations[0].id,projectionId,'projectionBindings · concept',true,true);
      edge(recordId,projectionId,'sourceRecord',false,true);edge(projectionId,e.id,'modelTarget',false,true);
    }
    add({id:'concept:dimension-style',label:'DimensionStyle',subtitle:'Gedeelde maatstijl · concept',kind:'concept-style',domain:'ifcx',concept:true,x:310,y:definitionsY+360,raw:{note:'Mogelijke gedeelde stijlidentiteit; schema en veldnamen nog te ontwerpen.'}});
    roots.push('concept:dimension-style');edge(dimension.id,'concept:dimension-style','styleRef',false,true);
  }
  addEntityFields(entities.filter(e=>e.concept),fieldContext);
  for(const n of nodes)n.expandable=n.expandable||edges.some(e=>e.structural&&e.source===n.id);
  const missing=edges.filter(e=>!byId.has(e.source)||!byId.has(e.target));
  // Unassessed preservation/extension links may have no available target.
  for(const n of nodes.filter(n=>n.domain==='ifcx'&&!n.concept))if(!edges.some(e=>e.structural&&e.target===n.id)&&!roots.includes(n.id))roots.push(n.id);
  const reached=new Set(),adj=new Map();for(const e of edges)if(e.structural){if(!adj.has(e.source))adj.set(e.source,[]);adj.get(e.source).push(e.target);}
  const reach=id=>{const stack=[id];while(stack.length){const next=stack.pop();if(reached.has(next))continue;reached.add(next);stack.push(...(adj.get(next)||[]));}};
  roots.forEach(reach);for(const n of nodes.filter(n=>n.domain==='ifcx'))if(!reached.has(n.id)){roots.push(n.id);reach(n.id);}
  return {fixture,nodes,edges,byId,roots,defaultCollapsed,entities,resources,preservations,concepts,paging,missing};
}

export function visibleGraph(model, collapsed) {
  const visible=new Set(),adj=new Map();
  for(const e of model.edges)if(e.structural){if(!adj.has(e.source))adj.set(e.source,[]);adj.get(e.source).push(e.target);}
  const stack=[...model.roots];while(stack.length){const id=stack.pop();if(visible.has(id)||!model.byId.has(id)||!model.paging.isVisible(id))continue;visible.add(id);if(!collapsed.has(id))stack.push(...(adj.get(id)||[]));}
  return {nodes:model.nodes.filter(n=>visible.has(n.id)),edges:model.edges.filter(e=>visible.has(e.source)&&visible.has(e.target))};
}

export function revealNode(model, collapsed, id) {
  model.paging?.materialize(id);
  const queue=[...model.roots],parents=new Map(queue.map(id=>[id,null])),adj=new Map();
  for(const e of model.edges)if(e.structural){if(!adj.has(e.source))adj.set(e.source,[]);adj.get(e.source).push(e.target);}
  for(let i=0;i<queue.length;i++){const node=queue[i];if(node===id){let parent=parents.get(node);while(parent!==null){model.paging.materialize(parent);collapsed.delete(parent);parent=parents.get(parent);}return true;}for(const next of adj.get(node)||[])if(!parents.has(next)){parents.set(next,node);queue.push(next);}}
  return false;
}
