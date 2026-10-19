// Compact IFCCAD graph recipes for navigation/rendering tests. These deliberately
// omit format validation; production readers are exercised by Rust and WASM tests.
import {createInspection as inspect} from '../../src/inspection-model.mjs';
export function createInspection(result,source='') {
 const p=result.presentation;
 if(!p||p.graph)return inspect(result,source);
 const data=[],add=(path,role,value,children)=>data.push({path,attributes:{['ifccad::'+role]:value},...(children?{children}:{})});
 const ids=values=>Object.fromEntries((values||[]).map((id,i)=>[String(i),'entity:'+id]));
 for(const row of p.layers||[])add('layer:'+row.id,'layer',row);
 for(const row of p.symbolSources||[])add('symbolSource:'+row.id,'symbolSource',row);
 for(const row of p.linePatterns||[])add('pattern:'+row.id,'linePattern',row);
 for(const row of p.layouts||[])add('layout:'+row.id,'layout',{...row,kind:String(row.kind).toLowerCase()==='paper'?'Paper':'Model'},ids(p.scopes?.find(s=>s.id===row.scopeId)?.entities??(p.entities||[]).map(e=>e.id)));
 for(const row of p.entities||[]){const g=row.geometry||{},path='entity:'+row.id;data.push({path,attributes:{'ifccad::entity':{id:row.id,layer:row.layerId==null?undefined:'layer:'+row.layerId,appearance:row.appearance},['ifccad::geom::'+(g.type==='line'?'lineSegment':g.type||'lineSegment')]:g}});}
 const boundsCompleteness=(p.boundsCompleteness||[]).map(a=>({...a,scopePath:'layout:'+(p.layouts?.find(l=>l.scopeId===a.scopeId)?.id??a.scopeId)}));
 return inspect({...result,presentation:{...p,graph:{data},boundsCompleteness}},source);
}
