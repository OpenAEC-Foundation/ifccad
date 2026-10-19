/** Explicit application-level file selection; each core reader still validates its format. */
export function ifccadEncoding(name){
 if(/\.zst$/i.test(name))return 'cbor-zstd';
 if(/\.cbor$/i.test(name))return 'cbor';
 if(/\.(?:ifcx(?:\.json)?|json)$/i.test(name))return 'ifcx';
 return null;
}
