import {loadPreferences,savePreferences} from './preferences.mjs';
import {translate,localize} from './i18n.mjs';
import {createInspection} from './inspection-model.mjs';
import {renderInspection,itemTitle,escapeHtml} from './inspection-view.mjs';
import {inspectionNavigation,cadSelectionForNode} from './inspection-navigation.mjs';
import {initializeElementList} from './element-list.mjs';
import {initializeDialogs} from './dialogs.mjs';
import {inspectionReport,conversionDirections} from './inspection-report.mjs';
export function initializeWorkspace({onTab=()=>{},onPreferences=()=>{},onSelection=()=>{},onLayout=()=>{}}={}){
 const $=id=>document.getElementById(id);let storage;try{storage=localStorage;}catch{}
 const preferences=loadPreferences(storage,navigator.language,matchMedia('(prefers-color-scheme: dark)').matches);
 const state={language:preferences.language,theme:preferences.theme,tab:'overview',conversionTab:'input',reportsCollapsed:false,view:'drawing',grouping:'type',selection:null,expanded:new Set(),fieldExpanded:new Set(),fieldCollapsed:new Set(),inspector:'properties',mode:'readable',collection:null,listOpen:false,inspectorOverride:null,statusSpec:{key:'select'}};let model,result,cadReport;
 const t=(key,values)=>translate(state.language,key,values);let loading,cadStatus;
 function renderStatus(){const spec=state.statusSpec;$('status').textContent=spec.message||t(spec.key,spec.values);$('status').hidden=spec.quiet??['select','ready','loading','preparing','converting','validating','exporting','writing','checking'].includes(spec.key);}
 function renderReportVisibility(){const key=state.reportsCollapsed?'showReports':'hideReports';$('application').dataset.reports=state.reportsCollapsed?'collapsed':'expanded';$('conversion-collapse').setAttribute('aria-expanded',!state.reportsCollapsed);$('conversion-collapse').setAttribute('aria-label',t(key));$('conversion-collapse').title=t(key);}
 function renderCadStatus(){const phase=cadStatus?.phase,label=phase==='ready'?t('roundtripReady'):phase==='failed'?t('roundtripFailed'):phase?t(phase):'',text=phase?t('roundtripStatus',{format:cadStatus.format.toUpperCase(),version:cadStatus.version,status:label}):'';$('cad-roundtrip-status').textContent=text;$('cad-roundtrip-summary').textContent=text;$('cad-roundtrip-summary').hidden=!text;$('cad-roundtrip-state').hidden=!text;$('cad-roundtrip-status').title=cadStatus?.error||'';$('cad-roundtrip-cancel').hidden=!phase||['ready','failed','cancelled'].includes(phase);}
 function renderLoading(){const panel=$('loading-panel');panel.hidden=!loading;document.querySelectorAll('.workspace-body>section').forEach(section=>section.setAttribute('aria-busy',!!loading));if(loading){$('loading-name').textContent=loading.name;$('loading-action').textContent=t(loading.action,loading);$('loading-phase').textContent=t(loading.phase);}}
 const elementList=initializeElementList({onSelect:key=>select(key),t,getLanguage:()=>state.language});
 let revealList=false;
 function layoutContents(){
  const open=!!model&&state.mode==='readable'&&state.listOpen;
  $('drawing').dataset.collection=open?'open':'closed';
  $('element-pane').hidden=!open;$('structure-resize').hidden=!open;
  const max=Math.max(120,Math.round($('drawing').clientWidth*0.55));$('structure-resize').setAttribute('aria-valuemax',max);
  const visible=state.inspectorOverride??!($('drawing').clientWidth<520||open&&$('drawing').clientWidth<850);
  $('drawing').dataset.info=visible?'visible':'hidden';$('information-pane').hidden=!visible;
  $('inspector-toggle').setAttribute('aria-expanded',visible);
 }
 new ResizeObserver(layoutContents).observe($('drawing'));
 function render(){
  $('item-information').querySelectorAll('details[data-field-id]').forEach(d=>{const id=d.dataset.fieldId;if(d.open){state.fieldExpanded.add(id);state.fieldCollapsed.delete(id);}else{state.fieldExpanded.delete(id);state.fieldCollapsed.add(id);}});
  document.documentElement.lang=state.language;document.documentElement.dataset.theme=state.theme;localize(document,state.language);$('language').value=state.language;$('theme').value=state.theme;
  $('application').dataset.workspace=state.tab;
  $('name').textContent=result?.source?.name||t('select');renderStatus();renderReportVisibility();
  const overview=state.tab==='overview';
  if(overview){$('workspace-body').setAttribute('role','group');$('workspace-body').setAttribute('aria-label',t('overview'));}else{$('workspace-body').removeAttribute('role');$('workspace-body').removeAttribute('aria-label');}
  for(const name of ['contents','conversion','drawing'])$(name+'-panel').hidden=!overview&&state.tab!==name;
  const format=model?.format==='ocdraw'?'OCDraw':'IFCCAD';$('content-title').textContent=format+'-'+t('contents').toLowerCase();$('content-filename').textContent=result?.source?.name||'';$('content-filename').title=result?.source?.name||'';$('content-validity').textContent=t('invalid');$('content-validity').hidden=!model||model.valid;$('export').title=t('downloadNative',{format});$('export').setAttribute('aria-label',$('export').title);
  for(const name of ['properties','relations','source'])$(name+'-tab').setAttribute('aria-selected',state.inspector===name);$('item-information').setAttribute('aria-labelledby',state.inspector+'-tab');
  $('structure').options[1].value=model?.format==='ocdraw'?'storage':'nodes';$('structure').options[1].textContent=t(model?.format==='ocdraw'?'storage':'nodes');$('structure').value=state.view;
  $('grouping-control').hidden=state.view!=='drawing'||state.mode==='json';$('grouping').value=state.grouping;
  $('empty-contents').hidden=!!model;$('content-tree').hidden=state.mode!=='readable';$('records').hidden=state.mode!=='json';$('readable-view').setAttribute('aria-pressed',state.mode==='readable');$('json-view').setAttribute('aria-pressed',state.mode==='json');
  layoutContents();
  if(model){const rendered=renderInspection(model,state);$('content-tree').innerHTML=rendered.tree;$('item-information').innerHTML=rendered.details;elementList.show(state.listOpen?rendered.collection:null,inspectionNavigation(model,state),model,state.selection,{revealSelection:revealList});revealList=false;const n=model.nodes.get(state.selection);const json=model.sourceText;$('records').textContent=(json||'').slice(0,2*1024*1024);$('tree-heading').textContent=state.mode==='json'?t('fileJson'):t('structure');$('selection-status').textContent=n?t('selection',{name:itemTitle(n,state.language)}):'';$('unit-status').textContent=t('unitLabel',{unit:result.presentation?.unit||result.presentation?.graph?.data?.find(n=>n.attributes?.['ifccad::drawing'])?.attributes['ifccad::drawing'].lengthUnit||'—'});$('document-format').textContent=format;$('document-validity').textContent=t(model.valid?'strictValid':'invalid');}
  renderReport();renderLoading();renderCadStatus();
 }
 function reportSections(target,parts){
  target.replaceChildren();
  for(const [key,part]of parts){
   const section=document.createElement('section');section.className='report-section';const h=document.createElement('h3');h.textContent=t(key);section.append(h);
   if(!part){const p=document.createElement('p');p.textContent=t('notRun');p.className='secondary';section.append(p);}
   else{
    const messages=part.messages||part.diagnostics||[],p=document.createElement('p');p.textContent=key==='validation'?t(part.strictAvailable?'strictValid':'invalid'):part.message||part.status||part.format||'';section.append(p);
    if(messages.length){const ul=document.createElement('ul');for(const message of messages.slice(0,500)){const li=document.createElement('li');li.textContent=typeof message==='string'?message:[message.code,message.location,message.message,message.action,...(message.reasons||[])].filter(Boolean).join(' · ');ul.append(li);}if(state.tab==='overview'){const details=document.createElement('details'),summary=document.createElement('summary');summary.textContent=t('reportMessages',{count:messages.length});details.append(summary,ul);section.append(details);}else section.append(ul);}
    if(part.text?.entries?.some(e=>e.glyphCoverage==='unassessed')){const notice=document.createElement('p');notice.className='secondary';notice.textContent=t('textGlyphNotice');section.append(notice);}
    if(key==='outputCheck'&&part.cadReadback===true)p.textContent=t('cadReadbackPassed');
    if(part.geometry||part.geometryAssessment||part.text||part.preservation||key==='outputCheck'){const details=document.createElement('details'),summary=document.createElement('summary'),pre=document.createElement('pre');summary.textContent=t(key==='outputCheck'?'readbackDetails':part.preservation?'restorationDetails':'conversionEvidence');pre.textContent=JSON.stringify(key==='outputCheck'?part:part.text?{geometry:part.geometry||part.geometryAssessment,text:part.text}:part.preservation?part:part.geometry||part.geometryAssessment,null,2);details.append(summary,pre);section.append(details);}
   }
   target.append(section);
  }
 }
 function renderReport(){
  const report=inspectionReport(result,cadReport),directions=conversionDirections(result,cadReport,cadStatus?.phase?cadStatus:{format:$('preview-format').value,version:$('preview-version').value});
  for(const name of ['input','output']){$(name+'-report-panel').hidden=state.tab!=='overview'&&state.conversionTab!==name;$(name+'-report-panel').setAttribute('role',state.tab==='overview'?'region':'tabpanel');$(name+'-conversion-controls').hidden=state.tab!=='overview'&&state.conversionTab!==name;$(name+'-conversion-tab').setAttribute('aria-selected',state.conversionTab===name);$(name+'-conversion-tab').tabIndex=state.conversionTab===name?0:-1;}
  $('output-conversion-actions').hidden=state.tab!=='overview'&&state.conversionTab!=='output';
  $('input-conversion-direction').textContent=directions.input;
  $('output-conversion-direction').textContent=directions.output+(directions.output&&directions.version?' · '+directions.version:'');
  $('input-report-direction').textContent=$('input-conversion-direction').textContent;$('output-report-direction').textContent=$('output-conversion-direction').textContent;
  $('input-conversion-description').textContent=!result?t('select'):directions.inputConverted?t('inputConversionDescription',{direction:directions.input}):t('nativeOpeningDescription',{format:directions.input});
  $('output-conversion-description').textContent=!result?t('select'):!result.validation?.strictAvailable||result.failure?t('outputBlocked'):t('outputConversionDescription',{direction:directions.output});
  $('input-report').textContent=JSON.stringify(report.input,null,2);$('report').textContent=JSON.stringify(report.output,null,2);
  reportSections($('input-readable-report'),[['reading',report.input.reader],['validation',report.input.validation],...(directions.inputConverted?[['inputConversion',report.input.conversion]]:[]),...(report.input.failure?[['failure',report.input.failure]]:[])]);
  const readback=report.output.export?.fileCheck||report.output.readback;
  reportSections($('readable-report'),[['export',report.output.export],...(report.output.restoration?[['restoration',report.output.restoration]]:[]),...(readback?[['outputCheck',readback]]:[]),...(report.output.failure?[['failure',report.output.failure]]:[])]);
 }
 function acceptCadReport(value){const previousFailure=cadReport?.failure?.message;cadReport=value;if(value){cadStatus={phase:value.failure?'failed':value.export?'ready':cadStatus?.phase||'',format:value.export?.format||cadStatus?.format||$('preview-format').value||'dxf',version:value.export?.requestedVersion||cadStatus?.version||$('preview-version').value,error:value.failure?.message||''};if(value.failure)state.statusSpec={message:value.failure.message};else if(previousFailure&&state.statusSpec.message===previousFailure)state.statusSpec={key:'ready'};}renderStatus();renderReport();renderCadStatus();}
 function acceptCadStatus(value){cadStatus=value;if(!value.phase||['preparing','cancelled'].includes(value.phase))cadReport=null;else if(cadReport?.failure&&value.phase==='ready')cadStatus={...value,phase:'failed',error:cadReport.failure.message};renderReport();renderCadStatus();}
 function select(key,notify=true){
  if(!model||!inspectionNavigation(model,state).nodes.has(key))return;
  state.selection=key;
  const navigation=inspectionNavigation(model,state),path=navigation.path(key);
  for(const parent of path.slice(0,-1))state.expanded.add(parent);
  const node=navigation.nodes.get(key);
  if(node.type==='group'&&!Object.keys(node.values||{}).length&&!navigation.isLarge(key)&&(navigation.children.get(key)?.length||0)>0)state.expanded.add(key);
  const collection=navigation.activeCollection(key,state.listOpen);
  state.collection=collection?.key||null;
  if(collection){state.listOpen=true;if(collection.key===key)state.expanded.add(key);}
  revealList=true;render();
  $('content-tree').querySelector('[aria-current=true]')?.scrollIntoView({block:'nearest'});
  if(notify){const target=cadSelectionForNode(model,key);if(target?.kind==='layout')onLayout(target.layout);else onSelection(target?.key||null);}
 }
 function changeView(name){const previous=state.tab;state.tab=name;render();onTab(name);$(name==='overview'?previous+'-expand':name+'-restore')?.focus();}
 document.querySelectorAll('[data-tab]').forEach(b=>b.onclick=()=>changeView(b.dataset.tab));
 $('conversion-collapse').onclick=()=>{state.reportsCollapsed=!state.reportsCollapsed;renderReportVisibility();};
 document.querySelectorAll('[data-inspector]').forEach(b=>b.onclick=()=>{state.inspector=b.dataset.inspector;render();});
 const conversionTabs=Array.from(document.querySelectorAll('[data-conversion-tab]'));
 for(const b of conversionTabs){b.onclick=()=>{state.conversionTab=b.dataset.conversionTab;renderReport();};b.onkeydown=e=>{if(!['ArrowLeft','ArrowRight','Home','End'].includes(e.key))return;e.preventDefault();const index=conversionTabs.indexOf(b),next=e.key==='Home'?0:e.key==='End'?conversionTabs.length-1:(index+(e.key==='ArrowRight'?1:-1)+conversionTabs.length)%conversionTabs.length;conversionTabs[next].click();conversionTabs[next].focus();};}
 $('structure').onchange=e=>{state.view=e.target.value;state.collection=null;state.listOpen=false;elementList.reset();if(!model){render();return;}const navigation=inspectionNavigation(model,state);select(navigation.nodes.has(state.selection)?state.selection:navigation.roots[0],false);};$('readable-view').onclick=()=>{state.mode='readable';render();};$('json-view').onclick=()=>{state.mode='json';render();};$('element-close').onclick=()=>{state.listOpen=false;state.collection=null;elementList.reset();render();$('content-tree').querySelector('[aria-current=true]')?.focus();};$('inspector-toggle').onclick=()=>{state.inspectorOverride=$('drawing').dataset.info!=='visible';layoutContents();};
 $('grouping').onchange=e=>{state.grouping=e.target.value;state.collection=null;state.listOpen=false;elementList.reset();select(state.selection,false);};
 $('drawing').addEventListener('click',event=>{const b=event.target.closest('button');if(!b)return;if(b.dataset.select)select(b.dataset.select);if(b.dataset.toggle){const key=b.dataset.toggle,navigation=inspectionNavigation(model,state);if(navigation.isLarge(key)){select(key);state.expanded.add(key);}else state.expanded.has(key)?state.expanded.delete(key):state.expanded.add(key);render();}});
 for(const name of ['language','theme'])$(name).onchange=e=>{state[name]=e.target.value;savePreferences(storage,{language:state.language,theme:state.theme});render();onPreferences({language:state.language,theme:state.theme});};
 initializeDialogs();const handle=document.querySelector('.information-resize');let width=350;
 const proportions={left:50,top:62};
 for(const [id,key,min,max,back,forward]of [['overview-column-resize','left',35,65,'ArrowLeft','ArrowRight'],['overview-row-resize','top',40,95,'ArrowUp','ArrowDown']]){
  const separator=$(id),resizeOverview=value=>{if(key==='top'){const height=$('workspace-body').getBoundingClientRect().height,compact=$('conversion-panel').querySelector('.overview-heading').getBoundingClientRect().height+document.querySelector('.conversion-settings').getBoundingClientRect().height+6,threshold=(height-compact)/height*100;state.reportsCollapsed=value>=threshold-1;renderReportVisibility();if(state.reportsCollapsed){separator.setAttribute('aria-valuenow',Math.round(threshold));return;}}proportions[key]=Math.max(min,Math.min(max,value));document.documentElement.style.setProperty('--overview-'+key,proportions[key]+'%');separator.setAttribute('aria-valuenow',Math.round(proportions[key]));};
  separator.setAttribute('aria-valuemin',min);separator.setAttribute('aria-valuemax',max);separator.setAttribute('aria-valuenow',proportions[key]);
  separator.onpointerdown=e=>{e.preventDefault();separator.setPointerCapture(e.pointerId);};separator.onpointermove=e=>{if(!separator.hasPointerCapture(e.pointerId))return;const rect=$('workspace-body').getBoundingClientRect();resizeOverview(key==='left'?(e.clientX-rect.left)/rect.width*100:(e.clientY-rect.top)/rect.height*100);};separator.onpointerup=separator.onpointercancel=e=>{if(separator.hasPointerCapture(e.pointerId))separator.releasePointerCapture(e.pointerId);};separator.onkeydown=e=>{if(![back,forward].includes(e.key))return;e.preventDefault();const current=key==='top'&&state.reportsCollapsed?$('contents-panel').getBoundingClientRect().height/$('workspace-body').getBoundingClientRect().height*100:proportions[key];resizeOverview(current+(e.key===forward?3:-3));};
 }
 const resize=w=>{width=Math.max(240,Math.min(520,w));$('drawing').style.setProperty('--inspector-width','min(40%, '+width+'px)');};handle.addEventListener('pointerdown',event=>{handle.setPointerCapture(event.pointerId);});handle.addEventListener('pointermove',event=>{if(handle.hasPointerCapture(event.pointerId))resize(document.getElementById('drawing').getBoundingClientRect().right-event.clientX);});handle.addEventListener('pointerup',event=>handle.releasePointerCapture(event.pointerId));handle.addEventListener('keydown',e=>{if(['ArrowLeft','ArrowRight'].includes(e.key)){e.preventDefault();resize(document.querySelector('.information-pane').getBoundingClientRect().width+(e.key==='ArrowLeft'?20:-20));}});
 const structureHandle=$('structure-resize');let treeWidth=220;
 const resizeStructure=value=>{treeWidth=Math.max(120,Math.min($('drawing').clientWidth*0.55,value));$('drawing').style.setProperty('--tree-width','min(55%, '+treeWidth+'px)');structureHandle.setAttribute('aria-valuenow',Math.round(treeWidth));};
 structureHandle.setAttribute('aria-valuemin','120');structureHandle.setAttribute('aria-valuenow',treeWidth);
 structureHandle.onpointerdown=e=>{e.preventDefault();structureHandle.setPointerCapture(e.pointerId);};structureHandle.onpointermove=e=>{if(structureHandle.hasPointerCapture(e.pointerId))resizeStructure(e.clientX-$('drawing').getBoundingClientRect().left);};structureHandle.onpointerup=structureHandle.onpointercancel=e=>{if(structureHandle.hasPointerCapture(e.pointerId))structureHandle.releasePointerCapture(e.pointerId);};structureHandle.onkeydown=e=>{if(['ArrowLeft','ArrowRight'].includes(e.key)){e.preventDefault();resizeStructure(document.querySelector('.tree-pane').getBoundingClientRect().width+(e.key==='ArrowRight'?20:-20));}};
 render();return {state,t,revealCadElement(key){if(!model?.nodes.has(key))return false;state.mode="readable";state.view="drawing";select(key,false);return true;},setCadStatus:acceptCadStatus,setLoading(value){loading={...loading,...value};renderLoading();},clearLoading(){loading=null;renderLoading();},setCadReport:acceptCadReport,show(value,sourceText){cadReport=null;result=value;state.statusSpec=value.failure?{message:value.failure.message}:{key:value.validation?.strictAvailable?'ready':'invalid'};state.fieldExpanded.clear();state.fieldCollapsed.clear();$('item-information').replaceChildren();model=createInspection(value,sourceText);state.collection=null;state.listOpen=false;elementList.reset();if(!model.nodes.has(state.selection)){state.selection=model.roots[0]||null;state.expanded=new Set(Array.from(model.nodes.values()).filter(n=>n.type==='layout'&&(n.values.kind==='model'||n.values['ifccad::layout']?.kind==='Model')||n.key==='group:layout'||n.key==='group:layouts').map(n=>n.key));state.view='drawing';state.mode='readable';}select(state.selection,false);},clear(){state.collection=null;state.listOpen=false;elementList.reset();model=null;result=null;cadReport=null;cadStatus=null;state.selection=null;state.expanded.clear();state.fieldExpanded.clear();state.fieldCollapsed.clear();$('selection-status').textContent='';$('unit-status').textContent='';$('document-format').textContent='';$('document-validity').textContent='';render();},selectTab:changeView,refresh:render,setMessage(message,quiet=false){state.statusSpec={message,quiet};renderStatus();},setStatus(key,values){state.statusSpec={key,values};renderStatus();}};
}
