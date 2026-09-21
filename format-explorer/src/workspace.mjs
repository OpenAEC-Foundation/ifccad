/** Keep resizing local to the explorer; source data and graph zoom stay intact. */
export function inspectorWidth(width, requested=380){
 const max=Math.max(0,width-328),min=Math.min(280,max);
 return Math.round(Math.max(min,Math.min(max,requested)));
}

export function initializeWorkspace(refreshText=()=>{}){
 const workspace=document.getElementById('workspace'),divider=document.getElementById('inspector-divider');
 let preferred=380,drag=null;
 function apply(){
  const width=workspace.clientWidth;if(!width)return;
  const value=inspectorWidth(width,preferred);
  workspace.style.setProperty('--inspector-width',value+'px');
  divider.setAttribute('aria-valuemin',String(inspectorWidth(width,0)));
  divider.setAttribute('aria-valuemax',String(inspectorWidth(width,Infinity)));
  divider.setAttribute('aria-valuenow',String(value));
 }
 divider.addEventListener('pointerdown',e=>{
  if(e.button!==0)return;
  e.preventDefault();divider.focus({preventScroll:true});
  drag={x:e.clientX,width:document.getElementById('inspector').getBoundingClientRect().width};
  divider.setPointerCapture(e.pointerId);workspace.classList.add('resizing');
 });
 divider.addEventListener('pointermove',e=>{if(!drag)return;preferred=inspectorWidth(workspace.clientWidth,drag.width+drag.x-e.clientX);apply();});
 const finish=()=>{drag=null;workspace.classList.remove('resizing');};
 divider.addEventListener('pointerup',finish);divider.addEventListener('pointercancel',finish);divider.addEventListener('lostpointercapture',finish);
 divider.addEventListener('dblclick',()=>{preferred=380;apply();});
 divider.addEventListener('keydown',e=>{
  const current=Number(divider.getAttribute('aria-valuenow')),step=e.shiftKey?50:10;
  const value={ArrowLeft:current+step,ArrowRight:current-step,Home:0,End:Infinity}[e.key];
  if(value===undefined)return;e.preventDefault();preferred=inspectorWidth(workspace.clientWidth,value);apply();
 });
 new ResizeObserver(apply).observe(workspace);apply();

 const dialog=document.getElementById('support-dialog'),trigger=document.getElementById('support-toggle');
 trigger.addEventListener('click',()=>dialog.showModal());
 document.getElementById('support-close').addEventListener('click',()=>dialog.close());
 dialog.addEventListener('close',()=>trigger.focus({preventScroll:true}));
 dialog.addEventListener('click',e=>{const r=dialog.getBoundingClientRect();if(e.target===dialog&&(e.clientX<r.left||e.clientX>r.right||e.clientY<r.top||e.clientY>r.bottom))dialog.close();});
 const report=document.getElementById('report-panel'),reportExpand=document.getElementById('report-expand');
 function focusReport(expanded){
  document.getElementById('app').classList.toggle('report-focused',expanded);
  reportExpand.setAttribute('aria-expanded',String(expanded));
  reportExpand.replaceChildren(document.createTextNode(expanded?'Graph tonen':'Rapport vergroten'));
  if(expanded)report.open=true;
  refreshText(reportExpand);
 }
 reportExpand.addEventListener('click',e=>{e.preventDefault();e.stopPropagation();focusReport(!document.getElementById('app').classList.contains('report-focused'));});
 report.addEventListener('toggle',()=>{if(!report.open)focusReport(false);});
 return {focusReport};
}
