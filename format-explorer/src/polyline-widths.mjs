import {translate} from './i18n.mjs';
const escape=value=>String(value??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
export function widthPresentation(widths,language){
 if(!widths)return '';const t=key=>translate(language,'width.'+key),e=escape;
 let html='<p>'+e(t(widths.kind||'unknown'))+(widths.kind==='constant'?': '+e(widths.width):'')+'</p>';
 if(widths.kind==='defaulted'){
 html+='<p>'+e(t('defaults'))+': '+e(widths.defaultStartWidth)+' / '+e(widths.defaultEndWidth)+'</p>';
 html+='<table><thead><tr><th>#</th><th>'+e(t('start'))+'</th><th>'+e(t('end'))+'</th></tr></thead><tbody>';
 for(const [i,row]of (widths.overrides||[]).entries()){const leaf=key=>Object.hasOwn(row,key)?t('explicit')+' '+row[key]:t('inherited');html+='<tr><td>'+i+'</td><td>'+e(leaf('start'))+'</td><td>'+e(leaf('end'))+'</td></tr>';}
 html+='</tbody></table>';
 }else if(widths.kind==='perVertex')html+='<p>'+e(JSON.stringify(widths.widths))+'</p>';
 return html;
}
export function effectiveWidthPresentation(widths,language){
 if(widths?.kind!=='defaulted')return '';
 const t=key=>translate(language,'width.'+key),e=escape;
 return '<h3>'+e(t('derived'))+'</h3><h4>'+e(t('effective'))+'</h4><table><thead><tr><th>#</th><th>'+e(t('start'))+'</th><th>'+e(t('end'))+'</th></tr></thead><tbody>'+(widths.overrides||[]).map((row,i)=>'<tr><td>'+i+'</td><td>'+e(row.start??widths.defaultStartWidth)+'</td><td>'+e(row.end??widths.defaultEndWidth)+'</td></tr>').join('')+'</tbody></table>';
}
