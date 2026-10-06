const key='openaec.format-explorer.preferences.v1';
export function loadPreferences(storage,language='en',dark=false){
 let saved;try{saved=JSON.parse(storage?.getItem(key)||'null');}catch{}
 return {language:['nl','en'].includes(saved?.language)?saved.language:String(language).toLowerCase().startsWith('nl')?'nl':'en',theme:['light','dark'].includes(saved?.theme)?saved.theme:dark?'dark':'light'};
}
export function savePreferences(storage,preferences){try{storage?.setItem(key,JSON.stringify({language:preferences.language,theme:preferences.theme}));}catch{}}
