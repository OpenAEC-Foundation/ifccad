/** Preserve integer identity tokens beyond JS precision; original text stays separate. */
export function parsePresentationJson(text){
 // Validate first, so token protection never repairs malformed input.
 const validated=JSON.parse(text),parts=[];
 let start=0,i=0;
 while(i<text.length){
  const code=text.charCodeAt(i);
  if(code===34){
   i++;
   while(i<text.length){
    const current=text.charCodeAt(i++);
    if(current===92)i++;
    else if(current===34)break;
   }
   continue;
  }
  if(code===45||(code>=48&&code<=57)){
   const begin=i++;let integer=true;
   while(i<text.length){
    const next=text.charCodeAt(i);
    if((next>=48&&next<=57)||next===43||next===45){i++;continue;}
    if(next===46||next===69||next===101){integer=false;i++;continue;}
    break;
   }
   const token=text.slice(begin,i);
   if(integer&&!Number.isSafeInteger(Number(token))){parts.push(text.slice(start,begin),'"',token,'"');start=i;}
   continue;
  }
  i++;
 }
 if(!parts.length)return validated;
 parts.push(text.slice(start));
 return JSON.parse(parts.join(''));
}
