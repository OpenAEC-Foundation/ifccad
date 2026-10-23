import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {sendFeedback,initializeFeedback} from '../src/feedback.mjs';
import {translate} from '../src/i18n.mjs';
const draft={category:'bug',title:'Missing dimension',message:'A dimension label is missing.'};
const issue='https://github.com/OpenAEC-Foundation/ifccad/issues/123';
const accepted=()=>Response.json({success:true,githubIssueUrl:issue});

test('feedback uses the website issue route with explicit fields and the title on the first line',async()=>{
 const requests=[];
 const result=await sendFeedback({...draft,title:' Missing dimension ',message:'  A dimension label is missing.  ',email:' person@example.org ',fileName:'secret.dwg',source:'private drawing'}, {fetch:async(url,options)=>{requests.push({url,options});return accepted();}});
 assert.equal(requests[0].url,'https://open-feedback-studio.pages.dev/api/feedback');
 assert.deepEqual(JSON.parse(requests[0].options.body),{app:'ifccad',category:'bug',message:'Missing dimension\n\nA dimension label is missing.',email:'person@example.org'});
 assert.equal(requests[0].options.credentials,'omit');assert.equal(requests[0].options.referrerPolicy,'no-referrer');
 assert.equal(result.githubIssueUrl,issue);
});

test('feedback validates title, category, description and optional email locally',async()=>{
 const fetch=()=>{throw Error('Invalid feedback must stay local');};
 for(const fields of [{category:'unknown'},{title:''},{title:'x'.repeat(201)},{title:'first\nsecond'},{message:'short'},{message:'x'.repeat(4501)},{email:'not an email'}])await assert.rejects(sendFeedback({...draft,...fields},{fetch}),{code:'feedbackInvalid'});
 let body;
 await sendFeedback({...draft,title:'x'.repeat(200),message:'x'.repeat(4500)},{fetch:async(_,options)=>{body=JSON.parse(options.body);return accepted();}});
 assert.equal(body.message.length,4702);assert.equal(body.email,undefined);
});

test('stored feedback succeeds without claiming issue creation and unsafe links are omitted',async()=>{
 for(const githubIssueUrl of [null,undefined,'javascript:alert(1)','https://github.com/other/repo/issues/1','https://github.com/OpenAEC-Foundation/ifccad/issues/1/extra','https://github.com.evil.test/OpenAEC-Foundation/ifccad/issues/1','https://github.com/OpenAEC-Foundation/ifccad/issues/1?evil=1']){
  const result=await sendFeedback(draft,{fetch:async()=>Response.json({success:true,githubIssueUrl})});
  assert.equal(result.githubIssueUrl,null);
 }
});

test('feedback rejects malformed acknowledgements and retains specific service errors',async()=>{
 for(const response of [new Response('<html>not deployed</html>',{status:404}),Response.json({success:false}),Response.json({success:true,visibility:'private'}),Response.json(null)]){
  let count=0;
  await assert.rejects(sendFeedback(draft,{fetch:async()=>{count++;return response;}}),{code:'feedbackUnavailable'});
  assert.equal(count,1);
 }
 for(const [status,code]of [[429,'feedbackRateLimit'],[409,'feedbackDuplicate'],[403,'feedbackUnavailable'],[500,'feedbackUnavailable']])await assert.rejects(sendFeedback(draft,{fetch:async()=>Response.json({error:'server failure'},{status})}),{code});
 await assert.rejects(sendFeedback(draft,{fetch:async()=>{throw Error('offline');}}),{code:'feedbackUnavailable'});
});

function formHarness(fetch){
 const elements=new Map();
 const element=id=>{if(!elements.has(id))elements.set(id,{value:'',hidden:true,textContent:'',disabled:false,listeners:{},addEventListener(name,callback){this.listeners[name]=callback;},removeAttribute(name){delete this[name];},reportValidity(){return true;},reset(){for(const name of ['category','title','message','email','website'])element('feedback-'+name).value='';}});return elements.get(id);};
 const view=initializeFeedback({document:{getElementById:element},fetch,translate:key=>key});
 for(const [key,value]of Object.entries(draft))element('feedback-'+key).value=value;
 return {element,view,submit:()=>element('feedback-form').onsubmit({preventDefault(){}})};
}

test('feedback keeps failed drafts, blocks duplicate clicks and shows the confirmed issue link',async()=>{
 let resolveRequest,count=0;
 const {element,submit,view}=formHarness(()=>{count++;return new Promise(resolve=>{resolveRequest=resolve;});});
 const pending=submit();await submit();
 assert.equal(count,1);assert.equal(element('feedback-send').disabled,true);assert.equal(element('feedback-title').disabled,true);
 resolveRequest(Response.json({error:'busy'},{status:429}));await pending;
 assert.equal(element('feedback-title').value,draft.title);assert.equal(element('feedback-message').value,draft.message);
 assert.equal(element('feedback-status').textContent,'feedbackRateLimit');assert.equal(element('feedback-issue').hidden,true);
 const retry=submit();resolveRequest(accepted());await retry;
 assert.equal(element('feedback-message').value,'');assert.equal(element('feedback-title').value,'');
 assert.equal(element('feedback-status').textContent,'feedbackIssueCreated');assert.equal(element('feedback-issue').href,issue);assert.equal(element('feedback-issue').hidden,false);
 view.refresh();assert.equal(element('feedback-issue').textContent,'feedbackViewIssue');
 element('feedback-form').listeners.input();assert.equal(element('feedback-issue').hidden,true);assert.equal(element('feedback-issue').href,undefined);
});

test('receipt without an issue URL does not claim an issue or display a link',async()=>{
 const {element,submit}=formHarness(async()=>Response.json({success:true,githubIssueUrl:null}));
 await submit();assert.equal(element('feedback-status').textContent,'feedbackSuccess');assert.equal(element('feedback-issue').hidden,true);
});

test('closing feedback cancels requests, clears issue links and protects reopened drafts from late responses',async()=>{
 let resolveRequest,signal;
 const {element,submit}=formHarness((_,options)=>{signal=options.signal;return new Promise(resolve=>{resolveRequest=resolve;});});
 const pending=submit();element('feedback-dialog').listeners.close();assert.equal(signal.aborted,true);
 element('feedback-message').value='A new message for later.';
 resolveRequest(accepted());await pending;
 assert.equal(element('feedback-message').value,'A new message for later.');assert.equal(element('feedback-send').disabled,false);assert.equal(element('feedback-issue').hidden,true);
 const retry=submit();resolveRequest(accepted());await retry;
 assert.equal(element('feedback-issue').hidden,false);element('feedback-dialog').listeners.close();assert.equal(element('feedback-issue').hidden,true);assert.equal(element('feedback-issue').href,undefined);
});

test('English and Dutch forms disclose public issues, email privacy and local drawing data',async()=>{
 const html=await readFile(new URL('../src/index.html',import.meta.url),'utf8');
 assert.match(html,/id="feedback-title"[^>]*maxlength="200"/);
 assert.match(html,/id="feedback-message"[^>]*maxlength="4500"/);
 assert.match(html,/id="feedback-issue"[^>]*target="_blank"[^>]*rel="noopener noreferrer"/);
 for(const language of ['en','nl']){
  assert.match(translate(language,'feedbackIntro'),/GitHub/);
  assert.match(translate(language,'feedbackIntro'),language==='en'?/public/:/openbaar/);
  assert.match(translate(language,'feedbackPrivacy'),language==='en'?/email.*issue/i:/e-mailadres.*issue/i);
 }
});
