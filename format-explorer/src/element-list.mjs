import {collectionEntries,filterCollection,listWindow} from './inspection-navigation.mjs';
import {itemTitle,escapeHtml as e} from './inspection-view.mjs';

// Only visible rows are mounted. Scrolling never rebuilds the structure or inspector.
export function initializeElementList({onSelect,t,getLanguage}){
 const $=id=>document.getElementById(id),viewport=$('element-list'),rows=$('element-rows'),search=$('element-search'),layer=$('element-layer');
 let collection,entries=[],filtered=[],selection,language,model;
 const resize=new ResizeObserver(()=>draw());resize.observe(viewport);
 function draw(){
  if(!collection||viewport.hidden)return;
  const focusKey=document.activeElement?.closest('[data-list-key]')?.dataset.listKey;
  const {start,end,top,bottom}=listWindow(filtered.length,viewport.scrollTop,viewport.clientHeight||400);
  rows.style.paddingTop=top+'px';rows.style.paddingBottom=bottom+'px';
  const focus=filtered.some(entry=>entry.key===selection)?selection:filtered[0]?.key;
  rows.innerHTML=filtered.slice(start,end).map((entry,index)=>'<button role="option" class="element-row'+(entry.key===selection?' selected':'')+'" data-list-key="'+e(entry.key)+'" aria-selected="'+(entry.key===selection)+'" aria-posinset="'+(start+index+1)+'" aria-setsize="'+filtered.length+'" tabindex="'+(entry.key===focus?'0':'-1')+'"><span class="element-label">'+e(entry.label)+'</span><span class="element-description">'+e([entry.layer,entry.summary].filter(Boolean).join(' · ')||entry.key)+'</span></button>').join('');
  $('element-empty').hidden=filtered.length!==0;$('element-count').textContent=t('collectionCount',{shown:filtered.length,total:entries.length});
  if(focusKey)Array.from(rows.querySelectorAll('[data-list-key]')).find(row=>row.dataset.listKey===focusKey)?.focus({preventScroll:true});
 }
 function filter(){filtered=filterCollection(entries,{query:search.value,layer:layer.value});viewport.scrollTop=0;draw();}
 function reveal(key){
  let index=filtered.findIndex(entry=>entry.key===key);if(index<0&&entries.some(entry=>entry.key===key)){search.value='';layer.value='';filtered=entries;index=filtered.findIndex(entry=>entry.key===key);}
  if(index>=0){const top=index*56,bottom=top+56;if(top<viewport.scrollTop)viewport.scrollTop=top;else if(bottom>viewport.scrollTop+viewport.clientHeight)viewport.scrollTop=Math.max(0,bottom-viewport.clientHeight);}
  draw();
 }
 viewport.addEventListener('scroll',draw,{passive:true});search.oninput=filter;layer.onchange=filter;
 rows.onclick=event=>{const row=event.target.closest('[data-list-key]');if(row)onSelect(row.dataset.listKey);};
 viewport.onkeydown=event=>{
  if(!['ArrowDown','ArrowUp','Home','End','PageDown','PageUp'].includes(event.key)||!filtered.length)return;
  event.preventDefault();let index=filtered.findIndex(entry=>entry.key===(event.target.closest('[data-list-key]')?.dataset.listKey||selection));
  const page=Math.max(1,Math.floor(viewport.clientHeight/56));index=event.key==='Home'?0:event.key==='End'?filtered.length-1:Math.max(0,Math.min(filtered.length-1,index+({ArrowDown:1,ArrowUp:-1,PageDown:page,PageUp:-page}[event.key])));
  const key=filtered[index].key;onSelect(key);reveal(key);Array.from(rows.querySelectorAll('[data-list-key]')).find(row=>row.dataset.listKey===key)?.focus({preventScroll:true});
 };
 return {
  show(next,navigation,nextModel,key,{revealSelection=false}={}){
   const nextLanguage=getLanguage(),changed=collection?.key!==next?.key||model!==nextModel;
   if(!next){collection=null;entries=[];filtered=[];rows.replaceChildren();rows.style.paddingTop=rows.style.paddingBottom='0px';viewport.scrollTop=0;$('element-path').textContent=t('selectCollection');$('element-count').textContent='';$('element-empty').hidden=true;search.value='';search.disabled=true;layer.hidden=true;return;}
   search.disabled=false;
   if(changed){search.value='';layer.value='';viewport.scrollTop=0;}
   collection=next;model=nextModel;selection=key;
   if(changed||language!==nextLanguage){
    entries=collectionEntries(navigation,next.keys,node=>itemTitle(node,nextLanguage));language=nextLanguage;
    const layers=new Map(entries.filter(entry=>entry.layerKey).map(entry=>[entry.layerKey,entry.layer]));const previous=layer.value;
    layer.innerHTML='<option value="">'+e(t('allLayers'))+'</option>'+Array.from(layers,([key,label])=>'<option value="'+e(key)+'">'+e(label)+'</option>').join('');layer.value=previous;layer.hidden=layers.size===0;
   }
   search.placeholder=t('searchCollection');search.setAttribute('aria-label',t('searchCollection'));layer.setAttribute('aria-label',t('layers'));
   $('element-path').textContent=next.path.map(key=>itemTitle(navigation.nodes.get(key),language)).join(' › ');$('element-path').title=$('element-path').textContent;
   viewport.setAttribute('aria-label',$('element-path').textContent);filtered=filterCollection(entries,{query:search.value,layer:layer.value});
   if(revealSelection)reveal(key);else draw();
  },reset(){collection=null;model=null;entries=[];filtered=[];search.value='';layer.value='';viewport.scrollTop=0;}
 };
}
