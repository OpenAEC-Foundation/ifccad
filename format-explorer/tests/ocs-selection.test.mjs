import test from 'node:test';import assert from 'node:assert/strict';
const module=await import('../src/ocs-selection.mjs').catch(()=>({}));
test('observer follows only focused generated selection and suppresses forward echoes',async()=>{
 assert.equal(typeof module.createSelectionObserver,'function');
 let focus=true,reads=0,state={ok:true,document_id:7,layout:'Model',selection:['A']};const timers=new Map(),events=[];let next=0;
 const observer=module.createSelectionObserver({readState:async()=>{reads++;return state;},getGeneratedId:()=>7,isFocused:()=>focus,emit:event=>events.push(event),schedule:callback=>{timers.set(++next,callback);return next;},cancel:id=>timers.delete(id)});
 const tick=async()=>{const [id,callback]=timers.entries().next().value;timers.delete(id);await callback();};
 observer.setEnabled(true);await tick();assert.equal(events.at(-1).handles[0],'A');
 await tick();assert.equal(events.length,1);
 observer.expect(7,['B'],'Model');state={...state,selection:['B']};await tick();assert.equal(events.length,1);
 state={...state,selection:['C']};await tick();assert.deepEqual(events.at(-1).handles,['C']);
 focus=false;await tick();assert.equal(reads,4);focus=true;
 observer.pause();assert.equal(timers.size,0);observer.resume();
 state={...state,document_id:8,selection:['D']};await tick();assert.equal(events.length,2);
 observer.setEnabled(false);assert.equal(timers.size,0);observer.close();
});
test('an in-flight state cannot publish after document replacement or close',async()=>{
 let finish,generated=7;const events=[],timers=new Map();let next=0;
 const observer=module.createSelectionObserver({readState:()=>new Promise(resolve=>{finish=resolve;}),getGeneratedId:()=>generated,emit:event=>events.push(event),schedule:callback=>{timers.set(++next,callback);return next;},cancel:id=>timers.delete(id)});
 observer.setEnabled(true);const [id,callback]=timers.entries().next().value;timers.delete(id);const pending=callback();generated=8;observer.close();finish({ok:true,document_id:7,layout:'Model',selection:['A']});await pending;assert.deepEqual(events,[]);assert.equal(timers.size,0);
});
test('a late poll cannot undo a forward selection, but a user hint can reaffirm selection',async()=>{
 let finish;const events=[],timers=new Map();let next=0;
 const observer=module.createSelectionObserver({readState:()=>new Promise(resolve=>{finish=resolve;}),getGeneratedId:()=>7,emit:event=>events.push(event),schedule:callback=>{timers.set(++next,callback);return next;},cancel:id=>timers.delete(id)});
 const start=()=>{const [id,callback]=timers.entries().next().value;timers.delete(id);return callback();};
 observer.setEnabled(true);const stale=start();observer.pause();observer.expect(7,['B'],'Model');observer.resume();finish({ok:true,document_id:7,layout:'Model',selection:['A']});await stale;assert.deepEqual(events,[]);
 const echo=start();finish({ok:true,document_id:7,layout:'Model',selection:['B']});await echo;assert.deepEqual(events,[]);
 observer.wake();const user=start();finish({ok:true,document_id:7,layout:'Model',selection:['B']});await user;assert.deepEqual(events[0].handles,['B']);observer.close();
});
