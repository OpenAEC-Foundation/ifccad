import {t,translateTree} from './i18n.mjs';
const el=(tag,text,cls)=>{const e=document.createElement(tag);if(text!==undefined)e.textContent=text;if(cls)e.className=cls;return e;};
export function summarizeEntities(rows){const kinds=new Map();for(const row of rows){if(!kinds.has(row.kind))kinds.set(row.kind,{kind:row.kind,read:0,emitted:0,partial:0,skipped:0,unclassified:0});const k=kinds.get(row.kind);k.read++;if(['emitted','partial'].includes(row.disposition))k.emitted++;if(row.disposition!=='emitted')k[row.disposition]++;}return [...kinds.values()];}
function paged(parent,items,render){let next=0;const list=el('div',undefined,'report-list'),more=el('button','Meer tonen');parent.append(list,more);function page(){const end=Math.min(next+50,items.length);for(;next<end;next++)list.append(render(items[next]));more.hidden=next>=items.length;translateTree(parent);}more.onclick=page;page();}
export function renderReport(container,result,{selectTarget=()=>{},canNavigate=false,diagnosticTarget=()=>null}={}){
 container.replaceChildren();if(!result){container.hidden=true;return;}container.hidden=false;
 container.append(el('h2',result.source.name));
 const status=el('div',undefined,'report-status');container.append(status);
 const p=result.validation;
 if(result.failure)status.append(el('p',result.failure.stage+' · '+result.failure.code+' · '+result.failure.message,'report-error'));
 if(p){const a=p.report.assessment;status.append(el('strong',a.validity==='valid'?'Geldig binnen gecontroleerd contract':a.validity==='invalid'?'Ongeldig pakket':'Niet volledig beoordeeld'));
  status.append(el('p',a.completeness==='complete'?'Beoordeling compleet':'Beoordeling onvolledig'));
  status.append(el('p',p.strictAvailable?'Ondersteund door de packagelezer':'Geen strikt leesbaar pakket'));
  const ds=p.report.diagnostics||[],gaps=a.gaps||[];
  if(ds.length||gaps.length){const details=el('details'),summary=el('summary','Validatiemeldingen');details.append(summary);paged(details,[...ds,...gaps],d=>{
   const row=el('div',undefined,'report-item');row.append(el('code',d.code||d.reason),el('p',d.message||d.reason),el('small',[d.resourceId,d.resourceUri,d.location].filter(Boolean).join(' · ')));const target=canNavigate&&diagnosticTarget(d);if(target){const button=el('button','Bekijk in de graph');button.onclick=()=>selectTarget(target);row.append(button);}return row;
  });container.append(details);}
 }
 if(result.reader.messages?.length){const d=el('details');d.append(el('summary','Meldingen van de CAD-lezer'));paged(d,result.reader.messages,m=>el('p',m));container.append(d);}
 const c=result.conversion;
 if(c){container.append(el('h3','Van CAD naar IFCCAD'));
  if(c.assessment){container.append(el('p',c.assessment.conclusion==='LossDetected'?'Informatieverlies vastgesteld':c.assessment.conclusion==='NoLossDetected'?'Geen verlies vastgesteld binnen de beoordeelde inhoud':'Niet volledig beoordeeld'));
   container.append(el('p','Dit rapport beoordeelt de inhoud die de CAD-lezer heeft aangeleverd. Het bewijst niet dat alle informatie uit het oorspronkelijke bestand is ingelezen.','small-note'));
   const evidence=el('details');evidence.append(el('summary','Beoordelingsgrenzen en nauwkeurigheid'),el('pre',JSON.stringify({assessment:c.assessment,geometry:c.geometry},null,2)));container.append(evidence);
  }
  const table=el('table',undefined,'stream-table'),head=el('tr');for(const label of ['Type','Ingelezen','Overgekomen','Waarvan gedeeltelijk','Overgeslagen','Niet ingedeeld'])head.append(el('th',label));const thead=el('thead');thead.append(head);table.append(thead);const body=el('tbody');for(const k of summarizeEntities(c.entities)){const row=el('tr');for(const value of Object.values(k))row.append(el('td',value));body.append(row);}table.append(body);const scroll=el('div',undefined,'report-table');scroll.append(table);container.append(scroll);
  container.append(el('p','Aantallen betreffen entiteiten, geen percentage behouden informatie.','small-note'));
  const filter=el('select');filter.setAttribute('aria-label','Entiteiten filteren');for(const [value,label]of [['all','Alle entiteiten'],['emitted','Overgekomen'],['partial','Gedeeltelijk'],['skipped','Overgeslagen'],['unclassified','Niet ingedeeld']]){const o=el('option',label);o.value=value;filter.append(o);}const rows=el('div');container.append(filter,rows);
  function show(){rows.replaceChildren();paged(rows,c.entities.filter(e=>filter.value==='all'||e.disposition===filter.value),e=>{const row=el('div',undefined,'report-item');row.append(el('strong',e.kind+' · '+e.handle),el('span',({emitted:'Overgekomen',partial:'Gedeeltelijk',skipped:'Overgeslagen',unclassified:'Niet ingedeeld'})[e.disposition]));for(const reason of e.reasons)row.append(el('p',reason));if(canNavigate&&e.target?.resourceId){const button=el('button','Bekijk in de graph');button.onclick=()=>selectTarget('entity:'+e.target.resourceId+':'+e.target.entityId);row.append(button);}return row;});}filter.onchange=show;show();
  if(c.diagnostics.length){const d=el('details');d.append(el('summary','Alle conversiemeldingen'));paged(d,c.diagnostics,x=>{const row=el('div',undefined,'report-item');row.append(el('code',typeof x.source==='string'?x.source:JSON.stringify(x.source)),el('p',x.action+' · '+x.reasons.join(' · ')));return row;});container.append(d);}
 }
 for(const w of result.presentation?.warnings||[])container.append(el('p',w.message,'small-note'));
 if(result.presentation?.documents){const d=el('details');d.append(el('summary','Oorspronkelijke JSON-documenten'));paged(d,result.presentation.documents,file=>{const row=el('details');row.append(el('summary',file.path),el('pre',file.text.slice(0,12000)));if(file.text.length>12000)row.append(el('p','Preview ingekort tot 12000 tekens.','small-note'));return row;});container.append(d);}
 if(!canNavigate)container.append(el('p','De graph toont nog het vorige pakket.','small-note'));
 translateTree(container);
}
