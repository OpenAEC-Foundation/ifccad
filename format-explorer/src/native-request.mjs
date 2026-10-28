import {decodeBase64} from './browser-client.mjs';

/** Export only the already inspected snapshot; never repeat CAD import. */
export function nativeExportRequest(source,opening,operation){
 const file=opening?.nativeFile;
 if(opening?.failure||!opening?.validation?.strictAvailable||!file)throw Error('No validated native snapshot');
 if(source.kind==='ifccad'&&operation.encoding)return {...source,export:operation};
 const bytes=decodeBase64(file.base64);
 if(bytes.byteLength!==file.byteLength)throw Error('Native snapshot length mismatch');
 return {kind:'ifccad',name:file.fileName,files:[{path:file.fileName,bytes:bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)}],exportConversionOptions:source.exportConversionOptions??source.conversionOptions,export:operation};
}
