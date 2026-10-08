import {translate} from './i18n.mjs';
import {inspectionNavigation} from './inspection-navigation.mjs';
import {ifcxNodeJson} from './presentation-json.mjs';
export const escapeHtml=value=>String(value??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
export function itemTitle(node,language){return node.type==='drawing'?translate(language,'drawing')+(node.key==='drawing'?'':' · '+node.title):node.type==='group'||node.type==='file'?translate(language,node.title):['node','layer','layers','layout','linePattern','linePatterns','blockDefinition','blockDefinitions','table','stream'].includes(node.type)?node.title:translate(language,node.type)+' · '+node.title;}
export function renderInspection(model,state){
 const chevron='<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M5.5 3 L10.5 8 L5.5 13"/></svg>';
 const {language,selection,expanded,inspector,field=[]}=state,t=(k,v)=>translate(language,k,v),e=escapeHtml;
 const summary=value=>value===null?'null':Array.isArray(value)?(value.length<=4&&value.every(v=>typeof v!=='object')?'['+value.map(v=>e(v)).join(', ')+']':t('items',{count:value.length})):typeof value==='object'?t('fields',{count:Object.keys(value).length}):typeof value==='boolean'?t(value?'yes':'no'):e(value);
 const keyLabel=key=>t(key.replace(/^ifccad::geom::|^ifccad::/,''));
 const jump=(key,label)=>'<button class="reference" data-select="'+e(key)+'">'+e(label)+' ↗</button>';
 const navigation=inspectionNavigation(model,state),roots=navigation.roots;
 const displayChildren=n=>navigation.children.get(n.key)||[];
 const forced=new Set(navigation.path(selection));
 const collection=state.collection?navigation.collection(state.collection):navigation.forSelection(selection);
 function fields(value,path=[],depth=0,owner=''){
  if(value===null||typeof value!=='object'||depth>16)return '';
  const entries=Object.entries(value),ownerNode=navigation.nodes.get(owner);
  return entries.slice(0,100).map(([key,v])=>{
   const next=[...path,key],attribute=e(JSON.stringify(next)),fieldId=JSON.stringify([owner,next]),selected=owner===selection&&JSON.stringify(next)===JSON.stringify(field),label=e(keyLabel(key));
   const composite=v!==null&&typeof v==='object'&&!(Array.isArray(v)&&v.length<=4&&v.every(item=>item===null||typeof item!=='object'));
   if(composite)return '<details class="inspector-field" data-field-id="'+e(fieldId)+'"'+(!state.fieldCollapsed?.has(fieldId)&&(depth===0||state.fieldExpanded?.has(fieldId))?' open':'')+'><summary>'+chevron+'<span class="property-label">'+label+'</span></summary><div class="field-children">'+fields(v,next,depth+1,owner)+'</div></details>';
   const edge=ownerNode?.outgoing.find(link=>link.field===next.join('.'));
   return '<div class="inspector-field field-value"><button class="property-choice'+(selected?' selected':'')+'" data-field="'+attribute+'"'+(selected?' aria-current="true"':'')+'>'+label+'</button><span class="field-content">'+(edge&&model.nodes.has(edge.target)?jump(edge.target,itemTitle(model.nodes.get(edge.target),language)):summary(v))+'</span></div>';
  }).join('')+(entries.length>100?'<div class="secondary">'+e(t('items',{count:entries.length}))+' · JSON</div>':'');
 }
 function tree(keys,seen=new Set()){return keys.map(key=>{const n=navigation.nodes.get(key);if(!n)return '';const children=displayChildren(n).length,open=expanded.has(key),cycle=seen.has(key);return '<div><div class="tree-row'+(key===selection?' selected':'')+'">'+(children&&!cycle?'<button class="tree-expand" data-toggle="'+e(key)+'" aria-expanded="'+open+'" aria-label="'+e(itemTitle(n,language))+'">'+chevron+'</button>':'<span class="tree-expand"></span>')+'<button class="tree-item" data-select="'+e(key)+'"'+(key===selection?' aria-current="true"':'')+'><span>'+e(itemTitle(n,language))+'</span>'+(n.type==='group'?'<span class="group-count">'+n.children.length+'</span>':'')+'</button></div>'+(open&&!cycle?'<div class="tree-children">'+tree(navigation.isLarge(key)?displayChildren(n).filter(child=>forced.has(child)):displayChildren(n),new Set([...seen,key]))+'</div>':'')+'</div>';}).join('');}
 const treeHtml=tree(roots),node=navigation.nodes.get(selection);if(!node)return {tree:treeHtml,details:'<p class="secondary">'+e(t('noSelection'))+'</p>',more:false,collection};
 let value=node.values;for(const key of field)value=value?.[key];const heading='<h2>'+e(field.length?keyLabel(field.at(-1)):itemTitle(node,language))+'</h2>'+(field.length?'<p class="secondary">'+e(itemTitle(node,language))+'</p>':'')+'<div class="identity">'+e(node.key)+(field.length?' / '+e(field.join('/')):'')+'</div>';
 let details=heading;
 if(node.drawPosition!=null)details+='<p class="secondary">'+e(t('drawPosition'))+': '+(node.drawPosition+1)+'</p>';
 if(inspector==='relations'){
  for(const [title,edges]of [['outgoing',node.outgoing],['incoming',node.incoming]])details+='<h3>'+e(t(title))+'</h3><dl class="properties-list">'+(edges.length?edges.map(link=>{const key=title==='outgoing'?link.target:link.source;return '<div><dt>'+e(link.field)+'</dt><dd>'+(model.nodes.has(key)?jump(key,itemTitle(model.nodes.get(key),language)):e(key)+' · '+e(t('unresolved')))+'</dd></div>';}).join(''):'<p class="secondary">'+e(t('noRelations'))+'</p>')+'</dl>';
 }else if(inspector==='source'){
  if(model.format==='ifccad'&&node.raw){details+='<p class="secondary">'+e(t('compositionHelp'))+'</p><details open><summary>'+e(t('effectiveNode'))+'</summary><pre>'+e(ifcxNodeJson(node.raw))+'</pre></details>';for(const f of node.fragments)details+='<details><summary>'+e(t('fragment',{index:f.index}))+' · data['+f.index+']</summary><pre>'+e(JSON.stringify(f.value,null,2))+'</pre></details>';}
  else details+='<h3>'+e(t(['stream','table'].includes(node.type)?'storageJson':'inspectionJson'))+'</h3><pre>'+e(JSON.stringify(value,null,2))+'</pre>';
 }else{
 if(!field.length&&node.workspace)details+='<p class="secondary">'+e(t('coordinateDomain'))+': '+e(t(node.workspace.coordinateDomain))+(node.workspace.unit?' · '+e(t('unitLabel',{unit:node.workspace.unit})): '')+'</p>';
 if(!field.length&&node.workspaceChoices){
  const current=value=>value==null?t('unspecified'):value.kind==='World'?t('ucsWorld'):value.kind==='Named'?(model.nodes.get(model.format==='ifccad'?value.ucs:'ucs:'+String(value.ucsId))?.title||t('unspecified')):value.kind||String(value);
  details+='<dl class="properties-list workspace-choices">'+Object.entries(node.workspaceChoices).map(([key,value])=>'<div><dt>'+e(t(key))+'</dt><dd>'+e(key==='currentUcs'?current(value):value==null?t('unspecified'):typeof value==='string'?(model.nodes.get(value)?.title||value):value.kind)+'</dd></div>').join('')+'</dl>';
 }
 const help=t('help.'+node.type);if(!help.startsWith('help.'))details+='<p class="secondary">'+e(help)+'</p>';
  const assessment=node.values.boundsAssessment;if(assessment)details+='<p class="secondary">'+e(t('boundsSummary',{quality:t(assessment.quality||'empty'),proof:t(assessment.enclosureVerified?'enclosureChecked':'enclosureNotChecked')}))+'</p>';
  details+=node.type==='group'?'<p class="secondary">'+e(t('group'))+'</p>':'<h3>'+e(t('storedValues'))+'</h3>';
  if(node.type!=='group')details+='<div class="inspector-fields" data-field-owner="'+e(node.key)+'">'+fields(node.values,[],0,node.key)+'</div>';

 }
 if(node.storage){const s=node.storage;details+='<h3>'+e(t('storageLocation'))+'</h3><dl class="properties-list"><div><dt>Stream</dt><dd>'+jump('stream:'+s.stream,s.stream)+'</dd></div><div><dt>'+e(t('row'))+'</dt><dd>'+s.row+'</dd></div>'+(s.offset!==undefined?'<div><dt>'+e(t('range'))+'</dt><dd>'+s.offset+' / '+s.count+'</dd></div>':'')+'</dl>';}
 return {tree:treeHtml,details,more:false,collection};
}
