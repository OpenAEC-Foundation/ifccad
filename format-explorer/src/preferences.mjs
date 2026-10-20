const key='openaec.format-explorer.preferences.v1';
export function detectLanguage(languages){
 for(const value of Array.isArray(languages)?languages:[languages]){
  const language=typeof value==='string'?value.trim().toLowerCase().split('-')[0]:'';
  if(['nl','en'].includes(language))return language;
 }
 return 'en';
}
export function loadPreferences(storage,language='en',dark=false){
 let saved;try{saved=JSON.parse(storage?.getItem(key)||'null');}catch{}
 const languageChoice=['nl','en'].includes(saved?.language)?saved.language:'auto';
 return {language:languageChoice==='auto'?detectLanguage(language):languageChoice,languageChoice,theme:['light','dark'].includes(saved?.theme)?saved.theme:dark?'dark':'light'};
}
export function savePreferences(storage,preferences){try{storage?.setItem(key,JSON.stringify({language:preferences.languageChoice??preferences.language,theme:preferences.theme}));}catch{}}
