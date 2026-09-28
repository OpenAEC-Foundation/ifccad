/** Bounded inspector windows. Page indexes are presentation state only. */
const escape=value=>String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
export function pageWindow(model,id,total,size=10){
 model.inspectorPages??=new Map();
 const pages=Math.max(1,Math.ceil(total/size));
 const page=Math.max(0,Math.min(pages-1,model.inspectorPages.get(id)||0));
 return {page,pages,start:page*size,end:Math.min(total,(page+1)*size),total};
}
export function collectionWindow(model,id){
 const c=model.paging.collections.get(id);if(!c)return null;
 return {page:Math.floor(c.start/c.pageSize),pages:Math.max(1,Math.ceil(c.items.length/c.pageSize)),start:c.start,end:c.end||Math.min(c.pageSize,c.items.length),total:c.items.length};
}
export function pager(id,w,kind='inspector'){
 if(w.pages<=1)return '';
 const attrs=`data-page-id="${escape(id)}" data-page-kind="${kind}"`;
 return `<nav class="collection-pager" aria-label="Bladeren door verzameling"><button ${attrs} data-page="${w.page-1}" ${w.page===0?'disabled':''} aria-label="Vorige pagina">←</button><span class="page-range" data-no-i18n>${w.start+1}–${w.end} / ${w.total}</span><button ${attrs} data-page="${w.page+1}" ${w.page===w.pages-1?'disabled':''} aria-label="Volgende pagina">→</button><label>Pagina <input type="number" min="1" max="${w.pages}" value="${w.page+1}" ${attrs} aria-label="Ga naar pagina"></label><span data-no-i18n>/ ${w.pages}</span><button ${attrs} data-page-go>Ga</button></nav>`;
}
export function columnTable(node,model){
 const stream=node.raw,columns=Object.keys(stream).filter(k=>k!=='count');
 const pools=columns.filter(k=>['x','y','z','bulge'].includes(k)&&Object.hasOwn(stream,'vertexOffset')&&Object.hasOwn(stream,'vertexCount'));
 function section(keys,key,total,title,shared=false){
  if(!keys.length)return '';
  const w=shared?collectionWindow(model,node.id):pageWindow(model,key,total),indexes=Array.from({length:w.end-w.start},(_,i)=>w.start+i);
  return `<div class="structure-heading"><h3>${title}</h3><span>${total} rijen</span></div>${pager(key,w,shared?'graph':'inspector')}${shared&&total>10?'<p class="small-note">Graph en kolommen tonen dezelfde 10 items per pagina.</p>':''}<div class="column-map" tabindex="0" role="region" aria-label="Kolomstructuur"><div class="column-row"><code>index</code><div class="column-values">${indexes.map(i=>`<span class="column-cell column-index">${i}</span>`).join('')}</div></div>${keys.map(k=>`<div class="column-row"><code>${escape(k)}</code><div class="column-values">${Array.isArray(stream[k])?stream[k].slice(w.start,w.end).map(v=>{
   const value=typeof v==='object'?JSON.stringify(v):String(v),short=value.slice(0,160)+(value.length>160?'…':'');
   return `<span class="column-cell${k==='entityId'?' id-cell':''}" title="${escape(value.slice(0,2000))}">${k==='entityId'?`<button data-select="${escape('entity:'+node.resourceId+':'+v)}" aria-label="Bekijk entiteit ${escape(v)}">${escape(value)}</button>`:escape(short)}</span>`;
  }).join(''):escape(stream[k])}</div></div>`).join('')}</div>`;
 }
 return section(columns.filter(k=>!pools.includes(k)),node.id,stream.count,'Kolomstructuur',true)+
  section(pools,node.id+':pool',Math.max(0,...pools.map(k=>stream[k].length)),'Puntenpool')+
  (pools.length?`<p class="small-note">vertexOffset en vertexCount kiezen een bereik in de gedeelde ${pools.filter(k=>k!=='bulge').join('/')}-puntenpool.${pools.includes('bulge')?' De bulge-kolom volgt dezelfde vertexindex.':''}</p>`:'');
}
export function collectionBrowser(node,model){
 const c=model.paging.collections.get(node.id);if(!c)return '';
 const w=collectionWindow(model,node.id);
 return `<div class="relation-list collection-list">${c.items.slice(w.start,w.end).map((item,i)=>{
  const id=c.idFor(item,w.start+i),label=model.byId.get(id)?.subtitle||item.attributes?.name||item.name||(item.kind!==undefined&&item.id!==undefined?`scopeId ${item.id}`:null)||item.path||item.sourceType||id;
  return `<button class="relation" data-select="${escape(id)}"><span><small>${w.start+i}</small>${escape(label)}</span><i aria-hidden="true">↗</i></button>`;
 }).join('')}</div>`;
}
export function graphPager(node,model){
 const c=model.paging.collections.get(node.id);if(!c||c.items.length<=c.pageSize)return '';
 return '<div class="structure-heading"><h3>Items in de graph</h3></div>'+pager(node.id,collectionWindow(model,node.id),'graph');
}
