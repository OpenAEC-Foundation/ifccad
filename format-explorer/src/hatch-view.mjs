import {translate} from './i18n.mjs';
export function hatchValues(node){return node?.values?.['ifccad::hatch']||(node?.values?.geometry?.type==='hatch'?node.values.geometry:null);}
export function hatchSummary(value,language='en'){
 const t=(key,values)=>translate(language,key,values);
 const fill=value.fill?.kind==='solid'?t('solidFill'):value.fill?.kind==='linePattern'?t('patternFill')+' · '+t('hatchFamilies',{count:value.fill.families?.length||0}):value.fill?.kind||'—';
 return t('hatchSummary',{fill,count:value.loops?.length||0,rule:t('hatchRule.'+value.areaRule)});
}
