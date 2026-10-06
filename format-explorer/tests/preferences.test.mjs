import test from 'node:test';
import assert from 'node:assert/strict';
const preferences=await import('../src/preferences.mjs').catch(()=>({}));
test('preferences detect Dutch and dark appearance without stored choices',()=>{
 assert.equal(typeof preferences.loadPreferences,'function');
 assert.deepEqual(preferences.loadPreferences(null,'nl-NL',true),{language:'nl',theme:'dark'});
});
test('stored preferences are validated and unavailable storage does not block a session',()=>{
 assert.equal(typeof preferences.loadPreferences,'function');
 const storage={getItem(){return '{"language":"xx","theme":"invalid"}';},setItem(){throw Error('disabled');}};
 assert.deepEqual(preferences.loadPreferences(storage,'fr',false),{language:'en',theme:'light'});
 assert.doesNotThrow(()=>preferences.savePreferences(storage,{language:'nl',theme:'dark'}));
 const corrupt={getItem(){throw Error('disabled');}};
 assert.deepEqual(preferences.loadPreferences(corrupt,'en',true),{language:'en',theme:'dark'});
});
test('explicit language and appearance survive reopening',()=>{
 assert.equal(typeof preferences.savePreferences,'function');
 const values=new Map(),storage={getItem:k=>values.get(k),setItem:(k,v)=>values.set(k,v)};
 preferences.savePreferences(storage,{language:'en',theme:'light'});
 assert.deepEqual(preferences.loadPreferences(storage,'nl',true),{language:'en',theme:'light'});
});
