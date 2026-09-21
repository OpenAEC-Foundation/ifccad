import {productionServer} from './scripts/production.mjs';

// This entry point is only for the existing private Docker network. There are
// no published container ports; nginx is the sole public entry point.
const service=productionServer();
service.server.listen(4183,'0.0.0.0',()=>console.log('IFCCAD Format Explorer ready'));
for(const signal of ['SIGINT','SIGTERM'])process.on(signal,async()=>{
  await service.close();
  process.exit(0);
});
