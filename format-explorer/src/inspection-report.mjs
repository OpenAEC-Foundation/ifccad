import {translate} from './i18n.mjs';
/** Keep the two conversion boundaries distinct; downloads are not diagnostics. */
export function inspectionReport(input={},cadResult){
 const exported=cadResult?.export,conversion=cadResult?.conversion;
 // CAD downloads are published only after codec readback and version checks.
 // Older result payloads may have only basic CAD readback evidence.
 const readback=!exported?.fileCheck&&cadResult?.failure===null&&exported?.download&&['dxf','dwg'].includes(exported.format)?{cadReadback:true,versionMatched:true,version:exported.requestedVersion}:undefined;
 return {
  input:{preview:input?.preview?{backend:input.preview.backend,rendering:input.preview.rendering,diagnostics:[...(input.preview.diagnostics||[]),...(input.preview.failure?[{code:'IFCX_VIEWER_PROJECTION_FAILED',message:input.preview.failure.message}]:[])]}:undefined,reader:input?.reader,validation:input?.validation,conversion:input?.conversion,failure:input?.failure},
  output:{restoration:conversion&&Object.hasOwn(conversion,'restoration')?conversion.restoration:conversion,failure:cadResult?.failure,readback,
   export:exported?{format:exported.format,requestedVersion:exported.requestedVersion,options:exported.options,geometry:exported.geometry,geometryAssessment:exported.geometryAssessment,text:exported.text,diagnostics:exported.diagnostics,fileCheck:exported.fileCheck}:undefined},
 };
}

export function conversionDirections(input,cadResult,settings={}){
 const cad=input?.source?.name?.match(/\.(dwg|dxf)$/i)?.[1]?.toUpperCase(),native='IFCCAD';
 const format=cadResult?.export?.format||settings?.format||'dxf',version=cadResult?.export?.requestedVersion??settings?.version;
 return {input:input?(cad?`${cad} → ${native}`:native):'',output:input?`${native} → ${format.toUpperCase()}`:'',version,inputConverted:!!cad};
}

export function filledEvidenceSummary(assessment,language){
 const rows=assessment?.filledSurfaces||[];if(!rows.length)return '';
 return [translate(language,'filled.positions'),translate(language,'filled.summary',{proven:rows.filter(r=>r.surfaceCoverage==='Proven').length,unassessed:rows.filter(r=>r.surfaceCoverage!=='Proven').length}),translate(language,'filled.displayUnassessed')].join(' ');
}

export function dimensionGraphicsSummary(assessment,language){
 const rows=assessment?.entries||[];if(!rows.some(r=>r.graphicsCoverage))return '';
 const count=state=>rows.filter(r=>r.graphicsCoverage?.state===state).length;
 return translate(language,'dimension.graphicsTransfer',{transferred:count('transferred'),empty:count('explicitEmpty'),absent:count('absent'),omitted:count('notTransferred')});
}

export function polylineWidthEvidenceSummary(assessment,language){
 const rows=assessment?.polylineWidths||[];if(!rows.length)return '';
 return [translate(language,'width.band'),translate(language,'width.summary',{proven:rows.flatMap(r=>r.segments||[]).filter(s=>s.bandCoverage==='Proven').length,unassessed:rows.flatMap(r=>r.segments||[]).filter(s=>s.bandCoverage==='Unassessed').length}),translate(language,'width.display')].join(' ');
}
