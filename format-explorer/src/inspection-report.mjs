/** Keep the two conversion boundaries distinct; downloads are not diagnostics. */
export function inspectionReport(input={},cadResult){
 const exported=cadResult?.export,conversion=cadResult?.conversion;
 // CAD downloads are published only after codec readback and version checks.
 // OCDraw currently has no additional native fileCheck payload.
 const readback=!exported?.fileCheck&&cadResult?.failure===null&&exported?.download&&['dxf','dwg'].includes(exported.format)?{cadReadback:true,versionMatched:true,version:exported.requestedVersion}:undefined;
 return {
  input:{reader:input?.reader,validation:input?.validation,conversion:input?.conversion,failure:input?.failure},
  output:{restoration:conversion&&Object.hasOwn(conversion,'restoration')?conversion.restoration:conversion,failure:cadResult?.failure,readback,
   export:exported?{format:exported.format,requestedVersion:exported.requestedVersion,options:exported.options,geometry:exported.geometry,geometryAssessment:exported.geometryAssessment,text:exported.text,diagnostics:exported.diagnostics,fileCheck:exported.fileCheck}:undefined},
 };
}

export function conversionDirections(input,cadResult,settings={}){
 const cad=input?.source?.name?.match(/\.(dwg|dxf)$/i)?.[1]?.toUpperCase(),native=input?.inspectedFormat==='ocdraw'||input?.presentation?.format==='ocdraw'?'OCDraw':'IFCCAD';
 const format=cadResult?.export?.format||settings?.format||'dxf',version=cadResult?.export?.requestedVersion??settings?.version;
 return {input:input?(cad?`${cad} → ${native}`:native):'',output:input?`${native} → ${format.toUpperCase()}`:'',version,inputConverted:!!cad};
}
