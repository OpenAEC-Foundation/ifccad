import {parsePresentationJson} from './presentation-json.mjs';
const id=value=>String(value);
const numeric=(a,b)=>{try{return BigInt(a)<BigInt(b)?-1:BigInt(a)>BigInt(b)?1:0;}catch{return String(a).localeCompare(String(b));}};
export function createInspection(result,sourceText=''){
 const p=result?.presentation||{},format=p.format==='ifccad'||result?.inspectedFormat==='ifccad'||result?.source?.format==='ifccad'?'ifccad':'ocdraw',nodes=new Map(),roots=[],nodeRoots=[],storageRoots=[];
 let source;try{source=parsePresentationJson(sourceText);}catch{}
 const put=(key,title,values={},type='record')=>{const node={key,title,values,type,children:[],outgoing:[],incoming:[],fragments:[]};nodes.set(key,node);return node;};
 if(!result?.presentation){roots.push(put('file:source','file',source||{},'file').key);return {format,nodes,roots,nodeRoots:roots,storageRoots:roots,sourceText,valid:false};}
 const group=(key,title,children)=>{const n=put('group:'+key,title,{},'group');n.children=children;return n.key;};
 const link=(from,to,field,kind='reference')=>{if(!nodes.has(from)||typeof to!=='string')return;const edge={source:from,target:to,field,kind};nodes.get(from).outgoing.push(edge);nodes.get(to)?.incoming.push(edge);};
 const dimensionRefs=(from,value,field,ifccad)=>{if(!value)return;const target=(domain,v)=>v==null?null:ifccad?v:domain+':'+id(v);link(from,target('textStyle',value.text?.style),field+'.text.style','textStyle');for(const side of ['first','second']){const marker=value.arrows?.[side];if(marker?.kind==='localBlock')link(from,target('block',marker.block),field+'.arrows.'+side+'.block','marker');}for(const [name,component]of [['dimensionLine',value.dimensionLine],['extensionLines.first',value.extensionLines?.first],['extensionLines.second',value.extensionLines?.second],['text',value.text]]){const pattern=component?.appearance?.linePattern;if(pattern?.mode==='explicit')link(from,target('pattern',pattern.value),field+'.'+name+'.appearance.linePattern.value','pattern');}};
 if(format==='ifccad'){
  const graph=p.graph||{},all=graph.data||[];
  for(const item of all){const a=item.attributes||{},role=Object.keys(a).find(k=>k==='ifccad::entity')||Object.keys(a).find(k=>k.startsWith('ifccad::'));
   const payload=role?a[role]:{},geometry=Object.keys(a).find(k=>k.startsWith('ifccad::geom::')&&k!=='ifccad::geom::placement');
   const type=a['ifccad::opaqueEntity']?'opaque':a['ifccad::dimension']?'dimension':a['ifccad::hatch']?'hatch':a['ifccad::text']?'text':a['ifccad::mText']?'mText':a['ifccad::blockInstance']?'blockInstance':a['ifccad::viewport']?'viewport':geometry?geometry.split('::').at(-1):role?.split('::').at(-1)||'node';
   const title=payload?.name||payload?.fileName||payload?.kind||item.path.split('/').at(-1),n=put(item.path,title,a,type);n.raw=item;
   nodeRoots.push(n.key);
  }
  for(const [i,fragment] of (source?.data||[]).entries())nodes.get(fragment.path)?.fragments.push({index:i,value:fragment});
  for(const dimension of p.dimensions||[]){const n=nodes.get(dimension.path);if(n)n.derived=dimension.geometry||dimension;}
  for(const assessment of p.boundsCompleteness||[]){const n=nodes.get(assessment.scopePath);if(n)n.values={...n.values,boundsAssessment:assessment};}
  for(const item of all){const a=item.attributes||{},key=item.path,ordered=Object.entries(item.children||{}).sort(([x],[y])=>numeric(x,y));
   for(const [name,target]of ordered){link(key,target,'children.'+name,'child');if(/^\d+$/.test(name)&&(a['ifccad::layout']||a['ifccad::blockDefinition']))nodes.get(key).children.push(target);}
   for(const [name,target]of Object.entries(item.inherits||{}))if(typeof target==='string')link(key,target,'inherits.'+name,'inherits');
   if(a['ifccad::entity']){link(key,a['ifccad::entity'].layer,'ifccad::entity.layer','layer');const pattern=a['ifccad::entity'].appearance?.linePattern;if(pattern?.mode==='Explicit')link(key,pattern.value,'ifccad::entity.appearance.linePattern.value','pattern');}
   const opaque=a['ifccad::opaqueEntity'];if(opaque){link(key,opaque.preservationRecord,'ifccad::opaqueEntity.preservationRecord','preservation');link(key,opaque.nativeLayer,'ifccad::opaqueEntity.nativeLayer','layer');const pattern=opaque.nativeAppearance?.appearance?.linePattern;if(pattern?.mode==='Explicit')link(key,pattern.value,'ifccad::opaqueEntity.nativeAppearance.appearance.linePattern.value','pattern');}
   const record=a['ifccad::preservationRecord'];if(record){link(key,record.subject?.path,'ifccad::preservationRecord.subject','subject');for(const [i,b]of(record.bindings||[]).entries())link(key,b.target?.path,'ifccad::preservationRecord.bindings.'+i,'binding');for(const [i,c]of(record.conditions||[]).entries())link(key,c.target?.path,'ifccad::preservationRecord.conditions.'+i,'condition');}
   if(a['ifccad::preservation'])nodes.get(key).children=Object.entries(item.children||{}).sort(([x],[y])=>numeric(x.slice(1),y.slice(1))).map(([,path])=>path);
   link(key,a['ifccad::layer']?.appearance?.linePattern,'ifccad::layer.appearance.linePattern','pattern');
   link(key,a['ifccad::blockInstance']?.definition,'ifccad::blockInstance.definition','definition');
   for(const [i,loop]of(a['ifccad::hatch']?.loops||[]).entries())link(key,loop.source,'ifccad::hatch.loops.'+i+'.source','hatchSource');
   for(const kind of ['text','mText'])link(key,a['ifccad::'+kind]?.style,'ifccad::'+kind+'.style','style');
   const dimension=a['ifccad::dimension'];if(dimension){link(key,dimension.style,'ifccad::dimension.style','dimensionStyle');for(const [i,r]of(dimension.associations||[]).entries())link(key,r.source,'ifccad::dimension.associations.'+i+'.source','dimensionSource');dimensionRefs(key,dimension.overrides,'ifccad::dimension.overrides',true);}
   const dimensionStyle=a['ifccad::dimensionStyle'];if(dimensionStyle){dimensionRefs(key,dimensionStyle.settings,'ifccad::dimensionStyle.settings',true);for(const [kind,v]of Object.entries(dimensionStyle.typeOverrides||{}))dimensionRefs(key,v,'ifccad::dimensionStyle.typeOverrides.'+kind,true);}
   for(const [i,element] of (a['ifccad::linePattern']?.pattern||[]).entries()) {const m=element?.marker;if(m?.kind==='text')link(key,m.textStyle,'ifccad::linePattern.pattern.'+i+'.marker.textStyle','textStyle');if(m?.kind==='symbol')link(key,m.symbolSource,'ifccad::linePattern.pattern.'+i+'.marker.symbolSource','symbolSource');}
   const v=a['ifccad::viewport'];if(v){link(key,v.model,'ifccad::viewport.model','model');link(key,v.paperClip?.boundary,'ifccad::viewport.paperClip.boundary','clip');for(const [i,row]of(v.layerOverrides||[]).entries()){link(key,row.layer,'ifccad::viewport.layerOverrides.'+i+'.layer','layer');link(key,row.linePattern,'ifccad::viewport.layerOverrides.'+i+'.linePattern','pattern');}}
   if(a['ifccad::layout']||a['ifccad::blockDefinition'])nodes.get(key).children=Object.entries(item.children||{}).filter(([key])=>/^\d+$/.test(key)).sort(([x],[y])=>numeric(x,y)).map(([,target])=>target);
  }
  const role=r=>all.filter(n=>n.attributes?.['ifccad::'+r]).map(n=>n.path);
  const drawing=role('drawing');roots.push(...drawing);
  for(const [r,title]of [['layout','layouts'],['layer','layers'],['linePattern','linePatterns'],['symbolSource','symbolSources'],['blockDefinition','blocks'],['preservationRecord','preservation'],['ucsDefinition','ucsDefinitions'],['modelWindow','modelWindows']]){const children=role(r);if(r==='layout')children.sort((a,b)=>(nodes.get(a).values['ifccad::layout'].tabIndex||0)-(nodes.get(b).values['ifccad::layout'].tabIndex||0));if(children.length)roots.push(group(r,title,children));}
  const textStyles=role('textStyle');if(textStyles.length)roots.push(group('textStyle','textStyles',textStyles));
  const dimensionStyles=role('dimensionStyle');if(dimensionStyles.length)roots.push(group('dimensionStyle','dimensionStyles',dimensionStyles));
  const known=new Set(['drawing','layout','layer','linePattern','symbolSource','textStyle','dimensionStyle','blockDefinition','entity','opaqueEntity','preservation','preservationRecord','ucsDefinition','modelWindow'].flatMap(role));const others=all.filter(n=>!known.has(n.path)).map(n=>n.path);if(others.length)roots.push(group('other','otherNodes',others));
  for(const name of ['header','imports','schemas']){const n=put('file:'+name,name,graph[name]??source?.[name]??{},'file');roots.push(n.key);nodeRoots.push(n.key);}
 }else{
  const table=(name,domain,title)=>{const children=[];for(const v of p[name]||[]){const n=put(domain+':'+id(v.id??v.scopeId??v.viewportEntityId),v.name||v.fileName||id(v.id??v.scopeId??v.viewportEntityId),v,name);children.push(n.key);}return children.length?group(name,title,children):null;};
  roots.push(put('drawing','drawing',Object.fromEntries(Object.entries(p).filter(([k,v])=>!Array.isArray(v)&&!['streams','entities','sourceText'].includes(k))), 'drawing').key);
  for(const name of ['layers','linePatterns','symbolSources','textStyles','dimensionStyles','blockDefinitions','ucsDefinitions','modelWindows','paperCanvases','viewportWorkspaces']){const k=table(name,{layers:'layer',linePatterns:'pattern',symbolSources:'symbolSource',textStyles:'textStyle',dimensionStyles:'dimensionStyle',blockDefinitions:'block',ucsDefinitions:'ucs',modelWindows:'window',paperCanvases:'canvas',viewportWorkspaces:'workspace'}[name],name==='blockDefinitions'?'blocks':name);if(k)roots.push(k);}
  for(const style of p.dimensionStyles||[]){const key='dimensionStyle:'+id(style.id);dimensionRefs(key,style.settings,'settings',false);for(const [kind,v]of Object.entries(style.typeOverrides||{}))dimensionRefs(key,v,'typeOverrides.'+kind,false);}
  if(p.preservation){const records=(p.preservation.records||[]).map(v=>put('preservation:'+id(v.id),id(v.id),v,'preservationRecord').key);roots.push(group('preservation','preservation',records));}
  const entities=[...(p.entities||[]),...(p.opaqueEntities||[]).map(v=>({...v,type:'opaque'}))];
  for(const v of entities){const n=put('entity:'+id(v.id),id(v.id),v,v.geometry?.type||v.type||'entity');if(v.layerId!=null)link(n.key,'layer:'+id(v.layerId),'layerId','layer');if(v.geometry?.type==='hatch')for(const [i,loop]of(v.geometry.loops||[]).entries())if(loop.sourceEntityId!=null)link(n.key,'entity:'+id(loop.sourceEntityId),'geometry.loops.'+i+'.sourceEntityId','hatchSource');if(v.geometry?.type==='dimension'){if(v.styleId!=null)link(n.key,'dimensionStyle:'+id(v.styleId),'styleId','dimensionStyle');for(const [i,r]of(v.geometry.associations||[]).entries())link(n.key,'entity:'+id(r.sourceEntityId),'geometry.associations.'+i+'.sourceEntityId','dimensionSource');dimensionRefs(n.key,v.geometry.overrides,'geometry.overrides',false);}else if(v.styleId!=null)link(n.key,'textStyle:'+id(v.styleId),'styleId','textStyle');if(v.preservationRecordId!=null)link(n.key,'preservation:'+id(v.preservationRecordId),'preservationRecordId','preservation');if(v.geometry?.definitionScopeId!==undefined)link(n.key,'block:'+id(v.geometry.definitionScopeId),'geometry.definitionScopeId','definition');const pat=v.appearance?.linePattern;if(pat?.mode==='Explicit')link(n.key,'pattern:'+id(pat.value),'appearance.linePattern.value','pattern');nodeRoots.push(n.key);}
  for(const ptn of p.linePatterns||[])for(const [i,element] of (ptn.pattern||[]).entries()){const m=element?.marker;if(m?.kind==='text')link('pattern:'+id(ptn.id),'textStyle:'+id(m.textStyleId),'pattern.'+i+'.marker.textStyleId','textStyle');if(m?.kind==='symbol')link('pattern:'+id(ptn.id),'symbolSource:'+id(m.symbolSourceId),'pattern.'+i+'.marker.symbolSourceId','symbolSource');}
  const assessments=new Map((p.boundsCompleteness||[]).map(s=>[id(s.scopeId),s]));
  const scopes=new Map((p.scopes||[]).map(s=>[id(s.id),s]));for(const layout of [...(p.layouts||[])].sort((a,b)=>a.tabIndex-b.tabIndex)){const assessment=assessments.get(id(layout.scopeId)),n=put('layout:'+id(layout.id),layout.name,assessment?{...layout,boundsAssessment:assessment}:layout,'layout');n.children=(scopes.get(id(layout.scopeId))?.entities||[]).map(e=>'entity:'+id(e));for(const target of n.children)link(n.key,target,'scope.entities','owner');roots.push(n.key);}
  for(const n of nodes.values())if(n.type==='blockDefinitions'){const assessment=assessments.get(id(n.values.scopeId));if(assessment)n.values={...n.values,boundsAssessment:assessment};n.children=(scopes.get(id(n.values.scopeId))?.entities||[]).map(e=>'entity:'+id(e));for(const target of n.children)link(n.key,target,'scope.entities','owner');}
  for(const [name,stream]of Object.entries(p.streams||{})){const s=put('stream:'+name,name,stream,'stream');storageRoots.push(s.key);for(const [row,value]of (stream.entityId||[]).entries()){const n=nodes.get('entity:'+id(value));if(n){n.storage={stream:name,row,...(stream.vertexOffset?{offset:stream.vertexOffset[row],count:stream.vertexCount[row]}:{})};s.children.push(n.key);}}}
  for(const name of ['layers','layouts','scopes','blockDefinitions','linePatterns','symbolSources','textStyles','dimensionStyles','ucsDefinitions','viewState','modelWindows','paperCanvases','drawingWorkspaceState','viewportWorkspaces'])if(p[name]!=null)storageRoots.push(put('table:'+name,name,p[name],'table').key);
  for(const n of nodes.values())if(n.type==='viewport'){for(const [i,row]of(n.values.layerOverrides||[]).entries()){link(n.key,'layer:'+id(row.layerId),'layerOverrides.'+i+'.layerId','layer');if(row.linePatternId!=null)link(n.key,'pattern:'+id(row.linePatternId),'layerOverrides.'+i+'.linePatternId','pattern');}}
  const layouts=roots.filter(key=>nodes.get(key)?.type==='layout');for(const key of layouts)roots.splice(roots.indexOf(key),1);if(layouts.length)roots.splice(1,0,group('layouts','layouts',layouts));
 }
 const drawingUnit=p.unit||(p.graph?.data||[]).find(n=>n.attributes?.['ifccad::drawing'])?.attributes['ifccad::drawing'].lengthUnit;
 const namedUcs=(from,choice,field)=>{if(choice?.kind==='Named')link(from,format==='ifccad'?choice.ucs:'ucs:'+id(choice.ucsId),field,'ucs');};
 for(const n of nodes.values()){
  if(format==='ifccad'){
   const a=n.values,model=a['ifccad::modelWindow'],canvas=a['ifccad::paperCanvas'],viewport=a['ifccad::viewportWorkspace'];
   const snapshot=model||canvas||viewport;
   if(snapshot){n.workspace={coordinateDomain:canvas?'Paper':'Model',unit:canvas?null:drawingUnit,useStoredUcs:snapshot.useStoredUcs??true};namedUcs(n.key,snapshot.storedUcs,(model?'ifccad::modelWindow':canvas?'ifccad::paperCanvas':'ifccad::viewportWorkspace')+'.storedUcs');}
   if(canvas){n.workspaceChoices={currentUcs:canvas.currentUcs??null,activeContext:canvas.activeContext??null};namedUcs(n.key,canvas.currentUcs,'ifccad::paperCanvas.currentUcs');if(canvas.activeContext?.kind==='Viewport')link(n.key,canvas.activeContext.viewport,'ifccad::paperCanvas.activeContext','viewport');}
   if(a['ifccad::drawing']){
    const current=a['ifccad::modelViewState'];
    if(current||p.modelWindows?.length||(p.graph?.data||[]).some(v=>v.attributes?.['ifccad::modelWindow']))n.workspaceChoices={currentUcs:current?.currentModelUcs??null,activeModelWindow:current?.activeModelWindow??null};
    namedUcs(n.key,current?.currentModelUcs,'ifccad::modelViewState.currentModelUcs');link(n.key,current?.activeModelWindow,'ifccad::modelViewState.activeModelWindow','window');
    for(const target of a['ifccad::drawing'].modelWindows||[])link(n.key,target,'ifccad::drawing.modelWindows','window');
    link(n.key,a['ifccad::drawingWorkspace']?.currentLayer,'ifccad::drawingWorkspace.currentLayer','layer');link(n.key,a['ifccad::drawingWorkspace']?.activeLayout,'ifccad::drawingWorkspace.activeLayout','layout');
   }
  }else{
   const snapshot=['modelWindows','paperCanvases','viewportWorkspaces'].includes(n.type);
   if(snapshot){const canvas=n.type==='paperCanvases';n.workspace={coordinateDomain:canvas?'Paper':'Model',unit:canvas?null:drawingUnit,useStoredUcs:n.values.useStoredUcs??true};namedUcs(n.key,n.values.storedUcs,'storedUcs');}
   if(n.type==='viewportWorkspaces')link(n.key,'entity:'+id(n.values.viewportEntityId),'viewportEntityId','viewport');
   if(n.type==='paperCanvases'){n.workspaceChoices={currentUcs:n.values.currentUcs??null,activeContext:n.values.activeContext??null};namedUcs(n.key,n.values.currentUcs,'currentUcs');if(n.values.activeContext?.kind==='Viewport')link(n.key,'entity:'+id(n.values.activeContext.viewportEntityId),'activeContext','viewport');}
   if(n.key==='drawing'&&(p.viewState||p.modelWindows?.length)){n.workspaceChoices={currentUcs:p.viewState?.currentModelUcs??null,activeModelWindow:p.viewState?.activeModelWindowId==null?null:'window:'+id(p.viewState.activeModelWindowId)};namedUcs(n.key,p.viewState?.currentModelUcs,'viewState.currentModelUcs');if(p.viewState?.activeModelWindowId!=null)link(n.key,'window:'+id(p.viewState.activeModelWindowId),'viewState.activeModelWindowId','window');}
  }
 }

 for(const n of nodes.values())if(['layout','blockDefinition','blockDefinitions'].includes(n.type)){
  const layout=n.values['ifccad::layout']||n.values;
  n.unit=n.type==='layout'&&String(layout.kind).toLowerCase()==='paper'?null:drawingUnit;
  const kinds=new Map();n.children.forEach((key,index)=>{const child=nodes.get(key);if(!child)return;child.owner=n.key;child.drawPosition=index;child.unit=n.unit;const kind=child.type;kinds.set(kind,[...(kinds.get(kind)||[]),key]);});
  n.groupedChildren=Array.from(kinds,([kind,keys])=>{const key=group('types:'+n.key+':'+kind,'plural.'+kind,keys);nodes.get(key).unit=n.unit;return key;});
 }
 for(const n of nodes.values())if(n.type==='dimension'){
  n.sourceValues=n.values;
  if(format==='ocdraw'){
   const geometry={...n.values.geometry};n.derived={};
   for(const key of ['rawMeasurement','label','effectiveSettings','associationUpdate','displayEvaluation'])if(key in geometry){n.derived[key]=geometry[key];delete geometry[key];}
   n.values={...n.values,geometry};
  }
  if(n.derived?.effectiveSettings)dimensionRefs(n.key,n.derived.effectiveSettings,'derived.effectiveSettings',format==='ifccad');
 }
 for(const n of nodes.values())n.incoming=[];
 for(const n of nodes.values())for(const edge of n.outgoing)nodes.get(edge.target)?.incoming.push(edge);
 return {format,nodes,roots,nodeRoots,storageRoots,sourceText,valid:result?.validation?.strictAvailable===true};
}
