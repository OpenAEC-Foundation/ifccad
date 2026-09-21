/** Preserve integer identity tokens beyond JS precision; original text stays separate. */
export function parsePresentationJson(text){
 const tokens=text.match(/"(?:\\[\s\S]|[^"\\])*"|-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?|[^"\d-]+|[\s\S]/g)||[];
 // Validate first, so token protection never repairs malformed input.
 JSON.parse(text);
 return JSON.parse(tokens.map(token=>/^-?\d+$/.test(token)&&!Number.isSafeInteger(Number(token))?JSON.stringify(token):token).join(''));
}
export function decodeBundle(bundle){
 const rawDocuments=Object.create(null),files=Object.create(null),blobs=Object.create(null),blobLengths=Object.create(null);
 for(const d of bundle.documents){rawDocuments[d.path]=d.text;files[d.path]=parsePresentationJson(d.text);}
 for(const b of bundle.blobs||[]){blobs[b.path]=Array.from(atob(b.previewBase64),c=>c.charCodeAt(0));blobLengths[b.path]=b.byteLength;}
 if(!files['package.ifcx.json'])throw Error('Missing package.ifcx.json');
 return {name:bundle.name,label:bundle.name,description:'Lokaal geopend pakket',ifcx:files['package.ifcx.json'],files,blobs,blobLengths,rawDocuments,warnings:bundle.warnings||[]};
}
