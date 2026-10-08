export function normalizeTolerance(input={mode:'default'}){
 if(!['default','exact','custom'].includes(input.mode))throw Error('Invalid tolerance mode');
 if(input.mode!=='custom')return {mode:input.mode};
 if(!['mm','m','drawing'].includes(input.unit)||input.value==null||String(input.value).trim()===''||!Number.isFinite(Number(input.value))||Number(input.value)<0)throw Error('Tolerance must be finite, nonnegative and have an explicit unit');
 const result={mode:'custom',value:Number(input.value),unit:input.unit};
 if(input.coordinateFallback!==undefined){
  const value=input.coordinateFallback;
  if(input.unit==='drawing'||value==null||String(value).trim()===''||!Number.isFinite(Number(value))||Number(value)<0)throw Error('Coordinate fallback requires a physical tolerance and a finite nonnegative value');
  result.coordinateFallback=Number(value);
 }
 return result;
}
export function conversionOptions(tolerance){return {tolerance:normalizeTolerance(tolerance)};}
