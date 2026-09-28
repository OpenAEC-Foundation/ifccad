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
export function decodeBundle(bundle){
 const rawDocuments=Object.create(null),files=Object.create(null),blobs=Object.create(null),blobLengths=Object.create(null);
 for(const d of bundle.documents){rawDocuments[d.path]=d.text;files[d.path]=parsePresentationJson(d.text);}
 for(const b of bundle.blobs||[]){blobs[b.path]=Array.from(atob(b.previewBase64),c=>c.charCodeAt(0));blobLengths[b.path]=b.byteLength;}
 if(!files['package.ifcx.json'])throw Error('Missing package.ifcx.json');
 return {name:bundle.name,label:bundle.name,description:'Lokaal geopend pakket',ifcx:files['package.ifcx.json'],files,blobs,blobLengths,rawDocuments,warnings:bundle.warnings||[]};
}
