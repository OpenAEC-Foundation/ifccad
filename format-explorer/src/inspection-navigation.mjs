import {hatchValues,hatchSummary} from './hatch-view.mjs';
// A UI projection: native nodes, identities and source ordering are unchanged.
const projections=new WeakMap();
export function cadSelectionForNode(model,key){
 const node=model.nodes.get(key);if(!node)return null;
 if(node.type==='layout'){
  const values=node.values['ifccad::layout']||node.values;
  const layout=String(values.kind).toLowerCase()==='model'?'Model':values.name;
  return typeof layout==='string'&&layout?{kind:'layout',layout}:null;
 }
 return (model.format==='ifccad'?!!node.values['ifccad::entity']:key.startsWith('entity:'))?{kind:'element',key}:null;
}
export function inspectionNavigation(model,{view='drawing',grouping='type'}={}){
 let cached=projections.get(model);if(!cached){cached=new Map();projections.set(model,cached);}
 const signature=view+':'+grouping;if(cached.has(signature))return cached.get(signature);
 const nodes=new Map(model.nodes),children=new Map(),parents=new Map();
 const flatten=keys=>keys.flatMap(key=>key.startsWith('group:types:')&&nodes.get(key)?.children.length===1?nodes.get(key).children:[key]);
 for(const node of nodes.values())children.set(node.key,flatten(view==='drawing'&&grouping!=='order'?(node.groupedChildren||node.children):node.children));
 let roots=flatten(view==='nodes'?model.nodeRoots:view==='storage'?model.storageRoots:model.roots);
 if(roots.length>10&&view!=='drawing'){
  const key='inspection:roots';nodes.set(key,{key,title:view==='nodes'?'nodes':'storage',type:'group',children:roots,values:{},incoming:[],outgoing:[]});children.set(key,roots);roots=[key];
 }
 const visited=new Set(),queue=roots.map(key=>[key,null]);
 for(let i=0;i<queue.length;i++){const [key,parent]=queue[i];if(visited.has(key))continue;visited.add(key);parents.set(key,parent);for(const child of children.get(key)||[])queue.push([child,key]);}
 const path=key=>{const result=[];while(key!=null&&parents.has(key)){result.unshift(key);key=parents.get(key);}return result;};
 const collection=key=>{const node=nodes.get(key),keys=children.get(key)||[];let trail=path(key);if(!trail.length&&key.startsWith('group:types:')){const child=nodes.get(keys[0]);trail=[...path(child?.owner),key];}return node&&keys.length?{key,node,keys,path:trail}:null;};
 const isLarge=key=>(children.get(key)?.length||0)>10&&!(view==='drawing'&&grouping!=='order'&&nodes.get(key)?.groupedChildren);
 const forSelection=key=>{const ancestors=path(key);for(let i=ancestors.length-1;i>=0;i--)if(isLarge(ancestors[i]))return collection(ancestors[i]);return null;};
 const activeCollection=(key,alreadyOpen=false)=>{
  const large=forSelection(key);if(large||!alreadyOpen)return large;
  if(collection(key))return collection(key);
  const node=nodes.get(key),typeKey='group:types:'+node?.owner+':'+node?.type;
  if(view==='drawing'&&grouping!=='order'&&collection(typeKey))return collection(typeKey);
  const trail=path(key);for(let i=trail.length-2;i>=0;i--)if(collection(trail[i]))return collection(trail[i]);return null;
 };
 const navigation={nodes,roots,children,path,collection,isLarge,forSelection,activeCollection};cached.set(signature,navigation);return navigation;
}

export function listWindow(length,scrollTop=0,height=400,rowHeight=56,overscan=4){
 const first=Math.max(0,Math.min(Math.max(0,length-1),Math.floor(scrollTop/rowHeight)));
 const start=Math.max(0,first-overscan),end=Math.min(length,first+Math.ceil(height/rowHeight)+overscan);
 return {start,end,top:start*rowHeight,bottom:Math.max(0,(length-end)*rowHeight)};
}

export function collectionEntries(model,keys,title,language='en'){
 return keys.flatMap(key=>{
  const node=model.nodes.get(key);if(!node)return [];
  const edge=node.outgoing.find(e=>e.kind==='layer'),layerKey=edge?.target||'',layer=model.nodes.get(layerKey)?.title||layerKey;
  const geometry=node.values.geometry||Object.entries(node.values).find(([k])=>k.startsWith('ifccad::geom::')&&k!=='ifccad::geom::placement')?.[1];
  const point=value=>Array.isArray(value)?value.map(v=>typeof v==='number'?Number(v.toPrecision(6)):v).join(', '):'';
  const hatch=hatchValues(node);
  const summary=hatch?hatchSummary(hatch,language):geometry?.start&&geometry?.end?point(geometry.start)+' → '+point(geometry.end):geometry?.radius!=null?'r = '+geometry.radius:geometry?.text||node.values.text||'';
  const label=title(node);return [{key,label,layerKey,layer,summary:String(summary),search:[key,label,node.type,layer,summary].join(' ').toLocaleLowerCase()}];
 });
}
export function filterCollection(entries,{query='',layer=''}={}){
 const terms=query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
 return entries.filter(entry=>(!layer||entry.layerKey===layer)&&terms.every(term=>entry.search.includes(term)));
}
