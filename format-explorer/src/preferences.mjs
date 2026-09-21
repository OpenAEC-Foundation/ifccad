export const preferenceKey = 'ifccad-viewer.settings.v1';
const normalized = value => ({
  language:['auto','nl','en'].includes(value?.language)?value.language:'auto',
  appearance:['auto','light','dark'].includes(value?.appearance)?value.appearance:'auto',
});
export function resolveLanguage(preference, languages = []) {
  if(preference==='nl'||preference==='en')return preference;
  return languages.map(tag=>String(tag).toLowerCase().split(/[-_]/)[0]).find(tag=>tag==='nl'||tag==='en')||'en';
}
export function resolveTheme(preference, dark) {
  return preference==='light'||preference==='dark'?preference:dark?'dark':'light';
}
export function readPreferences(storage) {
  try{return normalized(JSON.parse(storage?.getItem(preferenceKey)||'null'));}catch{return normalized(null);}
}
export function savePreferences(storage, preferences) {
  try{if(!storage)return false;storage.setItem(preferenceKey,JSON.stringify(normalized(preferences)));return true;}catch{return false;}
}
