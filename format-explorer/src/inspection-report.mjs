/** One central report; downloaded file bytes are not diagnostic information. */
export function inspectionReport(input={},cadResult){
 const output=cadResult||input,exported=output?.export;
 return {reader:input?.reader,validation:input?.validation,conversion:cadResult?.conversion??input?.conversion,failure:output?.failure,
  export:exported?{format:exported.format,requestedVersion:exported.requestedVersion,options:exported.options,geometry:exported.geometry,geometryAssessment:exported.geometryAssessment,diagnostics:exported.diagnostics,fileCheck:exported.fileCheck}:undefined};
}
