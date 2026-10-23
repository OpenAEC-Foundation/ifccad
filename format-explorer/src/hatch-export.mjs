// Rust supplies drawing-specific qualification. The UI only selects and presents it.
export function hatchExportState(presentation,{format,version,lossPolicy='allow',stale=false}={}){
 const restrictions=presentation?.hatchExportPreflight?.find(e=>e.format===format&&e.version===version)?.restrictions??[];
 return {restrictions,blocked:stale||restrictions.some(r=>r.policyEffect==='block'||lossPolicy==='reject'&&r.policyEffect==='loss')};
}
const messages={
 HATCH_PREFLIGHT_UNAVAILABLE:'hatchRestrictionCheck',
 HATCH_RECIPE_PREFLIGHT_CHECK:'hatchRestrictionRecipeCheck',
 HATCH_TARGET_FEATURE_UNSUPPORTED:'hatchRestrictionGradientVersion',
 TARGET_CODEC_HATCH_ASSOCIATION_STATE_LOSS:'hatchRestrictionUnbound',
 TARGET_CODEC_HATCH_GRADIENT_STATE_LOSS:'hatchRestrictionInactiveGradient',
 TARGET_CODEC_HATCH_DESCRIPTION_LOSS:'hatchRestrictionDescription',
 TARGET_CODEC_HATCH_CLASSIC_SETTINGS_LOSS:'hatchRestrictionClassic',
 TARGET_CODEC_HATCH_BOUNDARY_CONSTRUCTION_LOSS:'hatchRestrictionPixel',
 'hatch-classic-settings-loss':'hatchRestrictionRecipe',
 'hatch-boundary-construction-loss':'hatchRestrictionConstruction',
};
export function showHatchExportRestrictions(note,button,presentation,selection,translate){
 const state=hatchExportState(presentation,selection);
 note.hidden=state.restrictions.length===0;
 const groups=new Map();
 for(const restriction of state.restrictions){
  const reason=messages[restriction.code]?translate(messages[restriction.code]):restriction.message;
  const key=JSON.stringify([reason,restriction.policyEffect]);
  const group=groups.get(key)||{reason,policyEffect:restriction.policyEffect,count:0};
  group.count++;groups.set(key,group);
 }
 note.textContent=Array.from(groups.values(),r=>translate('hatchRestriction',{format:selection.format.toUpperCase(),version:selection.version,reason:r.count>1?translate('hatchRestrictionRepeated',{reason:r.reason,count:r.count}):r.reason,effect:translate(r.policyEffect==='block'?'hatchExportBlocked':r.policyEffect==='check'?'hatchExportCheck':selection.lossPolicy==='reject'?'hatchExportRejected':'hatchExportLoss')})).join('\n');
 button.disabled=selection.unavailable===true||state.blocked;
 return state;
}
