import { readFile } from 'node:fs/promises';
export const examples = [
  ['unrepresented-packed','Een tekening met bronbehoud','Lijnen, polylijnen en bewaarde bronrecords.'],
  ['multi-drawing-projections','Twee tekeningen, één bron','Entiteit-ID’s zijn uniek binnen hun eigen resource.'],
  ['inline-both','Resources binnen IFCX','Dezelfde inhoud, nu ingebed in het graphdocument.'],
  ['tilted-plane','Een polylijn in een vlak','Lokale XY-punten krijgen een plaats in XYZ.'],
];
export async function readExamples() {
  const result=[];
  for(const [name,label,description]of examples){
    const root=new URL(`../../conformance/next/packages/valid/${name}/`,import.meta.url);
    const ifcx=JSON.parse(await readFile(new URL('package.ifcx.json',root),'utf8'));
    const files={},blobs={};
    for(const node of ifcx.data){const d=node.attributes?.resource||node.attributes?.preservation;if(!d)continue;let body=d.content;if(d.uri){body=JSON.parse(await readFile(new URL(d.uri,root),'utf8'));files[d.uri]=body;}for(const blob of body.blobs||[])blobs[blob.uri]=[...await readFile(new URL(blob.uri,root))];}
    result.push({name,label,description,ifcx,files,blobs});
  }
  return result;
}
