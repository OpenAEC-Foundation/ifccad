import {normalizeTolerance} from './conversion-settings.mjs';

const equal=(a,b)=>['mode','value','unit','coordinateFallback'].every(key=>a?.[key]===b?.[key]);
export function createTolerancePolicies(){
 const policies={input:{mode:'default'},output:{mode:'default'}};
 return {selected(stage){if(!['input','output'].includes(stage))throw Error('Unknown tolerance stage');return {...policies[stage]};},apply(target,value){if(!['both','input','output'].includes(target))throw Error('Unknown tolerance target');const policy=normalizeTolerance(value);for(const stage of target==='both'?['input','output']:[target])policies[stage]={...policy};}};
}

/** Read certified limits from the actual result; never infer a Paper unit. */
export function effectiveToleranceRows(input,output){
 const report=output||input,exp=report?.export,conversion=report?.conversion;
 const geometry=exp?.geometryAssessment||exp?.geometry||conversion?.geometryAssessment||conversion?.geometry;
 const policy=exp?.options?.tolerance||conversion?.options?.tolerance;
 const names=new Map((input?.presentation?.layouts||[]).map(node=>{const layout=node.attributes?.['ifccad::layout']||node;return [String(layout.scopeId??layout.id??node.path?.split('/').at(-1)),layout.name];}));
 return (geometry?.domains||[]).map(domain=>{
  const kind=domain.domain?.kind||String(domain.domain).split('(')[0],id=String(domain.domain?.layoutId??String(domain.domain).match(/\((\d+)\)/)?.[1]??'');
  const meaning=domain.coordinateMeaning||{},paper=kind==='PaperLayout',unit=meaning.unit||'unitless',factor=meaning.physicalOutputFactor||null;
  return {id,name:paper?(names.get(id)||id):'Model',paper,unit,physicalOutputFactor:factor,lower:domain.resolvedTolerance?.lower,upper:domain.resolvedTolerance?.upper,fallback:policy?.mode==='custom'&&policy.unit!=='drawing'&&policy.coordinateFallback!==undefined&&(paper?!factor:unit==='unitless')};
 });
}

