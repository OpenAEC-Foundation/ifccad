// Geometry in graph coordinates, independent of the current pan/zoom transform.
export function overlaps(a, b, gap = 0) {
  return a.x < b.x + b.width + gap && a.x + a.width + gap > b.x
    && a.y < b.y + b.height + gap && a.y + a.height + gap > b.y;
}

const same=(a,b)=>a.x===b.x&&a.y===b.y;
const between=(value,a,b)=>value>Math.min(a,b)&&value<Math.max(a,b);
function simplify(points){
  const result=[];
  for(const point of points){
    if(result.length&&same(result.at(-1),point))continue;
    while(result.length>1){
      const previous=result.at(-2),last=result.at(-1);
      if((previous.x===last.x&&last.x===point.x&&between(last.y,previous.y,point.y))||
         (previous.y===last.y&&last.y===point.y&&between(last.x,previous.x,point.x)))result.pop();
      else break;
    }
    result.push(point);
  }
  return result;
}
function crossesBox(from,to,box,gap){
  const left=box.x-gap,right=box.x+box.width+gap,top=box.y-gap,bottom=box.y+box.height+gap;
  if(from.x===to.x)return from.x>left&&from.x<right&&Math.max(from.y,to.y)>top&&Math.min(from.y,to.y)<bottom;
  if(from.y===to.y)return from.y>top&&from.y<bottom&&Math.max(from.x,to.x)>left&&Math.min(from.x,to.x)<right;
  return true;
}
function clearReferenceRoute(a,b,obstacles){
  const port=(box,side)=>{
    const cx=box.x+box.width/2,cy=box.y+box.height/2;
    return side==='top'?{x:cx,y:box.y,dx:0,dy:-1}:side==='bottom'?{x:cx,y:box.y+box.height,dx:0,dy:1}:
      side==='left'?{x:box.x,y:cy,dx:-1,dy:0}:{x:box.x+box.width,y:cy,dx:1,dy:0};
  };
  const sides=['top','bottom','left','right'],clearance=16,stub=20;
  let best=null;
  for(const sourceSide of sides)for(const targetSide of sides){
    const start=port(a,sourceSide),end=port(b,targetSide);
    const outward={x:start.x+start.dx*stub,y:start.y+start.dy*stub};
    const approach={x:end.x+end.dx*stub,y:end.y+end.dy*stub};
    const middleX=(outward.x+approach.x)/2,middleY=(outward.y+approach.y)/2;
    const candidates=[
      [outward,approach],
      [outward,{x:outward.x,y:approach.y},approach],
      [outward,{x:approach.x,y:outward.y},approach],
    ];
    const near=values=>[...new Set(values)].sort((left,right)=>Math.abs(left-middleX)-Math.abs(right-middleX)).slice(0,16);
    const xChannels=near([middleX,...obstacles.flatMap(box=>[box.x-clearance-4,box.x+box.width+clearance+4])]);
    const yChannels=[...new Set([middleY,...obstacles.flatMap(box=>[box.y-clearance-4,box.y+box.height+clearance+4])])]
      .sort((top,bottom)=>Math.abs(top-middleY)-Math.abs(bottom-middleY)).slice(0,16);
    for(const x of xChannels)candidates.push([outward,{x,y:outward.y},{x,y:approach.y},approach]);
    for(const y of yChannels)candidates.push([outward,{x:outward.x,y},{x:approach.x,y},approach]);
    for(const candidate of candidates){
      const points=simplify([{x:start.x,y:start.y},...candidate,{x:end.x,y:end.y}]);
      if(points.length<2||points.some((point,i)=>i>0&&point.x!==points[i-1].x&&point.y!==points[i-1].y))continue;
      if(points.some((point,i)=>i===0?false:obstacles.some(box=>crossesBox(points[i-1],point,box,clearance))))continue;
      if(points.some((point,i)=>i<2||i===points.length-1?false:[a,b].some(box=>crossesBox(points[i-1],point,box,0))))continue;
      const length=points.slice(1).reduce((sum,point,i)=>sum+Math.abs(point.x-points[i].x)+Math.abs(point.y-points[i].y),0);
      const score=length+(points.length-2)*28;
      if(!best||score<best.score)best={score,points};
    }
  }
  if(!best)return null;
  const points=best.points;
  return {kind:'clear-corridor',start:points[0],end:points.at(-1),points,path:points.map((point,i)=>`${i?'L':'M'}${point.x} ${point.y}`).join(' ')};
}

