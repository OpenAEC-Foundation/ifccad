import { readFile } from 'node:fs/promises';
export const examples = [
  ['unrepresented-packed','Een tekening met bronbehoud','Lijnen, polylijnen en bewaarde bronrecords.'],
  ['multi-drawing-projections','Twee tekeningen, één bron','Entiteit-ID’s zijn uniek binnen hun eigen resource.'],
  ['inline-both','Resources binnen IFCX','Dezelfde inhoud, nu ingebed in het graphdocument.'],
  ['tilted-plane','Een polylijn in een vlak','Lokale XY-punten krijgen een plaats in XYZ.'],
  ['blocks-demo','Een gedeeld block, vier plaatsingen','Een Arrow-definitie met rotatie, spiegeling en niet-uniforme schaal.'],
];
export const exampleRoot=name=>new URL(name==='blocks-demo'?'../examples/blocks-demo/':`../../conformance/next/packages/valid/${name}/`,import.meta.url);
export async function readExamples() {
  const result=[];
  for(const [name,label,description]of examples){
    const root=exampleRoot(name);
    const ifcx=JSON.parse(await readFile(new URL('package.ifcx.json',root),'utf8'));
    const files={},blobs={};
    for(const node of ifcx.data){const d=node.attributes?.resource||node.attributes?.preservation;if(!d)continue;let body=d.content;if(d.uri){body=JSON.parse(await readFile(new URL(d.uri,root),'utf8'));files[d.uri]=body;}for(const blob of body.blobs||[])blobs[blob.uri]=[...await readFile(new URL(blob.uri,root))];}
    const exportFiles=[];
    for(const path of new Set(['package.ifcx.json',...Object.keys(files),...Object.keys(blobs)]))exportFiles.push({path,base64:(await readFile(new URL(path,root))).toString('base64')});
    result.push({name,label,description,ifcx,files,blobs,exportFiles});
  }
  return result;
}
