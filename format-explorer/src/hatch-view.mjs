import {translate} from './i18n.mjs';
export function hatchValues(node){return node?.values?.['ifccad::hatch']||(node?.values?.geometry?.type==='hatch'?node.values.geometry:null);}
export function hatchSummary(value,language='en'){
 const t=(key,values)=>translate(language,key,values);
 return t('hatchSummary',{fill:value.fill?.kind==='solid'?t('solidFill'):value.fill?.kind||'—',count:value.loops?.length||0,rule:t('hatchRule.'+value.areaRule)});
}
