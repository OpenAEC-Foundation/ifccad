import {parsePresentationJson} from './presentation-json.mjs';
import {createFontSymbolReport} from './font-symbol-sources.mjs';
const id=value=>String(value);
const numeric=(a,b)=>{try{return BigInt(a)<BigInt(b)?-1:BigInt(a)>BigInt(b)?1:0;}catch{return String(a).localeCompare(String(b));}};
const leaderRoles=['leader','multiLeader','tolerance','multiLeaderPresentation','leaderGraphics','leaderGraphicsCommand','multiLeaderBranches','multiLeaderBranch','multiLeaderLine','multiLeaderContent','multiLeaderText','multiLeaderBlock','multiLeaderTolerance','multiLeaderBlockLabels','multiLeaderBlockLabel'];
export function createInspection(result,sourceText=''){
 const p=result?.presentation||{},format='ifccad',nodes=new Map(),roots=[],nodeRoots=[],storageRoots=[];
 let source;try{source=parsePresentationJson(sourceText);}catch{}
 const put=(key,title,values={},type='record')=>{const node={key,title,values,type,children:[],outgoing:[],incoming:[],fragments:[]};nodes.set(key,node);return node;};
 if(!result?.presentation){roots.push(put('file:source','file',source||{},'file').key);return {format,nodes,roots,nodeRoots:roots,storageRoots:roots,sourceText,valid:false};}
 const group=(key,title,children)=>{const n=put('group:'+key,title,{},'group');n.children=children;return n.key;};
 const link=(from,to,field,kind='reference')=>{if(!nodes.has(from)||typeof to!=='string')return;const edge={source:from,target:to,field,kind};nodes.get(from).outgoing.push(edge);nodes.get(to)?.incoming.push(edge);};
 const dimensionRefs=(from,value,field,ifccad)=>{if(!value)return;const target=(domain,v)=>v==null?null:ifccad?v:domain+':'+id(v);link(from,target('textStyle',value.text?.style),field+'.text.style','textStyle');for(const side of ['first','second']){const marker=value.arrows?.[side];if(marker?.kind==='localBlock')link(from,target('block',marker.block),field+'.arrows.'+side+'.block','marker');}for(const [name,component]of [['dimensionLine',value.dimensionLine],['extensionLines.first',value.extensionLines?.first],['extensionLines.second',value.extensionLines?.second],['text',value.text]]){const pattern=component?.appearance?.linePattern;if(pattern?.mode==='explicit')link(from,target('pattern',pattern.value),field+'.'+name+'.appearance.linePattern.value','pattern');}};

  const graph=p.graph||{},all=graph.data||[];
  for(const item of all){const a=item.attributes||{},role=Object.keys(a).find(k=>k==='ifccad::entity')||Object.keys(a).find(k=>k.startsWith('ifccad::'));
   const payload=role?a[role]:{},geometry=Object.keys(a).find(k=>k.startsWith('ifccad::geom::')&&k!=='ifccad::geom::placement');
   const type=leaderRoles.find(role=>a['ifccad::'+role])||(a['ifccad::attributeDefinition']?'attributeDefinition':a['ifccad::blockAttribute']?'blockAttribute':a['ifccad::opaqueEntity']?'opaque':a['ifccad::dimension']?'dimension':a['ifccad::hatch']?'hatch':a['ifccad::text']?'text':a['ifccad::mText']?'mText':a['ifccad::blockInstance']?'blockInstance':a['ifccad::viewport']?'viewport':geometry?geometry.split('::').at(-1):role?.split('::').at(-1)||'node');
   const title=a['ifccad::blockAttribute']?.tag||a['ifccad::attributeDefinition']?.tag||payload?.name||payload?.fileName||payload?.kind||item.path.split('/').at(-1),n=put(item.path,title,a,type);n.raw=item;
   nodeRoots.push(n.key);
  }
  for(const [i,fragment] of (source?.data||[]).entries())nodes.get(fragment.path)?.fragments.push({index:i,value:fragment});
  for(const dimension of p.dimensions||[]){const n=nodes.get(dimension.path);if(n)n.derived=dimension.geometry||dimension;}
  for(const leader of p.leaders||[]){const n=nodes.get(leader.path);if(n)n.derived=leader;}
  for(const assessment of p.boundsCompleteness||[]){const n=nodes.get(assessment.scopePath);if(n)n.values={...n.values,boundsAssessment:assessment};}
  for(const item of all){const a=item.attributes||{},key=item.path,ordered=Object.entries(item.children||{}).sort(([x],[y])=>numeric(x,y));
   for(const [name,target]of ordered){link(key,target,'children.'+name,'child');if(/^\d+$/.test(name)&&(a['ifccad::layout']||a['ifccad::blockDefinition']||a['ifccad::blockInstance']))nodes.get(key).children.push(target);}
   if(leaderRoles.some(role=>a['ifccad::'+role]))nodes.get(key).children=ordered.map(([,path])=>path);
   const leader=a['ifccad::leader']||a['ifccad::tolerance']||a['ifccad::multiLeaderTolerance'];if(leader){link(key,leader.annotationEntity,'leader.annotationEntity','annotationTarget');link(key,leader.dimensionStyle,'leader.dimensionStyle','dimensionStyle');if(leader.arrow?.kind==='symbol')link(key,leader.arrow.definition,'leader.arrow.definition','marker');dimensionRefs(key,leader.overrides,'leader.overrides',true);}
   const leaderLine=(from,line,field)=>{if(!line)return;if(line.linePattern?.mode==='Explicit')link(from,line.linePattern.value,field+'.linePattern.value','pattern');if(line.arrow?.kind==='symbol')link(from,line.arrow.definition,field+'.arrow.definition','marker');};
   const common=a['ifccad::multiLeader']?.settings?.common||a['ifccad::multiLeaderStyle']?.common;if(common){leaderLine(key,common.line,'leader.line');link(key,common.text?.style,'leader.text.style','textStyle');link(key,common.block?.definition,'leader.block.definition','definition');}
   link(key,a['ifccad::multiLeader']?.style,'multiLeader.style','multiLeaderStyle');leaderLine(key,a['ifccad::multiLeaderLine']?.settings,'multiLeaderLine.settings');
   link(key,a['ifccad::multiLeaderBlock']?.definition,'multiLeaderBlock.definition','definition');link(key,a['ifccad::multiLeaderBlockLabel']?.definition,'multiLeaderBlockLabel.definition','attributeDefinition');link(key,a['ifccad::multiLeaderText']?.text?.style,'multiLeaderText.text.style','textStyle');
   for(const [name,target]of Object.entries(item.inherits||{}))if(typeof target==='string')link(key,target,'inherits.'+name,'inherits');
   if(a['ifccad::entity']){link(key,a['ifccad::entity'].layer,'ifccad::entity.layer','layer');const pattern=a['ifccad::entity'].appearance?.linePattern;if(pattern?.mode==='Explicit')link(key,pattern.value,'ifccad::entity.appearance.linePattern.value','pattern');}
   const part=a['ifccad::dimensionGraphicsPart'];if(part){link(key,part.layer,'ifccad::dimensionGraphicsPart.layer','layer');const pattern=part.appearance?.linePattern;if(pattern?.mode==='Explicit')link(key,pattern.value,'ifccad::dimensionGraphicsPart.appearance.linePattern.value','pattern');}
   if(a['ifccad::dimension']&&item.children?.graphics){nodes.get(key).children=[item.children.graphics];nodes.get(item.children.graphics).owner=key;}
   if(a['ifccad::dimensionGraphics']){const n=nodes.get(key);n.children=ordered.filter(([name])=>/^\d+$/.test(name)).map(([,path])=>path);for(const child of n.children){nodes.get(child).owner=key.slice(0,-'/graphics'.length);nodes.get(child).partIndex=child.split('/p').at(-1);}}
   const opaque=a['ifccad::opaqueEntity'];if(opaque){link(key,opaque.preservationRecord,'ifccad::opaqueEntity.preservationRecord','preservation');link(key,opaque.nativeLayer,'ifccad::opaqueEntity.nativeLayer','layer');const pattern=opaque.nativeAppearance?.appearance?.linePattern;if(pattern?.mode==='Explicit')link(key,pattern.value,'ifccad::opaqueEntity.nativeAppearance.appearance.linePattern.value','pattern');}
   const record=a['ifccad::preservationRecord'];if(record){link(key,record.subject?.path,'ifccad::preservationRecord.subject','subject');for(const [i,b]of(record.bindings||[]).entries())link(key,b.target?.path,'ifccad::preservationRecord.bindings.'+i,'binding');for(const [i,c]of(record.conditions||[]).entries())link(key,c.target?.path,'ifccad::preservationRecord.conditions.'+i,'condition');}
   if(a['ifccad::preservation'])nodes.get(key).children=Object.entries(item.children||{}).sort(([x],[y])=>numeric(x.slice(1),y.slice(1))).map(([,path])=>path);
   link(key,a['ifccad::layer']?.appearance?.linePattern,'ifccad::layer.appearance.linePattern','pattern');
   link(key,a['ifccad::blockInstance']?.definition,'ifccad::blockInstance.definition','definition');
   for(const [i,loop]of(a['ifccad::hatch']?.loops||[]).entries())for(const [j,source]of(loop.sources||[]).entries())link(key,source,'ifccad::hatch.loops.'+i+'.sources.'+j,'hatchSource');
   for(const role of ['ifccad::attributeDefinition','ifccad::blockAttribute']){const value=a[role];if(!value)continue;link(key,value.definition,role+'.definition','attributeDefinition');for(const [i,p]of(value.secondaryAttributes||[]).entries())link(key,p,role+'.secondaryAttributes.'+i,'secondaryAttribute');link(key,value.text?.style,role+'.text.style','style');link(key,value.mText?.text?.style,role+'.mText.text.style','style');link(key,value.mText?.layer,role+'.mText.layer','layer');const visit=(v,path)=>{if(!v||typeof v!=='object')return;if(v.kind==='Object')link(key,v.value,path+'.value','fieldObject');for(const [k,c]of Object.entries(v))visit(c,path+'.'+k);};visit(value.fields,role+'.fields');const annotation=value.annotation;link(key,annotation?.reference?.scale,role+'.annotation.reference.scale','annotationScale');for(const [i,c]of(annotation?.contexts||[]).entries()){link(key,c.key?.scale,role+'.annotation.contexts.'+i+'.key.scale','annotationScale');link(key,c.presentation?.embedded?.scaleId,role+'.annotation.contexts.'+i+'.presentation.embedded.scaleId','annotationScale');}}
   for(const kind of ['text','mText'])link(key,a['ifccad::'+kind]?.style,'ifccad::'+kind+'.style','style');
   const annotation=a['ifccad::annotation'];
   if(annotation){link(key,annotation.reference?.scaleId,'ifccad::annotation.reference.scaleId','annotationScale');for(const [i,c]of(annotation.contexts||[]).entries())link(key,c.key?.scaleId,`ifccad::annotation.contexts.${i}.key.scaleId`,'annotationScale');}
   for(const role of ['layout','viewport','annotationView']){const v=a['ifccad::'+role];link(key,v?.annotationState?.currentScale,role+'.annotationState.currentScale','annotationScale');link(key,v?.annotationScale,role+'.annotationScale','annotationScale');}
   for(const [i,c]of(a['ifccad::hatchAnnotationViews']?.contexts||[]).entries()){link(key,c.scaleId,`hatchAnnotationViews.contexts.${i}.scaleId`,'annotationScale');link(key,c.target?.entityId??c.target?.id,`hatchAnnotationViews.contexts.${i}.target`,'annotationView');}
   const dimension=a['ifccad::dimension'];if(dimension){link(key,dimension.style,'ifccad::dimension.style','dimensionStyle');for(const [i,r]of(dimension.associations||[]).entries())link(key,r.source,'ifccad::dimension.associations.'+i+'.source','dimensionSource');dimensionRefs(key,dimension.overrides,'ifccad::dimension.overrides',true);}
   const dimensionStyle=a['ifccad::dimensionStyle'];if(dimensionStyle){dimensionRefs(key,dimensionStyle.settings,'ifccad::dimensionStyle.settings',true);for(const [kind,v]of Object.entries(dimensionStyle.typeOverrides||{}))dimensionRefs(key,v,'ifccad::dimensionStyle.typeOverrides.'+kind,true);}
   for(const [i,element] of (a['ifccad::linePattern']?.pattern||[]).entries()) {const m=element?.marker;if(m?.kind==='text')link(key,m.textStyle,'ifccad::linePattern.pattern.'+i+'.marker.textStyle','textStyle');if(m?.kind==='symbol')link(key,m.symbolSource,'ifccad::linePattern.pattern.'+i+'.marker.symbolSource','symbolSource');}
   const v=a['ifccad::viewport'];if(v){link(key,v.model,'ifccad::viewport.model','model');link(key,v.paperClip?.boundary,'ifccad::viewport.paperClip.boundary','clip');for(const [i,row]of(v.layerOverrides||[]).entries()){link(key,row.layer,'ifccad::viewport.layerOverrides.'+i+'.layer','layer');link(key,row.linePattern,'ifccad::viewport.layerOverrides.'+i+'.linePattern','pattern');}}
   if(a['ifccad::layout']||a['ifccad::blockDefinition']||a['ifccad::blockInstance'])nodes.get(key).children=Object.entries(item.children||{}).filter(([key])=>/^\d+$/.test(key)).sort(([x],[y])=>numeric(x,y)).map(([,target])=>target);
  }
  const role=r=>all.filter(n=>n.attributes?.['ifccad::'+r]).map(n=>n.path);
  for(const root of all.filter(n=>['leader','multiLeader','tolerance'].some(role=>n.attributes?.['ifccad::'+role]))){const pending=[...nodes.get(root.path).children],seen=new Set();while(pending.length){const path=pending.pop();if(seen.has(path))continue;seen.add(path);const part=nodes.get(path);if(!part)continue;part.owner=root.path;pending.push(...part.children);}}
  const drawing=role('drawing');roots.push(...drawing);
  for(const [r,title]of [['layout','layouts'],['layer','layers'],['linePattern','linePatterns'],['symbolSource','symbolSources'],['blockDefinition','blocks'],['preservationRecord','preservation'],['ucsDefinition','ucsDefinitions'],['modelWindow','modelWindows'],['annotationScale','annotationScales'],['annotationView','annotationViews']]){const children=role(r);if(r==='layout')children.sort((a,b)=>(nodes.get(a).values['ifccad::layout'].tabIndex||0)-(nodes.get(b).values['ifccad::layout'].tabIndex||0));if(children.length)roots.push(group(r,title,children));}
  const textStyles=role('textStyle');if(textStyles.length)roots.push(group('textStyle','textStyles',textStyles));
  const dimensionStyles=role('dimensionStyle');if(dimensionStyles.length)roots.push(group('dimensionStyle','dimensionStyles',dimensionStyles));
  const leaderStyles=role('multiLeaderStyle');if(leaderStyles.length)roots.push(group('multiLeaderStyle','multiLeaderStyles',leaderStyles));
  const known=new Set(['drawing','layout','layer','linePattern','symbolSource','textStyle','dimensionStyle','multiLeaderStyle','dimensionGraphics','dimensionGraphicsPart','blockDefinition','entity','opaqueEntity','preservation','preservationRecord','ucsDefinition','modelWindow','annotationScale','annotationView',...leaderRoles].flatMap(role));const others=all.filter(n=>!known.has(n.path)).map(n=>n.path);if(others.length)roots.push(group('other','otherNodes',others));
  for(const name of ['header','imports','schemas']){const n=put('file:'+name,name,graph[name]??source?.[name]??{},'file');roots.push(n.key);nodeRoots.push(n.key);}

 const drawingUnit=p.unit||(p.graph?.data||[]).find(n=>n.attributes?.['ifccad::drawing'])?.attributes['ifccad::drawing'].lengthUnit;
 const namedUcs=(from,choice,field)=>{if(choice?.kind==='Named')link(from,format==='ifccad'?choice.ucs:'ucs:'+id(choice.ucsId),field,'ucs');};
 for(const n of nodes.values()){

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

 }

 for(const n of nodes.values())if(['layout','blockDefinition','blockDefinitions','blockInstance'].includes(n.type)){
  const layout=n.values['ifccad::layout']||n.values;
  n.unit=n.type==='layout'&&String(layout.kind).toLowerCase()==='paper'?null:drawingUnit;
  const kinds=new Map();n.children.forEach((key,index)=>{const child=nodes.get(key);if(!child)return;child.owner=n.key;child.drawPosition=index;child.unit=n.unit;const kind=child.type;kinds.set(kind,[...(kinds.get(kind)||[]),key]);});
  n.groupedChildren=Array.from(kinds,([kind,keys])=>{const key=group('types:'+n.key+':'+kind,'plural.'+kind,keys);nodes.get(key).unit=n.unit;return key;});
 }
 const drawingPrefix=[...nodes.values()].find(n=>n.type==='drawing')?.key||[...nodes.values()].find(n=>n.type==='layout')?.key.split('/layout/')[0];
 for(const evidence of (result.conversion?.geometryAssessment||result.export?.geometryAssessment)?.filledSurfaces||[]){
  const source=evidence.source;if(!drawingPrefix)continue;
  if(source?.kind==='DimensionGraphicsPart'&&source.dimension?.kind==='NativeEntity'){
   const n=nodes.get(drawingPrefix+'/e'+source.dimension.entityId+'/graphics/p'+source.partIndex);
   if(n&&['solid','trace'].includes(n.type))n.filledEvidence=evidence;
   continue;
  }
  if(source?.kind!=='NativeEntity')continue;
  const n=nodes.get(drawingPrefix+'/e'+source.entityId),owner=drawingPrefix+(source.owner?.kind==='BlockDefinition'?'/block/':'/layout/')+source.owner?.id;
  if(n&&['solid','trace'].includes(n.type)&&n.owner===owner)n.filledEvidence=evidence;
 }
 for(const n of nodes.values())if(n.type==='blockInstance')n.unit=nodes.get(n.owner)?.unit;
 for(const n of nodes.values())if(n.type==='blockAttribute')n.unit=nodes.get(n.owner)?.unit;
 for(const n of nodes.values())if(leaderRoles.includes(n.type)&&n.owner)n.unit=nodes.get(n.owner)?.unit;
 for(const n of nodes.values())if(n.type==='dimensionGraphics'){n.unit=nodes.get(n.owner)?.unit;for(const key of n.children)nodes.get(key).unit=n.unit;}
 for(const n of nodes.values())if(n.type==='dimension'){
  n.sourceValues=n.values;

  if(n.derived?.effectiveSettings)dimensionRefs(n.key,n.derived.effectiveSettings,'derived.effectiveSettings',format==='ifccad');
 }
 for(const row of p.polylineWidths||[]){const n=nodes.get(row.path);if(n)n.polylineWidths=row.widths;}
 for(const row of p.extrusions||[]){const n=nodes.get(row.path);if(n)n.extrusion=row;}
 const widthNode=source=>{if(source?.kind==='NativeEntity')return nodes.get(drawingPrefix+'/e'+source.entityId);if(source?.kind==='DimensionGraphicsPart'&&source.dimension?.kind==='NativeEntity')return nodes.get(drawingPrefix+'/e'+source.dimension.entityId+'/graphics/p'+source.partIndex);};
 for(const row of (result.conversion?.geometryAssessment||result.export?.geometryAssessment)?.polylineWidths||[]){const n=widthNode(row.source);if(n)n.widthEvidence=row;}
 for(const n of nodes.values())n.incoming=[];
 for(const n of nodes.values())for(const edge of n.outgoing)nodes.get(edge.target)?.incoming.push(edge);
 const fontSymbolReport=createFontSymbolReport(p.fontSymbolRequests||[],result);
 return {fontSymbolReport,binary:result?.source?.representation==='decoded',format,nodes,roots,nodeRoots,storageRoots,sourceText,valid:result?.validation?.strictAvailable===true};
}
