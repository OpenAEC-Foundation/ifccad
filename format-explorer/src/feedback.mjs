export const feedbackEndpoint='https://open-feedback-studio.pages.dev/api/feedback';
const failure=code=>Object.assign(new Error(code),{code});

/** Explicit fields only: drawing contents and file names never enter the request. */
export async function sendFeedback(fields,{fetch:send=globalThis.fetch,signal}={}){
 const category=fields.category,title=String(fields.title??'').trim(),description=String(fields.message??'').trim(),email=String(fields.email??'').trim();
 if(!['bug','feature','general'].includes(category)||!title||title.length>200||/[\r\n]/.test(title)||description.length<10||description.length>4500||email&&(email.length>254||!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)))throw failure('feedbackInvalid');
 const body={app:'ifccad',category,message:title+'\n\n'+description,...(email?{email}:{})};
 if(fields.website)body.website=String(fields.website);
 let response;
 try{response=await send(feedbackEndpoint,{method:'POST',headers:{'Content-Type':'application/json'},credentials:'omit',referrerPolicy:'no-referrer',body:JSON.stringify(body),signal:signal?AbortSignal.any([signal,AbortSignal.timeout(15000)]):AbortSignal.timeout(15000)});}catch(error){if(signal?.aborted)throw error;throw failure('feedbackUnavailable');}
 if(!response.ok)throw failure(response.status===429?'feedbackRateLimit':response.status===409?'feedbackDuplicate':'feedbackUnavailable');
 let result;try{result=await response.json();}catch{throw failure('feedbackUnavailable');}
 if(result?.success!==true||result.visibility==='private')throw failure('feedbackUnavailable');
 const githubIssueUrl=typeof result.githubIssueUrl==='string'&&/^https:\/\/github\.com\/OpenAEC-Foundation\/ifccad\/issues\/[1-9]\d*$/.test(result.githubIssueUrl)?result.githubIssueUrl:null;
 return {...result,githubIssueUrl};
}

export function initializeFeedback({document=globalThis.document,fetch=globalThis.fetch,translate=key=>key}={}){
 const $=id=>document.getElementById('feedback-'+id),form=$('form'),dialog=$('dialog'),button=$('send'),status=$('status'),issue=$('issue');
 const fields=['category','title','message','email','website'];let pending=false,controller,requestId=0,statusKey='',issueUrl=null;
 const refresh=()=>{status.hidden=!statusKey;status.textContent=statusKey?translate(statusKey):'';button.textContent=translate(pending?'feedbackSending':'feedbackSend');issue.hidden=!issueUrl;issue.textContent=translate('feedbackViewIssue');if(issueUrl)issue.href=issueUrl;else issue.removeAttribute('href');};
 const busy=value=>{pending=value;button.disabled=value;for(const field of fields)$(field).disabled=value;form.setAttribute?.('aria-busy',String(value));refresh();};
 form.onsubmit=async event=>{
  event.preventDefault();if(pending||!form.reportValidity())return;
  const draft=Object.fromEntries(fields.map(field=>[field,$(field).value]));
  const id=++requestId;controller=new AbortController();statusKey='';issueUrl=null;busy(true);
  try{
   const result=await sendFeedback(draft,{fetch,signal:controller.signal});
   if(id!==requestId)return;
   form.reset();issueUrl=result.githubIssueUrl;statusKey=issueUrl?'feedbackIssueCreated':'feedbackSuccess';
  }catch(error){if(id===requestId)statusKey=error.code||'feedbackUnavailable';}
  finally{if(id===requestId)busy(false);}
 };
 dialog.addEventListener('close',()=>{requestId++;controller?.abort();statusKey='';issueUrl=null;busy(false);});
 form.addEventListener('input',()=>{if(!pending&&statusKey){statusKey='';issueUrl=null;refresh();}});
 return {refresh};
}
