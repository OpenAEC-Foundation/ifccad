/** Read-only view model for curated repository examples, not a package validator. */
import { addEntityFields } from './fields.mjs';
import { createCollections } from './collections.mjs';
const commonEntityColumns=new Set(['count','entityId','scopeId','layerId','appearanceId','visible']);
const defaultPlane={origin:{x:0,y:0,z:0},X:{x:1,y:0,z:0},Y:{x:0,y:1,z:0}};
function storedEntityFields(stream,row){
  const pooled=Object.hasOwn(stream,'vertexOffset')&&Object.hasOwn(stream,'vertexCount');
  return Object.fromEntries(Object.entries(stream).filter(([name,values])=>
    !commonEntityColumns.has(name)&&Array.isArray(values)&&values.length===stream.count&&
    !(pooled&&['x','y','z'].includes(name))
  ).map(([name,values])=>[name,values[row]]));
}
export function buildModel(fixture, { concepts = false } = {}) {
  const nodes = [], edges = [], entities = [], resources = [], preservations = [];
  const byId = new Map(), defaultCollapsed = new Set(), roots = [];
  const add = n => { if (!byId.has(n.id)) { nodes.push(n); byId.set(n.id, n); } return byId.get(n.id); };
  const edge = (source, target, relation, structural = false, concept = false, compact = false) => edges.push({source,target,relation,structural,concept,compact});
  const paging=createCollections({add,edge,byId,defaultCollapsed});
  const fieldContext={add,edge,byId,defaultCollapsed,register:paging.register};
  const drawings = fixture.ifcx.data.filter(n => n.type === 'openaec:Drawing');
  const offsets=[];let drawingsHeight=0;
  for(const drawing of drawings){offsets.push(drawingsHeight);drawingsHeight+=Math.max(460,310+(drawing.children?.Layouts?.length||1)*150);}
  const definitionsY = 100 + drawingsHeight;
  let layerIndex=0, appearanceIndex=0;
  const pagedDefinitions=new Map(['Layer','Appearance'].map(kind=>[kind,fixture.ifcx.data.filter(n=>n.type==='openaec:'+kind)]).filter(([,items])=>items.length>paging.pageSize));
  for(const [kind,items]of pagedDefinitions){
    const id='view:definitions:'+kind;
    add({id,label:kind==='Layer'?'Lagen':'Appearances',subtitle:items.length+' definities',kind:'group',domain:'ifcx',x:kind==='Layer'?40:580,y:definitionsY,raw:null});
    const relation=kind==='Layer'?'Layers':'Appearances';
    // The display group can follow the sole Drawing only when all its items belong to that list.
    if(drawings.length===1&&items.every(raw=>drawings[0].children?.[relation]?.includes(raw.path)))edge('ifcx:'+drawings[0].path,id,relation,true,false,true);
    else roots.push(id);
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
    const drawingY=offsets[i]||0,layoutIndex=Math.max(0,drawings[i]?.children?.Layouts?.indexOf(raw.path)??0);
    const positions={DrawingSet:[40,50],Drawing:[40,215+drawingY],DrawingLayout:[40,365+drawingY+layoutIndex*150],DrawingRepresentation:[310,290+drawingY],PreservationRepresentation:[310,50],Layer:[40,definitionsY+layerIndex*150],Appearance:[310,definitionsY+appearanceIndex*150]};
    if(kind==='Layer')layerIndex++;
    if(kind==='Appearance')appearanceIndex++;
    const [x,y]=Object.hasOwn(positions,kind)?positions[kind]:[40,definitionsY+230];
    const n=add({id:'ifcx:'+raw.path,label:kind,subtitle:raw.attributes?.name||raw.path,kind,domain:'ifcx',x,y,raw});
    if(kind==='DrawingSet')roots.unshift(n.id);
    for(const [relation,children]of Object.entries(raw.children||{}))for(const path of Array.isArray(children)?children:[children]){
      const pagedMembership=kind==='Drawing'&&((relation==='Layers'&&pagedDefinitions.has('Layer'))||(relation==='Appearances'&&pagedDefinitions.has('Appearance')));
      edge(n.id,'ifcx:'+path,relation,!pagedMembership);
    }
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
      add({id,label:isDrawing?'IFCDR':'IFCPR',subtitle:rid,kind:isDrawing?'drawing-resource':'preservation-resource',domain:isDrawing?'ifcdr':'ifcpr',x:580,y:isDrawing?290+drawingY:50,raw:body,resourceId:rid,item});
      edge(n.id,id,descriptor.content?'content · inline':'uri · extern',true);
      defaultCollapsed.add(id);
    }
  }
  for(const raw of fixture.ifcx.data.filter(n=>n.type==='openaec:DrawingLayout'&&n.attributes?.scopeId!==undefined)){
    const representation=fixture.ifcx.data.find(n=>n.path===raw.children?.Representation);
    const descriptor=representation?.attributes?.resource;
    const rid=descriptor?.resourceId;
    if(rid&&resources.some(resource=>resource.resourceId===rid))edge('ifcx:'+raw.path,`scope:${rid}:${raw.attributes.scopeId}`,'scopeId');
  }
  for(const resource of resources) {
    const {body:b,resourceId:rid,id}=resource, y=byId.get(id).y;
    const table=(name,items,label,atY,idFor,create)=>{
      if(items.length<=paging.pageSize){items.forEach((item,i)=>create(item,i,id,false));return;}
      const groupId=`view:ifcdr:${rid}:${name}`;
      add({id:groupId,label,subtitle:items.length+' items',kind:'group',domain:'ifcdr',resourceId:rid,x:850,y:atY,raw:null});
      edge(id,groupId,name,true);defaultCollapsed.add(groupId);
      paging.register(groupId,items,(item,i)=>create(item,i,groupId,true),idFor);
    };
    table('scopeTable',b.scopeTable,'Scopes',y,scope=>`scope:${rid}:${scope.id}`,(scope,j,parent,compact)=>{
      const sid=`scope:${rid}:${scope.id}`;
      add({id:sid,label:['Model space','Paper space','Block definition scope'][scope.kind]??'Scope',subtitle:'scopeId '+scope.id,kind:'scope',domain:'ifcdr',resourceId:rid,x:850+j*260,y,raw:scope});
      edge(parent,sid,compact?'item':'scopeTable',true,false,compact);
    });
    const definitions=b.blockDefinitionTable||[];
    if(definitions.length>paging.pageSize)for(const definition of definitions)edge(`block-definition:${rid}:${definition.scopeId}`,`scope:${rid}:${definition.scopeId}`,'scopeId');
    table('blockDefinitionTable',definitions,'Blockdefinities',y-150,definition=>`block-definition:${rid}:${definition.scopeId}`,(definition,j,parent,compact)=>{
      const did=`block-definition:${rid}:${definition.scopeId}`;
      add({id:did,label:'BlockDefinition',subtitle:definition.name,kind:'block-definition',domain:'ifcdr',resourceId:rid,x:850+j*260,y:y-150,raw:definition});
      edge(parent,did,compact?'item':'blockDefinitionTable',true,false,compact);
      if(!compact)edge(did,`scope:${rid}:${definition.scopeId}`,'scopeId');
    });
    const objectStreams=b.streamDirectory.streams.filter(entry=>entry.role==='object'&&b.streams[entry.name+'Stream']?.count>0);
    for(const [j,entry]of objectStreams.entries()) {
      const kind=entry.name,streamName=kind+'Stream',stream=b.streams[streamName];
      const cid=`collection:${rid}:${kind}`;
      add({id:cid,label:{point:'Punten',line:'Lijnen',polyline:'Polylijnen',planarPolyline:'Vlakke polylijnen',spatialPolyline:'Ruimtelijke polylijnen',circle:'Cirkels',arc:'Bogen',ellipse:'Ellipsen',ellipseArc:'Ellipsbogen',blockInstance:'Block instances',viewport:'Viewports'}[kind]||kind,subtitle:stream.count+' entiteiten · '+streamName,kind:'collection',domain:'ifcdr',resourceId:rid,x:850,y:y+120+j*124,raw:stream});
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
        } else if(kind==='blockInstance') {
          points=[];
          geometry={definitionScopeId:stream.definitionScopeId[row],transform:stream.transform[row]};
        } else if(kind==='viewport') {
          points=[];
          const child=b.streams.viewportLayerOverrideStream;
          const offset=Number(stream.layerOverrideOffset[row]),count=Number(stream.layerOverrideCount[row]);
          const layerOverrides=Array.from({length:count},(_,i)=>({layerId:child.layerId[offset+i],frozen:child.frozen[offset+i],appearanceOverrideId:child.appearanceOverrideId[offset+i]}));
          geometry={viewScopeId:stream.viewScopeId[row],frame:stream.frame[row],view:stream.view[row],renderMode:stream.renderMode[row],viewEnabled:stream.viewEnabled[row],viewLocked:stream.viewLocked[row],paperClip:stream.paperClip[row],plotShadingOverride:stream.plotShadingOverride[row],layerOverrides};
        } else if(kind==='polyline'||kind==='planarPolyline') {
          const start=Number(stream.vertexOffset[row]),count=Number(stream.vertexCount[row]),placement=stream.placement?.[row];
          const local=Array.from({length:count},(_,i)=>[stream.x[start+i],stream.y[start+i]]);
          const frame=placement?{...defaultPlane,...placement}:defaultPlane;
          points=local.map(([x,y])=>['x','y','z'].map(k=>Number(frame.origin[k])+Number(x)*Number(frame.X[k])+Number(y)*Number(frame.Y[k])));
          geometry={vertices:local,closed:stream.closed[row],placement:placement||'identity'};
          if(stream.bulge)geometry.bulges=stream.bulge.slice(start,start+count);
        } else if(kind==='spatialPolyline') {
          const start=Number(stream.vertexOffset[row]),count=Number(stream.vertexCount[row]);
          points=Array.from({length:count},(_,i)=>[stream.x[start+i],stream.y[start+i],stream.z[start+i]]);
          geometry={vertices:points,closed:stream.closed[row]};
        } else {
          points=[];
          geometry=storedEntityFields(stream,row);
        }
        const entity={id:entityKey,entityId:eid,resourceId:rid,kind,points,closed:['polyline','planarPolyline','spatialPolyline'].includes(kind)&&stream.closed[row],layer:'ifcx:'+layer.ifcxLayer,layerId:layer.id,appearanceId:appearance.id,appearance,scopeId:stream.scopeId[row],visible:stream.visible?.[row]??true,geometry,row,stream:streamName};
        entities.push(entity);
        add({id:entityKey,label:kind+' #'+eid,subtitle:'scope '+entity.scopeId+' · laag '+layer.id,kind:'entity',domain:'ifcdr',resourceId:rid,x:1120+j*260,y:y+120+entitySlot++*150,raw:{entityId:eid,scopeId:entity.scopeId,layerId:layer.id,appearanceId:appearance.id,...geometry},entity});
        edge(cid,entityKey,'rij '+row,true);
        edge(entityKey,entity.layer,'layerBinding');
        edge(entityKey,`scope:${rid}:${entity.scopeId}`,'scopeId');
        if(kind==='blockInstance')edge(entityKey,`scope:${rid}:${geometry.definitionScopeId}`,'definitionScopeId');
        if(kind==='viewport'){
          edge(entityKey,`scope:${rid}:${geometry.viewScopeId}`,'viewScopeId');
          if(geometry.paperClip?.enabled&&geometry.paperClip.boundaryEntityId!==undefined)edge(entityKey,`entity:${rid}:${geometry.paperClip.boundaryEntityId}`,'paperClip.boundaryEntityId');
        }
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
    const dimension={id:`entity:${rid}:6`,entityId:6,resourceId:rid,kind:'dimension',concept:true,scopeId:0,layerId:1,layer:'ifcx:layer-a-wall',appearanceId:0,visible:true,points:[[0,-3,0],[10,-3,0]],geometry:{kind:'linear',definitionPoints:[[0,0,0],[10,0,0]],dimensionLinePoint:[0,-3,0],styleRef:'concept:dimension-style',textOverride:null}};
    const e=dimension,cid='concept:collection:dimension',branchY=y+380;
    add({id:cid,label:'Dimensions',subtitle:'Voorbeeld van een eigen collectie',family:e.kind,resourceId:rid,kind:'collection',domain:'ifcdr',concept:true,x:850,y:branchY,raw:null});
    edge(resources[0].id,cid,'dimension · concept',true,true);defaultCollapsed.add(cid);
    entities.push(e);add({id:e.id,label:e.kind+' #'+e.entityId,subtitle:'Illustratief logisch record',kind:'entity',domain:'ifcdr',concept:true,x:1120,y:branchY,raw:e.geometry,entity:e,resourceId:rid});edge(cid,e.id,'voorbeeldentiteit',true,true);edge(e.id,e.layer,'layerBinding',false,true);
    add({id:'concept:source-dimension',label:'Bron-DIMENSION',subtitle:'Aanvullende brongegevens',kind:'concept-record',domain:'ifcpr',concept:true,x:1410,y:branchY,raw:{sourceType:'DIMENSION',note:'Brongebonden extra velden; illustratief, geen IFCPR-record.'}});
    add({id:'concept:projection-dimension',label:'Conceptbinding',subtitle:'dimension · gedeeltelijke dekking',kind:'concept-projection',domain:'ifcpr',concept:true,x:1680,y:branchY,raw:{source:'bron-dimension',target:{resourceId:rid,entityId:e.entityId},coverage:'partial'}});
    edge(preservations[0].id,'concept:source-dimension','records · concept',true,true);
    edge(preservations[0].id,'concept:projection-dimension','projectionBindings · concept',true,true);
    edge('concept:source-dimension','concept:projection-dimension','sourceRecord',false,true);edge('concept:projection-dimension',e.id,'modelTarget',false,true);
    add({id:'concept:dimension-style',label:'DimensionStyle',subtitle:'Gedeelde maatstijl · concept',kind:'concept-style',domain:'ifcx',concept:true,x:310,y:definitionsY+360,raw:{note:'Mogelijke gedeelde stijlidentiteit; schema en veldnamen nog te ontwerpen.'}});
    roots.push('concept:dimension-style');edge(dimension.id,'concept:dimension-style','styleRef',false,true);
  }
  addEntityFields(entities.filter(e=>e.concept),fieldContext);
  for(const n of nodes)n.expandable=n.expandable||edges.some(e=>e.structural&&e.source===n.id);
  const missing=edges.filter(e=>(!byId.has(e.source)&&!paging.has(e.source))||(!byId.has(e.target)&&!paging.has(e.target)));
  // Unassessed preservation/extension links may have no available target.
  for(const n of nodes.filter(n=>n.domain==='ifcx'&&!n.concept))if(!edges.some(e=>e.structural&&e.target===n.id)&&!roots.includes(n.id))roots.push(n.id);
  const reached=new Set(),adj=new Map();for(const e of edges)if(e.structural){if(!adj.has(e.source))adj.set(e.source,[]);adj.get(e.source).push(e.target);}
  const reach=id=>{const stack=[id];while(stack.length){const next=stack.pop();if(reached.has(next))continue;reached.add(next);stack.push(...(adj.get(next)||[]));}};
  roots.forEach(reach);for(const n of nodes.filter(n=>n.domain==='ifcx'))if(!reached.has(n.id)){roots.push(n.id);reach(n.id);}
  return {fixture,nodes,edges,byId,roots,defaultCollapsed,entities,resources,preservations,concepts,paging,missing};
}

export function graphConnections(visible,selected,mode='focus',limit=8){
  if(mode==='all')return {edges:visible.edges,hiddenReferences:0};
  const kept=[],focused=[];
  for(const edge of visible.edges){
    if(edge.structural||edge.relation==='linkedDrawingResources')kept.push(edge);
    else if(edge.source===selected||edge.target===selected)focused.push(edge);
  }
  return {edges:[...kept,...focused.slice(0,limit)],hiddenReferences:Math.max(0,focused.length-limit)};
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
