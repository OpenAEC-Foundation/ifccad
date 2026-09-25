import { buildModel, revealNode } from './model.mjs';
import { PackageGraph } from './graph.mjs';
import { renderInspector } from './inspector.mjs';
import { t, translateTree } from './i18n.mjs';
import { initializeSettings } from './settings.mjs';
import { initializeOpening } from './open-files.mjs';
import { initializeExporting } from './export-files.mjs';
import { initializeCadPreview } from './cad-preview.mjs';
import { decodeBundle } from './bundle.mjs';
import { renderReport } from './reports.mjs';
import { initializeWorkspace } from './workspace.mjs';
const $=id=>document.getElementById(id);
let examples=[],model,collapsed=new Set(),selected='';
let result=null,reportNavigable=false,knownNodes=new Set();
const graph=new PackageGraph($('graph'),{select,toggle,zoomChanged:n=>$('zoom-level').textContent=n+'%',countChanged:(n,total)=>$('graph-count').textContent=n+' / '+total+' nodes',referenceCountChanged:n=>{const summary=$('graph-reference-summary');summary.hidden=n===0;summary.textContent=n?t(`${n} meer relaties in inspecteur`):'';}});
function render(){for(const n of model.nodes){if(!knownNodes.has(n.id)&&model.defaultCollapsed.has(n.id))collapsed.add(n.id);knownNodes.add(n.id);}graph.render(model,collapsed,selected);renderInspector(model,selected,collapsed);translateTree($('inspector'));}
function refreshPresentation(){
  if(model){
    const scroll=$('inspector').scrollTop,details=[...$('inspector').querySelectorAll('details')].map(d=>d.open),edited=$('native-edited')?.checked;
    render();[...$('inspector').querySelectorAll('details')].forEach((d,i)=>{if(details[i]!==undefined)d.open=details[i];});
    if(edited&&$('native-edited')){$('native-edited').checked=true;$('native-edited').dispatchEvent(new Event('change',{bubbles:true}));}
    $('inspector').scrollTop=scroll;
  }
  translateTree(document.body);
  if(result)showReport();
}
function select(id){if(id.startsWith('more:')){model.paging.materialize(id);selected=model.byId.get(id).parentId;collapsed.delete(selected);render();return;}model.paging?.materialize(id);if(!model.byId.has(id))return;for(const n of model.nodes){if(!knownNodes.has(n.id)&&model.defaultCollapsed.has(n.id))collapsed.add(n.id);knownNodes.add(n.id);}selected=id;revealNode(model,collapsed,id);render();$('inspector').scrollTop=0;graph.focus(id);}
function toggle(id){model.paging?.expand(id);for(const n of model.nodes){if(!knownNodes.has(n.id)&&model.defaultCollapsed.has(n.id))collapsed.add(n.id);knownNodes.add(n.id);}collapsed.has(id)?collapsed.delete(id):collapsed.add(id);selected=id;render();$('inspector').scrollTop=0;}
function showReport(){renderReport($('file-report'),result,{canNavigate:reportNavigable,diagnosticTarget:d=>{const match=d.location?.match(/^\/data\/(\d+)/),id=d.resourceId?'resource:'+d.resourceId:match?'ifcx:'+model.fixture.ifcx.data[Number(match[1])]?.path:null;return model.byId.has(id)?id:null;},selectTarget:id=>{workspaceView.focusReport(false);$('report-panel').open=false;select(id);}});}
function openResult(value,request){
  workspaceView.focusReport(false);
  const previous={model,collapsed,selected,knownNodes,camera:{...graph.camera},example:$('example').value,options:[...$('example').options].map(o=>({value:o.value,textContent:o.textContent,disabled:o.disabled,id:o.id})),description:$('example-description').textContent,status:$('example-status').textContent,concept:$('concept-key').hidden};
  result=value;reportNavigable=false;
  if(value.presentation&&value.validation?.strictAvailable){
    try{const fixture=decodeBundle(value.presentation);fixture.name=fixture.label=value.source.name;const next=buildModel(fixture);model=next;knownNodes=new Set();collapsed=new Set(model.defaultCollapsed);selected=model.roots[0];reportNavigable=true;document.getElementById('local-option')?.remove();const option=document.createElement('option');option.id='local-option';option.value='local';option.textContent=value.source.name;option.disabled=true;$('example').append(option);$('example').value='local';$('example-description').textContent=value.source.name;$('example-status').textContent='Geopend pakket';$('concept-key').hidden=true;render();graph.fit();}
    catch(error){value.failure={stage:'preparing',code:'VIEWER_DISPLAY_FAILED',message:error.message};reportNavigable=false;({model,collapsed,selected,knownNodes}=previous);$('example').replaceChildren(...previous.options.map(o=>Object.assign(document.createElement('option'),o)));$('example').value=previous.example;$('example-description').textContent=previous.description;$('example-status').textContent=previous.status;$('concept-key').hidden=previous.concept;if(model){render();graph.camera=previous.camera;graph.apply();}}
  }
  if(reportNavigable){exporter.setSource(request,model.fixture);preview.setSource(request,model.fixture,request.kind==='cad'?{name:request.files[0].path,base64:request.files[0].base64}:null);}
  else preview.clear();
  $('report-panel').hidden=false;$('report-panel').open=true;showReport();workspaceView.focusReport(true);translateTree(document.body);$('report-panel').scrollTop=0;
}
function openExample(value){const concept=value==='concept',fixture=examples.find(e=>e.name===(concept?'unrepresented-packed':value));model=buildModel(fixture,{concepts:concept});collapsed=new Set(model.defaultCollapsed);selected=model.roots.find(id=>id!=='group:definitions');
  workspaceView.focusReport(false);
  knownNodes=new Set(model.byId.keys());result=null;reportNavigable=false;$('report-panel').hidden=true;
  exporter.setSource(concept?null:{kind:'package',name:fixture.name,files:fixture.exportFiles},fixture);
  preview.setSource(concept?null:{kind:'package',name:fixture.name,files:fixture.exportFiles},fixture);
  if(concept){selected='concept:collection:dimension';revealNode(model,collapsed,selected);collapsed.delete(selected);}
  $('example-description').textContent=concept?'Dimension als voorbeeld van een verdere CAD-entiteit.':fixture.description;
  const version=fixture.ifcx.data.find(n=>n.attributes?.resource?.format==='openaec.ifcdr')?.attributes.resource.version;
  $('example-status').textContent=concept?'Concept · nog niet ondersteund':`IFCDR ${version} / IFCPR 0.2.0`;$('concept-key').hidden=!concept;render();translateTree(document.body);$('inspector').scrollTop=0;if(concept)graph.frame(['resource:drawing-main',selected,'entity:drawing-main:6']);else graph.fit();
}
async function initialize(){
  $('load-error').hidden=true;$('workspace').setAttribute('aria-busy','true');
  try{const response=await fetch('./examples.json');if(!response.ok)throw new Error('Voorbeelddata ontbreekt (HTTP '+response.status+').');examples=await response.json();if(!Array.isArray(examples)||!examples.length)throw new Error('Geen voorbeelden beschikbaar.');
    $('example').replaceChildren(...examples.map(e=>{const o=document.createElement('option');o.value=e.name;o.textContent=e.label;return o;}));const option=document.createElement('option');option.value='concept';option.textContent='Concept · extra CAD-entiteittypen';$('example').append(option);$('example').disabled=false;
    openExample(examples[0].name);$('workspace').setAttribute('aria-busy','false');
  }catch(error){$('load-error').hidden=false;$('error-message').textContent=t(error.message)+' '+t('Start de demo via de lokale server of serveer de gebouwde website.');$('workspace').setAttribute('aria-busy','false');translateTree(document.body);}
}
$('example').addEventListener('change',e=>{try{openExample(e.target.value);}catch(error){$('load-error').hidden=false;$('error-message').textContent=error.message;}});
$('zoom-in').addEventListener('click',()=>graph.zoom(1.2));$('zoom-out').addEventListener('click',()=>graph.zoom(1/1.2));$('fit').addEventListener('click',()=>graph.fit());$('collapse').addEventListener('click',()=>{collapsed=new Set(model.defaultCollapsed);selected=model.roots.find(id=>id!=='group:definitions');render();graph.fit();});
$('graph-relations').addEventListener('click',e=>{graph.referenceMode=graph.referenceMode==='focus'?'all':'focus';e.currentTarget.setAttribute('aria-pressed',graph.referenceMode==='all');render();});
function changePage(target,page){
 if(!Number.isFinite(page))return;
 const scroll=$('inspector').scrollTop,id=target.dataset.pageId,kind=target.dataset.pageKind;
 if(kind==='graph'){model.paging.setPage(id,page);collapsed.delete(id);}
 else{model.inspectorPages??=new Map();model.inspectorPages.set(id,Math.max(0,Math.floor(page)));}
 render();$('inspector').scrollTop=scroll;
 $('inspector').querySelector(`${target.tagName==='INPUT'?'input':'button'}[data-page-id="${CSS.escape(id)}"][data-page-kind="${kind}"]${target.tagName==='INPUT'?'':`[data-page="${page}"]`}`)?.focus({preventScroll:true});
}
$('inspector').addEventListener('click',e=>{const page=e.target.closest('button[data-page-id]');if(page){if(page.hasAttribute('data-page-go')){const input=page.closest('nav').querySelector('input');changePage(input,Number(input.value)-1);}else changePage(page,Number(page.dataset.page));return;}const target=e.target.closest('[data-select],[data-toggle]');if(!target)return;target.dataset.select?select(target.dataset.select):toggle(target.dataset.toggle);});
$('inspector').addEventListener('keydown',e=>{if(e.key==='Enter'&&e.target.matches('input[data-page-id]')){e.preventDefault();changePage(e.target,Number(e.target.value)-1);}});
$('inspector').addEventListener('change',e=>{if(e.target.id==='native-edited')$('reuse-result').textContent=t(e.target.checked?'De native gegevens zijn gewijzigd. Herstelbeleid, baseline en afhankelijkheden moeten opnieuw worden beoordeeld. Bewaarde bytes mogen niet blind worden teruggeplaatst.':'Brondata hergebruiken vereist een passende baseline, geldig herstelbeleid en gecontroleerde afhankelijkheden.');});
const workspaceView=initializeWorkspace(translateTree);
$('retry').addEventListener('click',initialize);
initializeSettings(refreshPresentation);
const exporter=initializeExporting();
const preview=initializeCadPreview();
$('preview-open').addEventListener('click',()=>{workspaceView.focusReport(false);$('report-panel').open=false;});
initializeOpening({onResult:openResult});
initialize();