export function initializeToleranceSettings({translate,onApply}){
 const $=id=>document.getElementById(id),t=translate,dialog=$('tolerance-dialog'),fields=$('tolerance-fields'),inline=$('tolerance-inline');
 let source,output,view='overview',focus,previousBasis=$('tolerance-basis').value;
 const policies=createTolerancePolicies(),drafts={};let previousTarget='both';
 const values={drawing:'1e-9',physical:'0.001'};
 const number=value=>Number(Number(value).toPrecision(9)).toString();
 function selected(stage){if(stage)return policies.selected(stage);return normalizeTolerance({mode:$('tolerance-mode').value||'default',value:$('tolerance-value').value,unit:$('tolerance-basis').value==='physical'?$('tolerance-unit').value:'drawing',...($('tolerance-mode').value==='custom'&&$('tolerance-basis').value==='physical'?{coordinateFallback:$('tolerance-fallback').value}:{})});}
 function raw(){return {mode:$('tolerance-mode').value,basis:$('tolerance-basis').value,value:$('tolerance-value').value,unit:$('tolerance-unit').value,fallback:$('tolerance-fallback').value,values:{...values,[$('tolerance-basis').value]:$('tolerance-value').value}};}
 function load(value){$('tolerance-mode').value=value.mode;$('tolerance-basis').value=value.basis||'drawing';$('tolerance-value').value=value.value??'1e-9';$('tolerance-unit').value=value.unit==='m'?'m':'mm';$('tolerance-fallback').value=value.fallback??'1e-9';Object.assign(values,value.values||{});previousBasis=$('tolerance-basis').value;}
 function describe(policy){return t(policy.mode==='default'?'defaultTolerance':policy.mode==='exact'?'exact':'custom')+(policy.mode==='custom'?' · '+number(policy.value)+' '+(policy.unit==='drawing'?t('coordinateShort'):policy.unit)+(policy.coordinateFallback!==undefined?' / '+t('fallbackShort')+' '+number(policy.coordinateFallback)+' '+t('coordinateShort'):''):'');}
 function render(){
  const mode=$('tolerance-mode').value||'default',physical=$('tolerance-basis').value==='physical';
  $('custom-tolerance').hidden=mode!=='custom';$('tolerance-physical-unit').hidden=!physical;$('tolerance-coordinate-unit').hidden=physical;$('tolerance-fallback-control').hidden=mode!=='custom'||!physical;
  $('tolerance-basis-help').textContent=t(physical?'physicalToleranceHelp':'coordinateToleranceHelp');
  $('tolerance-preset-help').hidden=mode==='custom';$('tolerance-preset-help').textContent=t(mode==='exact'?'exactToleranceHelp':'defaultToleranceHelp');
  let tolerance;try{tolerance=selected();$('tolerance-error').hidden=true;$('tolerance-popup-apply').disabled=false;}catch(error){$('tolerance-error').textContent=t('invalidTolerance');$('tolerance-error').hidden=false;$('tolerance-popup-apply').disabled=true;}
  const inputPolicy=selected('input'),outputPolicy=selected('output'),target=$('tolerance-target').value||'both';
  $('tolerance-summary').textContent=equal(inputPolicy,outputPolicy)?describe(inputPolicy):t('separateTolerances');$('tolerance-open').title=t('inputConversion')+': '+describe(inputPolicy)+' · '+t('outputConversion')+': '+describe(outputPolicy);
  const nativeInput=source&&!/\.(dwg|dxf)$/i.test(source.source?.name||'');
  $('tolerance-stage-help').textContent=target==='input'&&nativeInput?t('nativeNoInputTolerance'):t(target==='both'?'bothToleranceHelp':target==='input'?'inputToleranceHelp':'outputToleranceHelp')+(target==='both'&&nativeInput?' '+t('nativeNoInputTolerance'):'');
  const list=$('tolerance-domain-list');list.replaceChildren();let matching=true;
  for(const stage of target==='both'?['input','output']:[target]){
  const rows=stage==='output'&&!output?[]:effectiveToleranceRows(source,stage==='output'?output:undefined),heading=document.createElement('h4');heading.textContent=t(stage==='input'?'inputConversion':'outputConversion');list.append(heading);
  if(!rows.length){const message=document.createElement('p');message.className='secondary';message.textContent=t(stage==='input'&&source&&!/\.(dwg|dxf)$/i.test(source.source?.name||'')?'nativeNoInputTolerance':'tolerancesAfterConversion');list.append(message);}
  for(const row of rows){
   const div=document.createElement('div'),name=document.createElement('strong'),meaning=document.createElement('span'),limit=document.createElement('span');div.className='tolerance-domain';
   name.textContent=row.paper?'Paper · '+row.name:row.name;
   const factor=row.physicalOutputFactor,mmPerCoordinate=factor?Number(factor.numerator)/Number(factor.denominator)*1000:NaN;
   meaning.className='secondary';meaning.textContent=row.paper?(factor?(Number.isFinite(mmPerCoordinate)&&mmPerCoordinate>0?t('paperPhysicalScale',{value:number(mmPerCoordinate)}):t('fixedPhysicalScale')):t('unknownPhysicalScale')):t('modelUnit',{unit:row.unit});
   if(factor)meaning.title=factor.numerator+'/'+factor.denominator+' m / '+t('paperCoordinate');
   limit.textContent=(Number.isFinite(row.upper)?number(row.upper):'—')+' '+(row.paper?t('paperCoordinate'):row.unit==='unitless'?t('modelCoordinate'):row.unit)+(row.fallback?' · '+t('fallbackShort'):'');
   if(row.lower!==row.upper)limit.title=t('certifiedLimitInterval',{lower:row.lower,upper:row.upper});div.append(name,meaning,limit);list.append(div);
  }
  const applied=stage==='output'?output?.export?.options?.tolerance:source?.conversion?.options?.tolerance;
  if(rows.length&&!equal(tolerance,applied||{mode:'default'}))matching=false;
  }
  $('tolerance-domain-empty').hidden=true;
  $('tolerance-domain-note').textContent=t(matching?'effectiveCurrentResult':'effectiveAfterApply');
  place();
 }
 function place(){
  if(!dialog.open)return;dialog.style.left='';dialog.style.top='';
  if(innerWidth<=620)return;
  const anchor=$('tolerance-open').getBoundingClientRect(),rect=dialog.getBoundingClientRect();
  dialog.style.left=Math.max(12,Math.min(anchor.left,innerWidth-rect.width-12))+'px';dialog.style.top=Math.max(12,Math.min(anchor.top-rect.height-8,innerHeight-rect.height-12))+'px';
 }
 function close(){if(dialog.open)dialog.close();}
 $('tolerance-open').onclick=()=>{focus=document.activeElement;dialog.showModal();$('tolerance-open').setAttribute('aria-expanded','true');place();};
 $('tolerance-close').onclick=close;dialog.addEventListener('close',()=>{$('tolerance-open').setAttribute('aria-expanded','false');focus?.focus();});
 dialog.addEventListener('click',event=>{if(event.target===dialog){const rect=dialog.getBoundingClientRect();if(event.clientX<rect.left||event.clientX>rect.right||event.clientY<rect.top||event.clientY>rect.bottom)close();}});
 $('tolerance-popup-apply').onclick=()=>{try{selected();}catch{return;}close();onApply();};
 for(const id of ['tolerance-mode','tolerance-value','tolerance-unit','tolerance-fallback'])$(id).addEventListener('input',render);
 $('tolerance-basis').onchange=()=>{values[previousBasis]=$('tolerance-value').value;previousBasis=$('tolerance-basis').value;$('tolerance-value').value=values[previousBasis];render();};
 $('tolerance-target').onchange=()=>{drafts[previousTarget]=raw();previousTarget=$('tolerance-target').value;const policy=policies.selected(previousTarget==='both'?'input':previousTarget);load(drafts[previousTarget]||{...policy,basis:policy.unit&&policy.unit!=='drawing'?'physical':'drawing',fallback:policy.coordinateFallback});render();};
 window.addEventListener('resize',place);
 render();
 return {selected,apply(){policies.apply($('tolerance-target').value||'both',selected());if(($('tolerance-target').value||'both')==='both'){drafts.input=raw();drafts.output=raw();}render();},refresh:render,setReport(input,next){if(input!==source){source=input;output=undefined;}if(next!==undefined)output=next;render();},setView(next){view=next;close();(view==='conversion'?inline:$('tolerance-dialog-content')).append(fields);$('tolerance-open').hidden=view==='conversion';render();}};
}
