/** Only typed entity streams within one drawing resource form an accordion. */
export function expandExclusiveStream(model,collapsed,id,selected){
 const opened=model.byId.get(id);
 if(opened?.kind!=='collection'||opened.domain!=='ifcdr'||opened.concept)return selected;
 let owner=model.byId.get(selected);
 while(owner?.ownerId)owner=model.byId.get(owner.ownerId);
 const activeCollection=owner?.collectionId;
 for(const node of model.byId.values()){
  if(node.id===id||node.kind!=='collection'||node.domain!=='ifcdr'||node.concept||node.resourceId!==opened.resourceId)continue;
  collapsed.add(node.id);
  if(activeCollection===node.id)selected=id;
 }
 collapsed.delete(id);
 return selected;
}
