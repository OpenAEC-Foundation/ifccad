import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {createJobHandler} from './jobs.mjs';

/** HTTPS terminates at the OpenAEC reverse proxy. This service stays on loopback. */
export function productionServer({publicOrigin=process.env.PUBLIC_ORIGIN,api,root=fileURLToPath(new URL('../dist/',import.meta.url))}={}){
 if(!publicOrigin)throw Error('PUBLIC_ORIGIN is required');
 api??=createJobHandler({publicOrigin});
 const origin=new URL(publicOrigin),base=path.resolve(root);
 const types={'.html':'text/html; charset=utf-8','.mjs':'text/javascript; charset=utf-8','.css':'text/css; charset=utf-8','.json':'application/json; charset=utf-8','.svg':'image/svg+xml','.ttf':'font/ttf','.txt':'text/plain; charset=utf-8'};
 const server=createServer(async(req,res)=>{
  res.setHeader('X-Content-Type-Options','nosniff');res.setHeader('Cache-Control','no-store');
  res.setHeader('Referrer-Policy','no-referrer');res.setHeader('X-Frame-Options','DENY');
  try{
   if(req.headers.host!==origin.host){res.writeHead(403);res.end('Invalid host');return;}
   if(await api.handle(req,res))return;
   if(!['GET','HEAD'].includes(req.method)){res.writeHead(405);res.end();return;}
   const pathname=decodeURIComponent(new URL(req.url,origin).pathname),file=path.resolve(base,'.'+(pathname==='/'?'/index.html':pathname));
   const type=types[path.extname(file)];
   if(!file.startsWith(base+path.sep)||!type){res.writeHead(404);res.end();return;}
   const data=await readFile(file);res.writeHead(200,{'Content-Type':type});res.end(req.method==='HEAD'?undefined:data);
  }catch{if(!res.headersSent)res.writeHead(404);res.end();}
 });
 server.requestTimeout=30000;server.headersTimeout=10000;server.keepAliveTimeout=5000;server.maxConnections=64;
 return {server,async close(){server.close();await api.manager.close();}};
}
if(process.argv[1]===fileURLToPath(import.meta.url)){
 const service=productionServer();
 service.server.listen(Number(process.env.PORT||4183),'127.0.0.1',()=>console.log('IFCCAD Format Explorer service ready'));
 for(const signal of ['SIGINT','SIGTERM'])process.on(signal,async()=>{await service.close();process.exit(0);});
}
