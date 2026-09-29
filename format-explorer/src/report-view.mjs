export function createReportView({dialog,openButton,closeButton,continueButton,scrollArea=dialog,focusWorkspace=()=>{},beforeShow=()=>{}}){
 let available=false;
 let focusAfterClose=true;
 openButton.hidden=true;
 function close(restoreFocus=true){
  if(!dialog.open)return;
  focusAfterClose=restoreFocus;
  dialog.close();
 }
 function show(){
  if(!available)return;
  beforeShow();
  scrollArea.scrollTop=0;
  if(!dialog.open)dialog.showModal();
 }
 function setAvailable(value){
  available=Boolean(value);
  openButton.hidden=!available;
  if(!available)close(false);
 }
 openButton.addEventListener('click',show);
 closeButton.addEventListener('click',()=>close());
 continueButton.addEventListener('click',()=>close());
 dialog.addEventListener('close',()=>{
  const restore=focusAfterClose;
  focusAfterClose=true;
  if(restore)focusWorkspace();
 });
 return {show,close,setAvailable};
}
