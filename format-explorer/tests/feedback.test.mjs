import test from 'node:test';
import assert from 'node:assert/strict';
const feedback=await import('../src/feedback.mjs').catch(()=>({}));

test('feedback sends only the chosen fields to the private Open Feedback Studio endpoint',async()=>{
 assert.equal(typeof feedback.sendFeedback,'function');
 const requests=[];
 await feedback.sendFeedback({category:'bug',message:'  A dimension label is missing.  ',email:' person@example.org ',fileName:'secret.dwg',source:'private drawing'}, {fetch:async(url,options)=>{requests.push({url,options});return Response.json({success:true,visibility:'private',githubIssueUrl:null});}});
 assert.equal(requests[0].url,'https://open-feedback-studio.pages.dev/api/private-feedback');
 assert.deepEqual(JSON.parse(requests[0].options.body),{app:'IFCCAD',category:'bug',message:'A dimension label is missing.',email:'person@example.org'});
 assert.equal(requests[0].options.credentials,'omit');
});

test('feedback validates category, message and optional email before contacting the service',async()=>{
 assert.equal(typeof feedback.sendFeedback,'function');
 const fetch=()=>{throw Error('Invalid feedback must stay local');};
 for(const fields of [{category:'unknown',message:'A useful message'},{category:'bug',message:'short'},{category:'bug',message:'x'.repeat(5001)},{category:'general',message:'A useful message',email:'not an email'}])await assert.rejects(feedback.sendFeedback(fields,{fetch}),{code:'feedbackInvalid'});
 let body;
 await feedback.sendFeedback({category:'feature',message:'Please add a useful feature.'},{fetch:async(_,options)=>{body=JSON.parse(options.body);return Response.json({success:true,visibility:'private'});}});
 assert.equal(body.email,undefined);
});

test('feedback never falls back to a public route and requires private acknowledgement',async()=>{
 assert.equal(typeof feedback.sendFeedback,'function');
 for(const response of [new Response('<html>not deployed</html>',{status:404}),Response.json({success:true}),Response.json({success:true,visibility:'private',githubIssueUrl:'https://github.com/a/b/issues/1'})]){
  let count=0;
  await assert.rejects(feedback.sendFeedback({category:'general',message:'A useful private message'},{fetch:async()=>{count++;return response;}}),{code:'feedbackUnavailable'});
  assert.equal(count,1);
 }
 for(const [status,code]of [[429,'feedbackRateLimit'],[409,'feedbackDuplicate'],[403,'feedbackUnavailable'],[500,'feedbackUnavailable']])await assert.rejects(feedback.sendFeedback({category:'bug',message:'A useful private message'},{fetch:async()=>Response.json({error:'server failure'},{status})}),{code});
});

function formHarness(fetch){
 const elements=new Map();
 const element=id=>{if(!elements.has(id))elements.set(id,{value:'',hidden:true,textContent:'',disabled:false,listeners:{},addEventListener(name,callback){this.listeners[name]=callback;},reportValidity(){return true;},reset(){for(const name of ['category','message','email','website'])element('feedback-'+name).value='';}});return elements.get(id);};
 const view=feedback.initializeFeedback({document:{getElementById:element},fetch,translate:key=>key});
 element('feedback-category').value='bug';element('feedback-message').value='A dimension label is missing.';
 return {element,view,submit:()=>element('feedback-form').onsubmit({preventDefault(){}})};
}

test('feedback keeps failed drafts, blocks duplicate clicks and clears only after private acceptance',async()=>{
 assert.equal(typeof feedback.initializeFeedback,'function');
 let resolveRequest,count=0;
 const {element,submit}=formHarness(()=>{count++;return new Promise(resolve=>{resolveRequest=resolve;});});
 const pending=submit();await submit();
 assert.equal(count,1);assert.equal(element('feedback-send').disabled,true);
 resolveRequest(Response.json({error:'busy'},{status:429}));await pending;
 assert.equal(element('feedback-message').value,'A dimension label is missing.');
 assert.equal(element('feedback-status').textContent,'feedbackRateLimit');
 assert.equal(element('feedback-send').disabled,false);
 const retry=submit();resolveRequest(Response.json({success:true,visibility:'private'}));await retry;
 assert.equal(element('feedback-message').value,'');assert.equal(element('feedback-status').textContent,'feedbackSuccess');
});

test('closing feedback cancels the request and a late response cannot clear a reopened draft',async()=>{
 assert.equal(typeof feedback.initializeFeedback,'function');
 let resolveRequest,signal;
 const {element,submit}=formHarness((_,options)=>{signal=options.signal;return new Promise(resolve=>{resolveRequest=resolve;});});
 const pending=submit();element('feedback-dialog').listeners.close();
 assert.equal(signal.aborted,true);
 element('feedback-message').value='A new private message for later.';
 resolveRequest(Response.json({success:true,visibility:'private'}));await pending;
 assert.equal(element('feedback-message').value,'A new private message for later.');
 assert.equal(element('feedback-send').disabled,false);
});
