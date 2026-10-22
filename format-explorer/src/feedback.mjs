export const feedbackEndpoint='https://open-feedback-studio.pages.dev/api/private-feedback';
const failure=code=>Object.assign(new Error(code),{code});

/** Explicit fields only: drawing contents and file names never enter the request. */
export async function sendFeedback(fields,{fetch:send=globalThis.fetch,signal}={}){
 const category=fields.category,message=String(fields.message??'').trim(),email=String(fields.email??'').trim();
 if(!['bug','feature','general'].includes(category)||message.length<10||message.length>5000||email&&(email.length>254||!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)))throw failure('feedbackInvalid');
 const body={app:'IFCCAD',category,message,...(email?{email}:{})};
 if(fields.website)body.website=String(fields.website);
 let response;
 try{response=await send(feedbackEndpoint,{method:'POST',headers:{'Content-Type':'application/json'},credentials:'omit',referrerPolicy:'no-referrer',body:JSON.stringify(body),signal:signal?AbortSignal.any([signal,AbortSignal.timeout(15000)]):AbortSignal.timeout(15000)});}catch(error){if(signal?.aborted)throw error;throw failure('feedbackUnavailable');}
 if(!response.ok)throw failure(response.status===429?'feedbackRateLimit':response.status===409?'feedbackDuplicate':'feedbackUnavailable');
 let result;try{result=await response.json();}catch{throw failure('feedbackUnavailable');}
 if(result?.success!==true||result.visibility!=='private'||result.githubIssueUrl)throw failure('feedbackUnavailable');
 return result;
}

export function initializeFeedback({document=globalThis.document,fetch=globalThis.fetch,translate=key=>key}={}){
 const $=id=>document.getElementById('feedback-'+id),form=$('form'),dialog=$('dialog'),button=$('send'),status=$('status');
 const fields=['category','message','email','website'];let pending=false,controller,requestId=0,statusKey='';
 const refresh=()=>{status.hidden=!statusKey;status.textContent=statusKey?translate(statusKey):'';button.textContent=translate(pending?'feedbackSending':'feedbackSend');};
 const busy=value=>{pending=value;button.disabled=value;for(const field of fields)$(field).disabled=value;form.setAttribute?.('aria-busy',String(value));refresh();};
 form.onsubmit=async event=>{
  event.preventDefault();if(pending||!form.reportValidity())return;
  const draft=Object.fromEntries(fields.map(field=>[field,$(field).value]));
  const id=++requestId;controller=new AbortController();statusKey='';busy(true);
  try{
   await sendFeedback(draft,{fetch,signal:controller.signal});
   if(id!==requestId)return;
   form.reset();statusKey='feedbackSuccess';
  }catch(error){if(id===requestId)statusKey=error.code||'feedbackUnavailable';}
  finally{if(id===requestId)busy(false);}
 };
 dialog.addEventListener('close',()=>{requestId++;controller?.abort();statusKey='';busy(false);});
 form.addEventListener('input',()=>{if(!pending&&statusKey){statusKey='';refresh();}});
 return {refresh};
}
