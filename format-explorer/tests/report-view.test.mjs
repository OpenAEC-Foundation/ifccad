import test from 'node:test';
import assert from 'node:assert/strict';
import {createReportView} from '../src/report-view.mjs';

function element(){
 const listeners=new Map();
 return {hidden:false,open:false,scrollTop:12,
  addEventListener(type,listener){listeners.set(type,listener);},
  dispatch(type){listeners.get(type)?.();},
  showModal(){this.open=true;},
  close(){this.open=false;this.dispatch('close');},
  click(){this.dispatch('click');},
  focus(){this.focused=true;},
 };
}

test('report can be reopened after closing without taking workspace space',()=>{
 const dialog=element(),openButton=element(),closeButton=element(),continueButton=element(),workspace=element();
 const view=createReportView({dialog,openButton,closeButton,continueButton,focusWorkspace:()=>workspace.focus()});
 assert.equal(openButton.hidden,true);
 view.setAvailable(true);
 view.show();
 assert.equal(dialog.open,true);
 assert.equal(dialog.scrollTop,0);
 continueButton.click();
 assert.equal(dialog.open,false);
 assert.equal(workspace.focused,true);
 openButton.click();
 assert.equal(dialog.open,true);
 closeButton.click();
 assert.equal(dialog.open,false);
});

test('leaving an imported package removes its report and reopen control',()=>{
 const dialog=element(),openButton=element(),closeButton=element(),continueButton=element();
 const view=createReportView({dialog,openButton,closeButton,continueButton});
 view.setAvailable(true);
 view.show();
 view.setAvailable(false);
 assert.equal(dialog.open,false);
 assert.equal(openButton.hidden,true);
});

test('Escape restores focus after the dialog has closed',()=>{
 const dialog=element(),openButton=element(),closeButton=element(),continueButton=element(),workspace=element();
 const view=createReportView({dialog,openButton,closeButton,continueButton,focusWorkspace:()=>workspace.focus()});
 view.setAvailable(true);
 view.show();
 dialog.open=false;
 dialog.dispatch('close');
 assert.equal(workspace.focused,true);
});
