// Application outcomes belong to a single inspection snapshot, never the drawing.
const statuses=new Set(['notChecked','found','missing','unreadable','substituted']);
const roles=new Set(['primary','bigFont','family','shx']);
const freeze=(snapshot,entries)=>Object.freeze({snapshot,entries:Object.freeze(entries.map(Object.freeze))});
export function createFontSymbolReport(requests,snapshot){
 if(!snapshot||typeof snapshot!=='object')throw Error('Font report requires an inspection snapshot');
 const paths=new Set();
 return freeze(snapshot,requests.map(request=>{
  if(typeof request.requestPath!=='string'||!request.requestPath||paths.has(request.requestPath)||!roles.has(request.fontRole)||typeof request.requestedName!=='string')throw Error('Invalid font request');
  paths.add(request.requestPath);
  return {requestPath:request.requestPath,fontRole:request.fontRole,requestedName:request.requestedName,status:'notChecked'};
 }));
}
export function applyFontSymbolResult(report,snapshot,outcome){
 if(report.snapshot!==snapshot)throw Error('Stale font resolution snapshot');
 const request=report.entries.find(row=>row.requestPath===outcome.requestPath);
 if(!request||request.fontRole!==outcome.fontRole||!statuses.has(outcome.status))throw Error('Unknown font request or outcome');
 if(outcome.status==='substituted'&&(typeof outcome.usedName!=='string'||!outcome.usedName.trim()))throw Error('Substitution requires the used source name');
 if(outcome.status==='notChecked'&&Object.hasOwn(outcome,'usedName'))throw Error('Unchecked request cannot have a used source');
 if(Object.hasOwn(outcome,'usedName')&&(typeof outcome.usedName!=='string'||!outcome.usedName.trim()))throw Error('Invalid used source name');
 const replacement={requestPath:request.requestPath,fontRole:request.fontRole,requestedName:request.requestedName,status:outcome.status};
 if(Object.hasOwn(outcome,'usedName'))replacement.usedName=outcome.usedName;
 if(typeof outcome.reason==='string')replacement.reason=outcome.reason;
 return freeze(snapshot,report.entries.map(row=>row===request?replacement:row));
}
