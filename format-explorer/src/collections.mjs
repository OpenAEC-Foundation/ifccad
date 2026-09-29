/** Presentation-only paging; semantic collections and source data remain intact. */
export function createCollections({add,edge,byId,defaultCollapsed,pageSize=10}){
 const collections=new Map(),targets=new Map();
 function register(id,items,create,idFor){
  const state={id,items,create,idFor,start:0,end:0,graphStart:0,pageSize,loaded:new Set()};collections.set(id,state);
  items.forEach((item,i)=>targets.set(idFor(item,i),{state,index:i}));
  if(items.length<=pageSize)setPage(id,0);else{byId.get(id).expandable=true;defaultCollapsed.add(id);}
 }
 function ensure(s,i){if(s.loaded.has(i))return;s.loaded.add(i);s.create(s.items[i],i);const node=byId.get(s.idFor(s.items[i],i));if(node){node.collectionId=s.id;node.collectionIndex=i;}}
 function ensureGraph(s){for(let i=s.graphStart;i<Math.min(s.graphStart+3,s.items.length);i++)ensure(s,i);}
 function setPage(id,page,preserveGraph=false){const s=collections.get(id);if(!s||!Number.isFinite(page))return;
  s.start=Math.max(0,Math.min(Math.floor(page),Math.max(0,Math.ceil(s.items.length/pageSize)-1)))*pageSize;
  s.end=Math.min(s.start+pageSize,s.items.length);for(let i=s.start;i<s.end;i++)ensure(s,i);
  if(!preserveGraph)s.graphStart=Math.min(s.start,Math.max(0,s.items.length-3));ensureGraph(s);
  const moreId='more:'+id;
  if(s.items.length>pageSize){if(!byId.has(moreId)){const p=byId.get(id);add({id:moreId,label:'Volgende groep',subtitle:'',kind:'more',domain:p.domain,parentId:id,fieldIndex:pageSize,x:p.x+270,y:p.y+pageSize*140,raw:null});edge(id,moreId,'bladeren',true);}
   Object.assign(byId.get(moreId),{label:s.end<s.items.length?'Volgende groep':'Eerste groep',subtitle:(s.start+1)+'–'+s.end+' / '+s.items.length});}
 }
 function materialize(id){if(id.startsWith('more:')){const s=collections.get(id.slice(5));if(s)setPage(s.id,s.end<s.items.length?s.start/pageSize+1:0);return false;}
  const target=targets.get(id);if(target){const s=target.state;if(!isVisible(id))setPage(s.id,Math.floor(target.index/pageSize),true);if(target.index<s.graphStart||target.index>=s.graphStart+3){s.graphStart=Math.max(0,Math.min(target.index-1,s.items.length-3));ensureGraph(s);}}return byId.has(id);}
 function isVisible(id){const t=targets.get(id);return !t||(t.index>=t.state.start&&t.index<t.state.end);}
 function isGraphVisible(id){const t=targets.get(id);return !t||(t.index>=t.state.graphStart&&t.index<t.state.graphStart+3);}
 function graphWindow(id){const s=collections.get(id);return s?{start:s.graphStart,end:Math.min(s.graphStart+3,s.items.length),total:s.items.length}:null;}
 function scrollGraph(id,delta){const s=collections.get(id);if(!s||!Number.isFinite(delta))return false;
  const next=Math.max(0,Math.min(s.graphStart+Math.trunc(delta),Math.max(0,s.items.length-3)));
  if(next===s.graphStart)return false;s.graphStart=next;
  const page=Math.floor(next/pageSize);if(s.start/pageSize!==page)setPage(id,page,true);else ensureGraph(s);
  return true;
 }
 return {register,materialize,setPage,isVisible,isGraphVisible,graphWindow,scrollGraph,pageSize,expand(id){const s=collections.get(id);if(s&&s.end===0)setPage(id,0);},has:id=>targets.has(id),collections};
}
