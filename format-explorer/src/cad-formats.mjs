export const cadVersions=Object.freeze([
 ['AC1015','2000'],['AC1018','2004'],['AC1021','2007'],
 ['AC1024','2010'],['AC1027','2013'],['AC1032','2018']
]);
export const defaultCadVersion='AC1032';
export const supportsCadVersion=value=>cadVersions.some(([code])=>code===value);
