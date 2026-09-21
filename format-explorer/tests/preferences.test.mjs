import test from 'node:test';
import assert from 'node:assert/strict';
import { resolveLanguage, resolveTheme, readPreferences, savePreferences } from '../src/preferences.mjs';
import { t } from '../src/i18n.mjs';

test('automatic language uses the ordered browser preferences and falls back to English', () => {
  assert.equal(resolveLanguage('auto',['nl-BE','en-US']), 'nl');
  assert.equal(resolveLanguage('auto',['de-DE','en-GB','nl']), 'en');
  assert.equal(resolveLanguage('auto',['fr']), 'en');
  assert.equal(resolveLanguage('auto',[]), 'en');
  assert.equal(resolveLanguage('nl',['en-US']), 'nl');
  assert.equal(resolveLanguage('en',['nl-NL']), 'en');
});

test('theme follows the system only in automatic mode', () => {
  assert.equal(resolveTheme('auto',true),'dark');
  assert.equal(resolveTheme('auto',false),'light');
  assert.equal(resolveTheme('light',true),'light');
});

test('preferences survive reload and unavailable or invalid storage is harmless', () => {
  let data;
  const storage={getItem:()=>data,setItem:(_,value)=>{data=value;}};
  assert.deepEqual(readPreferences(storage),{language:'auto',appearance:'auto'});
  assert.equal(savePreferences(storage,{language:'nl',appearance:'dark'}),true);
  assert.deepEqual(readPreferences(storage),{language:'nl',appearance:'dark'});
  data='{"language":"xx","appearance":"broken"}';
  assert.deepEqual(readPreferences(storage),{language:'auto',appearance:'auto'});
  data='invalid JSON';
  assert.deepEqual(readPreferences(storage),{language:'auto',appearance:'auto'});
  const blocked={getItem(){throw Error('blocked');},setItem(){throw Error('blocked');}};
  assert.deepEqual(readPreferences(blocked),{language:'auto',appearance:'auto'});
  assert.equal(savePreferences(blocked,{language:'en',appearance:'light'}),false);
});

test('UI captions translate while schema names and identities remain unchanged', () => {
  assert.equal(t('Gedeelde definities','en'),'Shared definitions');
  assert.equal(t('2 entiteiten · polylineStream','en'),'2 entities · polylineStream');
  assert.equal(t('scope 0 · laag 1','en'),'scope 0 · layer 1');
  for(const raw of ['placement','origin','layerId','drawing-main','polylineStream'])assert.equal(t(raw,'en'),raw);
  assert.equal(t('Typespecifieke velden','nl'),'Typespecifieke velden');
});

test('unrelated extension names remain literal in translation',()=>{
 for(const value of ['constructor','__proto__','toString'])assert.equal(t(value,'en'),value);
});
