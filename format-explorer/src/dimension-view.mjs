import {translate} from './i18n.mjs';

const escape=value=>String(value??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
export const isDimensionNode=node=>['dimension','dimensionStyle','dimensionStyles','dimensionGraphics'].includes(node?.type)||!!node?.values?.['ifccad::dimensionGraphicsPart'];
export function dimensionPresentation(node,format){
 if(!isDimensionNode(node))return null;
 return {authored:node.values,derived:node.derived,authoredPath:format==='ifccad'?['ifccad::dimension']:['geometry']};
}
export function dimensionFieldLabel(key,language){
 const field=key.replace(/^ifccad::/,'');
 const label=translate(language,'dimension.field.'+field);
 return label.startsWith('dimension.field.')?translate(language,field):label;
}
const enumFields=new Set(['type','kind','variant','state','format','angleFormat','application','role','attachment','insideOrientation','outsideOrientation','movement','preferredPlacement','arcSymbol','fractionLayout','mode','alignment','associationUpdate','displayEvaluation','origin','purpose']);
export function dimensionValueLabel(field,value,language){
 if(!enumFields.has(field)||typeof value!=='string')return value;
 const key='dimension.value.'+value.slice(0,1).toLowerCase()+value.slice(1),label=translate(language,key);
 return label===key?value:label;
}
function textRun(run){
 const decorations=[run?.underline?'underline':'',run?.overline?'overline':'',run?.strikeThrough?'line-through':''].filter(Boolean).join(' ');
 return '<span'+(decorations?' style="text-decoration:'+decorations+'"':'')+'>'+escape(run?.text)+'</span>';
}
export function renderDimensionLabel(label,language){
 if(!label)return '';
 if(label.state==='suppressed')return '<p class="secondary">'+escape(translate(language,'dimension.value.suppressed'))+'</p>';
 if(label.state==='unavailable')return '<p class="secondary">'+escape(label.reason||translate(language,'unavailable'))+'</p>';
 const fragments=label.fragments||(label.runs||[]).map(run=>({kind:'run',run}));
 const plain=label.runs?.map(run=>run.text).join('')??fragments.map(p=>p.kind==='fraction'?p.numerator+'/'+p.denominator:p.run?.text||'').join('');
 const content=fragments.map(part=>{
  if(part.kind==='run')return textRun(part.run);
  if(part.kind!=='fraction')return '';
  if(!['horizontal','diagonal'].includes(part.layout))return escape(part.numerator+'/'+part.denominator);
  const scale=Number.isFinite(part.textScale)&&part.textScale>0?part.textScale:1;
  return '<span class="dimension-fraction '+part.layout+'" style="font-size:'+scale+'em"><span class="numerator">'+escape(part.numerator)+'</span><span class="fraction-separator">/</span><span class="denominator">'+escape(part.denominator)+'</span></span>';
 }).join('');
 return '<div class="dimension-label" role="img" aria-label="'+escape(plain)+'"><span aria-hidden="true">'+content+'</span></div>';
}
