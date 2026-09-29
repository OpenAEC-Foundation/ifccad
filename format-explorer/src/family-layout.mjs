/** Deterministic positions for the visible ownership tree; no packet coordinates are changed. */
export function familyLayout(boxes,edges,roots){
 const parents=new Map(),children=new Map();
 const priority=id=>{const n=boxes.get(id);return ['group','workspace-group','collection'].includes(n?.kind)?3:n?.kind==='Drawing'?2:1;};
 for(const edge of edges){
  if(!edge.structural||!boxes.has(edge.source)||!boxes.has(edge.target))continue;
  const previous=parents.get(edge.target);
  if(!previous||priority(edge.source)>priority(previous))parents.set(edge.target,edge.source);
 }
 for(const [child,parent] of parents){if(!children.has(parent))children.set(parent,[]);children.get(parent).push(child);}
 const branchOrder=id=>{
  const n=boxes.get(id);
  if(n?.kind==='group'&&id.endsWith(':Layouts'))return 0;
  if(n?.kind==='group'&&(id.endsWith(':Layers')||id==='view:definitions:Layer'))return 1;
  if(n?.kind==='group'&&(id.endsWith(':Appearances')||id==='view:definitions:Appearance'))return 2;
  if(n?.kind==='DrawingRepresentation')return 3;
  if(id.endsWith(':scopeTable'))return 0;
  if(id.endsWith(':blockDefinitionTable'))return 1;
  if(id.endsWith(':streams'))return 2;
  if(id.endsWith(':workspace'))return 3;
  return 10;
 };
 for(const [parent,kids] of children)if(['Drawing','drawing-resource'].includes(boxes.get(parent)?.kind))kids.sort((a,b)=>branchOrder(a)-branchOrder(b));
 const placed=new Set();
 const visit=(id,x,y)=>{
  if(placed.has(id)||!boxes.has(id))return;
  placed.add(id);const box=boxes.get(id);box.x=x;box.y=y;
  const kids=children.get(id)||[],fan=box.kind==='Drawing';
  kids.forEach((child,i)=>{
   const target=boxes.get(child),below=['DrawingSet','group','workspace-group'].includes(box.kind);
   let nx,ny;
   if(box.kind==='DrawingSet'){
    if(target?.kind==='PreservationRepresentation'){nx=x+850;ny=y;}
    else{const drawingIndex=kids.slice(0,i).filter(key=>boxes.get(key)?.kind==='Drawing').length;nx=x+drawingIndex*1200;ny=y+135;}
   }
   else if(box.kind==='drawing-resource'){nx=x+i*300;ny=y+130;}
   else if(fan){nx=x+(i-(kids.length-1)/2)*300;ny=y+130;}
   else if(['DrawingRepresentation','PreservationRepresentation'].includes(box.kind)){nx=x;ny=y+box.height+35;}
   else if(below){nx=x;ny=y+box.height+70+i*145;}
   else if(box.kind==='collection'||box.kind==='entity'||box.kind==='field'){
    nx=x+365;ny=y+i*145;
   } else {nx=x+365;ny=y+i*160;}
   // A display window occupies fixed slots independent of absolute item index.
   if(target?.collectionId&&['group','workspace-group','collection'].includes(box.kind))ny=y+(box.kind==='collection'?0:box.height+70)+i*145;
   visit(child,nx,ny);
  });
 };
 let rootIndex=0;
 const packageRoot=roots.find(id=>boxes.get(id)?.kind==='DrawingSet');
 for(const id of roots)if(boxes.has(id)&&!placed.has(id)){
  if(packageRoot&&boxes.get(id).kind==='PackageWorkspaceState')continue;
  visit(id,200+rootIndex*420,120+rootIndex*540);rootIndex++;
 }
 if(packageRoot&&placed.has(packageRoot))for(const id of roots)if(boxes.get(id)?.kind==='PackageWorkspaceState'){
  const anchor=boxes.get(packageRoot);
  visit(id,anchor.x+400,anchor.y);
 }
 for(const id of boxes.keys())if(!placed.has(id)){visit(id,200+rootIndex*420,120+rootIndex*540);rootIndex++;}
 return boxes;
}

/** Orthogonal shared trunk for a fan of structural children. */
export function branchBusY(parent,children){
 const bottom=parent.y+parent.height,top=Math.min(...children.map(child=>child.y));
 return bottom+Math.max(4,Math.min(28,(top-bottom)/2));
}

export function branchRoute(parent,child,busY){
 const sx=parent.x+parent.width/2,sy=parent.y+parent.height;
 const tx=child.x+child.width/2,ty=child.y;
 return `M${sx} ${sy} L${sx} ${busY} L${tx} ${busY} L${tx} ${ty}`;
}
