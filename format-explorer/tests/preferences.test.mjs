import test from 'node:test';
import assert from 'node:assert/strict';
const preferences=await import('../src/preferences.mjs').catch(()=>({}));
test('preferences detect Dutch and dark appearance without stored choices',()=>{
 assert.equal(typeof preferences.loadPreferences,'function');
 assert.deepEqual(preferences.loadPreferences(null,'nl-NL',true),{language:'nl',languageChoice:'auto',theme:'dark'});
});
test('stored preferences are validated and unavailable storage does not block a session',()=>{
 assert.equal(typeof preferences.loadPreferences,'function');
 const storage={getItem(){return '{"language":"xx","theme":"invalid"}';},setItem(){throw Error('disabled');}};
 assert.deepEqual(preferences.loadPreferences(storage,'fr',false),{language:'en',languageChoice:'auto',theme:'light'});
 assert.doesNotThrow(()=>preferences.savePreferences(storage,{language:'nl',theme:'dark'}));
 const corrupt={getItem(){throw Error('disabled');}};
 assert.deepEqual(preferences.loadPreferences(corrupt,'en',true),{language:'en',languageChoice:'auto',theme:'dark'});
});
test('explicit language and appearance survive reopening',()=>{
 assert.equal(typeof preferences.savePreferences,'function');
 const values=new Map(),storage={getItem:k=>values.get(k),setItem:(k,v)=>values.set(k,v)};
 preferences.savePreferences(storage,{language:'en',theme:'light'});
 assert.deepEqual(preferences.loadPreferences(storage,['nl-BE','en'],true),{language:'en',languageChoice:'en',theme:'light'});
});

test('automatic language uses the first supported browser preference or English',()=>{
 for(const [languages,language]of [[['fr-FR','nl-BE','en-US'],'nl'],[['en-GB','nl'],'en'],[['NL-nl'],'nl'],[['fr','de'],'en'],[[],'en'],[null,'en'],[['nld','english','en-US'],'en']]){
  assert.equal(preferences.loadPreferences(null,languages).language,language,JSON.stringify(languages));
 }
});

test('saving a theme keeps automatic language responsive to later browser preferences',()=>{
 const values=new Map(),storage={getItem:k=>values.get(k),setItem:(k,v)=>values.set(k,v)};
 const automatic=preferences.loadPreferences(storage,['nl-NL']);
 preferences.savePreferences(storage,{...automatic,theme:'dark'});
 assert.deepEqual(preferences.loadPreferences(storage,['en-US']),{language:'en',languageChoice:'auto',theme:'dark'});
 preferences.savePreferences(storage,{language:'nl',languageChoice:'nl',theme:'light'});
 assert.equal(preferences.loadPreferences(storage,['en-US']).language,'nl');
 preferences.savePreferences(storage,{language:'nl',languageChoice:'auto',theme:'light'});
 assert.deepEqual(preferences.loadPreferences(storage,['en-US']),{language:'en',languageChoice:'auto',theme:'light'});
});