export function routeEdge(a,b,relation,obstacles){
  if(obstacles){const route=clearReferenceRoute(a,b,obstacles);if(route)return route;}
  if(['linkedDrawingResources','activeDrawing'].includes(relation)&&a.x>b.x+b.width){
    const start={x:a.x,y:a.y+a.height/2},end={x:b.x+b.width,y:b.y+b.height/2};
    const middle=(start.x+end.x)/2;
    return {kind:relation==='activeDrawing'?'workspace-bridge':'resource-bridge',start,end,path:`M${start.x} ${start.y} C${middle} ${start.y},${middle} ${end.y},${end.x} ${end.y}`};
  }
  const overlapX=Math.min(a.x+a.width,b.x+b.width)-Math.max(a.x,b.x);
  const separatedY=a.y+a.height<=b.y||b.y+b.height<=a.y;
  if(separatedY&&overlapX>=Math.min(a.width,b.width)/2){
    const downward=b.y>a.y;
    const start={x:a.x+a.width/2,y:a.y+(downward?a.height:0)};
    const end={x:b.x+b.width/2,y:b.y+(downward?0:b.height)};
    const middle=(start.y+end.y)/2;
    return {kind:'vertical',start,end,path:`M${start.x} ${start.y} C${start.x} ${middle},${end.x} ${middle},${end.x} ${end.y}`};
  }
  const backward=b.x<a.x;
  const start={x:backward?a.x:a.x+a.width,y:a.y+a.height/2};
  const end={x:backward?b.x+b.width:b.x,y:b.y+b.height/2};
  if(backward){
    const routeY=Math.min(a.y,b.y)-35;
    return {kind:'backward',start,end,path:`M${start.x} ${start.y} C${start.x-30} ${start.y},${start.x-30} ${routeY},${start.x-50} ${routeY} L${end.x+50} ${routeY} C${end.x+30} ${routeY},${end.x+30} ${end.y},${end.x} ${end.y}`};
  }
  const middle=(start.x+end.x)/2;
  return {kind:'forward',start,end,path:`M${start.x} ${start.y} C${middle} ${start.y},${middle} ${end.y},${end.x} ${end.y}`};
}

// New detail nodes find space without moving any already visible graph nodes.
export function placeDetailNode(preferred, occupied) {
  const box = {...preferred};
  while (occupied.some(other => overlaps(box, other, 32))) box.y += 36;
  return box;
}

export function wrapText(value, maxWidth, measure) {
  const lines = [];
  let rest = String(value);
  while (rest && measure(rest) > maxWidth) {
    let end = 1;
    while (end < rest.length && measure(rest.slice(0, end + 1)) <= maxWidth) end++;
    // Prefer word, identifier or camel-case boundaries without removing characters.
    const prefix = rest.slice(0, end + 1);
    const boundaries = [...prefix.matchAll(/[-_ /]|(?<=[a-z])(?=[A-Z])/g)]
      .map(m => m.index + (m[0] ? 1 : 0)).filter(i => i > end / 2 && i <= end);
    end = boundaries.at(-1) || end;
    lines.push(rest.slice(0, end));
    rest = rest.slice(end);
  }
  if (rest) lines.push(rest);
  return lines;
}

export function placeLabel(points, width, height, occupied, clearance=6) {
  const free = box => occupied.every(other => !overlaps(box, other, clearance));
  const candidates = [];
  // Keep the caption on its connection whenever possible. Compare all nearby
  // candidates before accepting an offset; first-free searches can jump to a
  // different branch just because an earlier sample point was obstructed.
  points.forEach((point, index) => {
    const offsets = [[0, 0]];
    for (const distance of [height / 2 + 8, height + 16, height + 32])
      offsets.push([0, -distance], [0, distance], [-distance, 0], [distance, 0]);
    for (const [dx, dy] of offsets) {
      const box = {x: point.x + dx - width / 2, y: point.y + dy - height / 2,
        width, height, anchor: point};
      if (free(box)) candidates.push({box, score: Math.hypot(dx, dy) + index * 12});
    }
  });
  if (candidates.length) return candidates.sort((a, b) => a.score - b.score)[0].box;
  // Dense reference graphs still keep every label. Search nearby rows, with a
  // guaranteed free final row above all existing nodes and labels.
  const point = points[0];
  const ceiling = Math.min(point.y, ...occupied.map(r => r.y)) - height - 12;
  for (let y = point.y - height - 8; y > ceiling; y -= height + 12) {
    for (const dx of [0, -width - 12, width + 12]) {
      const box = {x: point.x - width / 2 + dx, y, width, height, anchor: point};
      if (free(box)) return box;
    }
  }
  return {x: point.x - width / 2, y: ceiling, width, height, anchor: point};
}
