import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { watch } from 'node:fs';
import { build, output } from './build.mjs';
import { createJobHandler } from './jobs.mjs';
const api=createJobHandler();
await build();
const root=fileURLToPath(output),port=Number(process.env.PORT||4173);
const clients=new Set();
const reloadScript='<script>const changes=new EventSource("./__viewer_events");changes.addEventListener("change",()=>location.reload());</script>';
const types={'.html':'text/html; charset=utf-8','.css':'text/css; charset=utf-8','.mjs':'text/javascript; charset=utf-8','.json':'application/json; charset=utf-8','.svg':'image/svg+xml','.ttf':'font/ttf','.txt':'text/plain; charset=utf-8'};
const server=createServer(async(req,res)=>{
  try{
    if(await api.handle(req,res))return;
    const url=new URL(req.url,'http://localhost');
    if(url.pathname==='/__viewer_events'){
      res.writeHead(200,{'Content-Type':'text/event-stream','Cache-Control':'no-cache','Connection':'keep-alive'});
      res.write(': connected\n\n');clients.add(res);req.on('close',()=>clients.delete(res));return;
    }
    const pathname=decodeURIComponent(url.pathname);
    const file=path.resolve(root,'.'+(pathname==='/'?'/index.html':pathname));
    if(!file.startsWith(root)||!['GET','HEAD'].includes(req.method)){res.writeHead(403);res.end('Forbidden');return;}
    let data=await readFile(file);if(path.extname(file)==='.html')data=Buffer.from(data.toString('utf8').replace('</body>',reloadScript+'</body>'));
    res.writeHead(200,{'Content-Type':types[path.extname(file)]||'application/octet-stream','Cache-Control':'no-store'});res.end(req.method==='HEAD'?undefined:data);
  }catch{res.writeHead(404,{'Content-Type':'text/plain; charset=utf-8'});res.end('Niet gevonden');}
});
server.on('error',error=>{console.error(error.code==='EADDRINUSE'?`Port ${port} is in use. Choose another PORT.`:error);process.exit(1);});
server.requestTimeout=120000;
server.listen(port,'127.0.0.1',()=>console.log(`IFCCAD viewer ready at http://127.0.0.1:${port}`));
for(const signal of ['SIGINT','SIGTERM'])process.on(signal,async()=>{server.close();await api.manager.close();process.exit(0);});
let timer,building=false,pending=false;
async function rebuild(){
  if(building){pending=true;return;}
  building=true;
  try{await build();for(const client of clients)client.write('event: change\ndata: rebuilt\n\n');}
  catch(error){console.error('Viewer rebuild failed:',error.message);}
  finally{building=false;if(pending){pending=false;await rebuild();}}
}
watch(new URL('../src/',import.meta.url),()=>{clearTimeout(timer);timer=setTimeout(rebuild,120);});
