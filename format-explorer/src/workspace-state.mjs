/** Presentation of optional IFCDR 0.12 workspace data; package validation stays with Rust. */
import {fieldSummary,isVector} from './fields.mjs';

function addWorkspaceFields(owner,{add,edge,byId,defaultCollapsed,register}){
 defaultCollapsed.add(owner.id);
 function visit(parentId,key,value,path,index){
  const parent=byId.get(parentId),id=`field:${owner.id}:${path.join('.')}`;
  add({id,label:/^\d+$/.test(key)?`item ${key}`:key,subtitle:fieldSummary(value),kind:'field',domain:'ifcdr',
   ownerId:owner.id,parentId,fieldPath:path,fieldIndex:index,resourceId:owner.resourceId,
   x:parent.x+270,y:parent.y+index*140,raw:value});
  edge(parentId,id,key,true);
  parent.expandable=true;
  if(value&&typeof value==='object'&&!isVector(value)){
   defaultCollapsed.add(id);
   const entries=Object.entries(value),create=([child,childValue],i)=>visit(id,child,childValue,[...path,child],i);
   register(id,entries,create,([child])=>`field:${owner.id}:${[...path,child].join('.')}`);
  }
 }
 Object.entries(owner.raw).forEach(([key,value],i)=>visit(owner.id,key,value,[key],i));
}

function ucsLabel(selection,body){
 if(!selection)return '—';
 if(selection.kind===0||selection.kind==='World')return 'Wereld';
 if(selection.kind===2||selection.kind==='Unnamed')return 'Naamloos · inline frame';
 if(selection.kind===1||selection.kind==='Named'){
  const definition=body.ucsDefinitionTable?.find(row=>String(row.ucsId)===String(selection.ucsId));
  return `Benoemd · ${definition?.name||'#'+selection.ucsId} (#${selection.ucsId})`;
 }
 return '—';
}
const flag=value=>value===undefined?'—':value?'aan':'uit';
export function workspaceSummary(node,body){
 const row=node.raw;
 switch(node.kind){
  case 'drawing-view-state':return [['Actief modelvenster','#'+row.activeModelWindowId],['Huidige model-UCS',ucsLabel(row.currentModelUcs,body)]];
  case 'ucs-definition':return [['UCS-ID',row.ucsId],['Naam',row.name],['Hoogte',row.elevation]];
  case 'model-window':return [['Venster-ID',row.modelWindowId],['Opgeslagen UCS',ucsLabel(row.storedUcs,body)],['UCS toepassen',flag(row.useStoredUcs)],['Grid',flag(row.grid?.enabled)],['Snap',flag(row.snap?.enabled)]];
  case 'paper-canvas':return [['Scope-ID',row.scopeId],['Actieve context',row.activeContext?.kind===1||row.activeContext?.kind==='Viewport'?`Viewport #${row.activeContext.viewportEntityId}`:'Canvas'],['Huidige UCS',ucsLabel(row.currentUcs,body)],['Opgeslagen UCS',ucsLabel(row.storedUcs,body)],['Grid',flag(row.grid?.enabled)],['Snap',flag(row.snap?.enabled)]];
  case 'viewport-workspace':return [['Viewport-entiteit','#'+row.viewportEntityId],['Opgeslagen UCS',ucsLabel(row.storedUcs,body)],['UCS toepassen',flag(row.useStoredUcs)],['Grid',flag(row.grid?.enabled)],['Snap',flag(row.snap?.enabled)]];
  default:return [];
 }
}
export function addWorkspaceState(resource,{add,edge,byId,paging,defaultCollapsed}){
 const {body,resourceId:rid,id}=resource;
 const tables=[
  ['ucsDefinitionTable','UCS-definities','ucs-definition',row=>`workspace:${rid}:ucs:${row.ucsId}`,row=>['UCS-definitie',row.name]],
  ['modelWindowTable','Modelvensters','model-window',row=>`workspace:${rid}:model-window:${row.modelWindowId}`,row=>['Modelvenster','ID '+row.modelWindowId]],
  ['paperCanvasTable','Paperspace-canvassen','paper-canvas',row=>`workspace:${rid}:paper-canvas:${row.scopeId}`,row=>['Paperspace-canvas','scope '+row.scopeId]],
  ['viewportWorkspaceTable','Viewportstatus','viewport-workspace',row=>`workspace:${rid}:viewport:${row.viewportEntityId}`,row=>['Viewportstatus','viewport #'+row.viewportEntityId]],
 ];
 if(!body.drawingViewState&&!tables.some(([name])=>body[name]?.length))return;
 const baseY=resource.y-220,group=`view:ifcdr:${rid}:workspace`;
 add({id:group,label:'Werkruimtestatus',subtitle:'UCS · vensters · canvas',kind:'workspace-group',domain:'ifcdr',resourceId:rid,x:850,y:baseY,raw:null});
 edge(id,group,'workspace state',true);
 defaultCollapsed.add(group);
 const namedUcs=(source,selection,field)=>{
  if((selection?.kind===1||selection?.kind==='Named')&&selection.ucsId!==undefined)
   edge(source,`workspace:${rid}:ucs:${selection.ucsId}`,field+'.ucsId');
 };
 if(body.drawingViewState){
  const state=body.drawingViewState,key=`workspace:${rid}:drawing-view-state`;
  const node=add({id:key,label:'Tekenwerkruimte',subtitle:'actief modelvenster '+state.activeModelWindowId,kind:'drawing-view-state',domain:'ifcdr',resourceId:rid,x:1120,y:baseY,raw:state});
  edge(group,key,'drawingViewState',true);
  addWorkspaceFields(node,{add,edge,byId,defaultCollapsed,register:paging.register});
  edge(key,`workspace:${rid}:model-window:${state.activeModelWindowId}`,'activeModelWindowId');
  namedUcs(key,state.currentModelUcs,'currentModelUcs');
 }
 tables.forEach(([name,label,kind,idFor,text],index)=>{
  const rows=body[name]||[];
  if(!rows.length)return;
  const table=`view:ifcdr:${rid}:${name}`;
  add({id:table,label,subtitle:rows.length+' '+(rows.length===1?'item':'items'),kind:'group',domain:'ifcdr',resourceId:rid,x:1120,y:baseY+150+index*150,raw:null});
  edge(group,table,name,true);
  defaultCollapsed.add(table);
  paging.register(table,rows,(row,i)=>{
   const key=idFor(row),[title,subtitle]=text(row);
   const node=add({id:key,label:title,subtitle,kind,domain:'ifcdr',resourceId:rid,x:1390,y:baseY+i*140,raw:row});
   edge(table,key,'rij '+i,true);
   addWorkspaceFields(node,{add,edge,byId,defaultCollapsed,register:paging.register});
   if(kind==='model-window')namedUcs(key,row.storedUcs,'storedUcs');
   if(kind==='paper-canvas'){
    edge(key,`scope:${rid}:${row.scopeId}`,'scopeId');
    namedUcs(key,row.storedUcs,'storedUcs');
    namedUcs(key,row.currentUcs,'currentUcs');
    if((row.activeContext?.kind===1||row.activeContext?.kind==='Viewport')&&row.activeContext.viewportEntityId!==undefined)
     edge(key,`entity:${rid}:${row.activeContext.viewportEntityId}`,'activeContext.viewportEntityId');
   }
   if(kind==='viewport-workspace'){
    edge(key,`entity:${rid}:${row.viewportEntityId}`,'viewportEntityId');
    namedUcs(key,row.storedUcs,'storedUcs');
   }
  },idFor);
 });
}
