import {readdir,readFile,lstat} from 'node:fs/promises';
import path from 'node:path';
import {limits,normalizeUploadPath} from './upload-paths.mjs';

// ZIP transport only, not an IFCCAD container specification. Stored entries retain
// exact bytes/checksums. The 64 MiB / 1000-file limits exclude ZIP64 cases.
const crcTable=Uint32Array.from({length:256},(_,value)=>{for(let i=0;i<8;i++)value=value&1?0xedb88320^(value>>>1):value>>>1;return value>>>0;});
export function crc32(bytes){let value=0xffffffff;for(const byte of bytes)value=crcTable[(value^byte)&255]^(value>>>8);return (value^0xffffffff)>>>0;}
export function zipFiles(files,cap=limits){
 if(!files.length||files.length>cap.files)throw Error('Package file count limit exceeded');
 const local=[],central=[],seen=new Set();let offset=0,centralSize=0;
 for(const file of files){
  const name=normalizeUploadPath(file.path),key=name.toLowerCase();if(seen.has(key))throw Error('Duplicate package path');seen.add(key);
  const encoded=Buffer.from(name),bytes=Buffer.from(file.bytes);
  if(encoded.length>65535)throw Error('Package path is too long');
  const length=bytes.length,crc=crc32(bytes),header=Buffer.alloc(30),directory=Buffer.alloc(46);
  // UTF-8 names, method=stored, timestamp=1980-01-01, no extra fields/comments.
  header.writeUInt32LE(0x04034b50,0);header.writeUInt16LE(20,4);header.writeUInt16LE(0x800,6);header.writeUInt16LE(33,12);
  header.writeUInt32LE(crc,14);header.writeUInt32LE(length,18);header.writeUInt32LE(length,22);header.writeUInt16LE(encoded.length,26);
  directory.writeUInt32LE(0x02014b50,0);directory.writeUInt16LE(20,4);directory.writeUInt16LE(20,6);directory.writeUInt16LE(0x800,8);directory.writeUInt16LE(33,14);
  directory.writeUInt32LE(crc,16);directory.writeUInt32LE(length,20);directory.writeUInt32LE(length,24);directory.writeUInt16LE(encoded.length,28);directory.writeUInt32LE(offset,42);
  offset+=header.length+encoded.length+length;centralSize+=directory.length+encoded.length;
  if(offset+centralSize+22>cap.bytes)throw Error('Package download exceeds the 64 MiB limit');
  local.push(header,encoded,bytes);central.push(directory,encoded);
 }
 const end=Buffer.alloc(22);end.writeUInt32LE(0x06054b50,0);end.writeUInt16LE(files.length,8);end.writeUInt16LE(files.length,10);end.writeUInt32LE(centralSize,12);end.writeUInt32LE(offset,16);
 return Buffer.concat([...local,...central,end]);
}
export async function packageZip(root,{signal,cap=limits}={}){
 const files=[];let bytes=0;
 async function visit(relative=''){
  for(const entry of (await readdir(path.join(root,relative))).sort()){
   signal?.throwIfAborted();const name=relative?relative+'/'+entry:entry;
   normalizeUploadPath(name);const file=path.join(root,name),stat=await lstat(file);
   if(stat.isSymbolicLink())throw Error('Package links cannot be archived');
   if(stat.isDirectory()){await visit(name);continue;}
   if(!stat.isFile())throw Error('Unsupported package file');
   bytes+=stat.size;if(bytes>cap.bytes||files.length>=cap.files)throw Error('Package download limit exceeded');
   files.push({path:name,bytes:await readFile(file)});
  }
 }
 await visit();signal?.throwIfAborted();
 return {bytes:zipFiles(files,cap),fileCount:files.length};
}
