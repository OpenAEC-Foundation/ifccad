const table=Uint32Array.from({length:256},(_,value)=>{for(let i=0;i<8;i++)value=value&1?0xedb88320^(value>>>1):value>>>1;return value>>>0;});
function crc32(bytes){let value=0xffffffff;for(const byte of bytes)value=table[(value^byte)&255]^(value>>>8);return (value^0xffffffff)>>>0;}
export function validPackagePath(value){return typeof value==='string'&&value.length>0&&value.length<=1024&&!/[\\:%\x00-\x1f<>"|?*]/.test(value)&&!value.startsWith('/')&&value.split('/').every(part=>part&&part!=='.'&&part!=='..'&&!/[. ]$/.test(part)&&! /^(con|prn|aux|nul|com[0-9]|lpt[0-9])(?:\.|$)/i.test(part));}
function header(size){const bytes=new Uint8Array(size);return {bytes,view:new DataView(bytes.buffer)};}
function merge(parts,length){const result=new Uint8Array(length);let offset=0;for(const part of parts){result.set(part,offset);offset+=part.length;}return result;}

/** ZIP is only a transport wrapper; entries retain their exact package bytes. */
export function zipBrowserFiles(files,{maxFiles=1000,maxBytes=64*1024*1024}={}){
 if(!files.length||files.length>maxFiles)throw Error('Package file count limit exceeded');
 const local=[],central=[],seen=new Set(),encoder=new TextEncoder();let offset=0,centralSize=0;
 for(const file of files){
  if(!validPackagePath(file.path))throw Error('Unsafe package path');
  const key=file.path.toLowerCase();if(seen.has(key))throw Error('Duplicate package path');seen.add(key);
  const name=encoder.encode(file.path),bytes=file.bytes instanceof Uint8Array?file.bytes:new Uint8Array(file.bytes);
  if(name.length>65535)throw Error('Package path is too long');
  const crc=crc32(bytes),a=header(30),b=header(46);
  a.view.setUint32(0,0x04034b50,true);a.view.setUint16(4,20,true);a.view.setUint16(6,0x800,true);a.view.setUint16(12,33,true);
  a.view.setUint32(14,crc,true);a.view.setUint32(18,bytes.length,true);a.view.setUint32(22,bytes.length,true);a.view.setUint16(26,name.length,true);
  b.view.setUint32(0,0x02014b50,true);b.view.setUint16(4,20,true);b.view.setUint16(6,20,true);b.view.setUint16(8,0x800,true);b.view.setUint16(14,33,true);
  b.view.setUint32(16,crc,true);b.view.setUint32(20,bytes.length,true);b.view.setUint32(24,bytes.length,true);b.view.setUint16(28,name.length,true);b.view.setUint32(42,offset,true);
  offset+=a.bytes.length+name.length+bytes.length;centralSize+=b.bytes.length+name.length;
  if(offset+centralSize+22>maxBytes)throw Error('Package download exceeds the 64 MiB limit');
  local.push(a.bytes,name,bytes);central.push(b.bytes,name);
 }
 const end=header(22);end.view.setUint32(0,0x06054b50,true);end.view.setUint16(8,files.length,true);end.view.setUint16(10,files.length,true);end.view.setUint32(12,centralSize,true);end.view.setUint32(16,offset,true);
 return merge([...local,...central,end.bytes],offset+centralSize+22);
}
