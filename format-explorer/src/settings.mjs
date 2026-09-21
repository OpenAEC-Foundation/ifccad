import { readPreferences, savePreferences, resolveLanguage, resolveTheme } from './preferences.mjs';
import { setLanguage, translateTree } from './i18n.mjs';

export function initializeSettings(refresh){
  const $=id=>document.getElementById(id),dialog=$('settings-dialog');
  let storage;try{storage=window.localStorage;}catch{/* Session preferences still work. */}
  let saved=readPreferences(storage),draft={...saved};
  const system=window.matchMedia('(prefers-color-scheme: dark)');
  const about=$('about');about.hidden=false;$('settings-about').append(about);
  $('about-toggle').removeAttribute('aria-expanded');$('about-toggle').setAttribute('aria-controls','settings-dialog');
  function apply(value){
    const language=resolveLanguage(value.language,navigator.languages||[navigator.language]);
    setLanguage(language);document.documentElement.lang=language;
    document.documentElement.dataset.theme=resolveTheme(value.appearance,system.matches);
    $('resolved-language').textContent=(language==='nl'?'Huidige taal: Nederlands':'Current language: English');
    refresh();
  }
  function tab(name,focus=false){
    for(const button of dialog.querySelectorAll('[data-settings-tab]')){
      const active=button.dataset.settingsTab===name;button.setAttribute('aria-selected',String(active));button.tabIndex=active?0:-1;
      $('panel-'+button.dataset.settingsTab).hidden=!active;if(active&&focus)button.focus();
    }
  }
  function open(name='general'){
    draft={...saved};$('language-preference').value=draft.language;
    for(const radio of dialog.querySelectorAll('[name=appearance]'))radio.checked=radio.value===draft.appearance;
    $('settings-status').textContent='Wijzigingen zijn een preview tot je opslaat.';
    tab(name);translateTree(dialog);dialog.showModal();$('tab-'+name).focus();
  }
  function cancel(){draft={...saved};apply(saved);dialog.close();}
  $('settings-open').addEventListener('click',()=>open());
  $('about-toggle').addEventListener('click',()=>open('about'));
  $('settings-close').addEventListener('click',cancel);$('settings-cancel').addEventListener('click',cancel);
  dialog.addEventListener('cancel',e=>{e.preventDefault();cancel();});
  $('settings-save').addEventListener('click',()=>{
    saved={...draft};const stored=savePreferences(storage,saved);apply(saved);
    if(stored)dialog.close();else {
      $('settings-status').textContent='Opslaan is niet beschikbaar. De keuze geldt voor deze sessie.';
      translateTree(dialog);
    }
  });
  $('language-preference').addEventListener('change',e=>{draft.language=e.target.value;apply(draft);});
  for(const radio of dialog.querySelectorAll('[name=appearance]'))radio.addEventListener('change',()=>{if(radio.checked){draft.appearance=radio.value;apply(draft);}});
  const tabs=[...dialog.querySelectorAll('[data-settings-tab]')];
  tabs.forEach((button,index)=>{
    button.addEventListener('click',()=>tab(button.dataset.settingsTab));
    button.addEventListener('keydown',e=>{let next;if(e.key==='ArrowDown'||e.key==='ArrowRight')next=(index+1)%tabs.length;else if(e.key==='ArrowUp'||e.key==='ArrowLeft')next=(index+tabs.length-1)%tabs.length;else if(e.key==='Home')next=0;else if(e.key==='End')next=tabs.length-1;if(next!==undefined){e.preventDefault();tab(tabs[next].dataset.settingsTab,true);}});
  });
  system.addEventListener('change',()=>apply(dialog.open?draft:saved));
  window.addEventListener('languagechange',()=>apply(dialog.open?draft:saved));
  apply(saved);
}
