import { graphConnections, visibleGraph } from './model.mjs';
import { placeLabel, placeDetailNode, routeEdge, wrapText } from './layout.mjs';
import { t } from './i18n.mjs';
const NS='http://www.w3.org/2000/svg',WIDTH=260;
const element=(tag,attrs={})=>{const e=document.createElementNS(NS,tag);for(const[k,v]of Object.entries(attrs))e.setAttribute(k,v);return e;};
const text=(parent,value,x,y,cls)=>{const e=element('text',{x,y,class:cls});e.textContent=t(value);parent.append(e);return e;};
export class PackageGraph {
  constructor(svg,{select,toggle,zoomChanged,countChanged,referenceCountChanged}){
    this.svg=svg;this.callbacks={select,toggle,zoomChanged,countChanged,referenceCountChanged};this.camera={x:20,y:30,k:1};this.drag=null;this.referenceMode='focus';
    svg.addEventListener('click',e=>{if(this.moved)return;const toggleEl=e.target.closest('[data-toggle]'),node=e.target.closest('[data-node]');if(toggleEl)toggle(toggleEl.dataset.toggle);else if(node)select(node.dataset.node);});
    svg.addEventListener('keydown',e=>{if(e.key!=='Enter'&&e.key!==' ')return;const target=e.target.closest('[data-toggle],[data-node]');if(target){e.preventDefault();const attribute=target.hasAttribute('data-toggle')?'data-toggle':'data-node',id=target.getAttribute(attribute);attribute==='data-toggle'?toggle(id):select(id);svg.querySelector('['+attribute+'="'+CSS.escape(id)+'"]')?.focus({preventScroll:true});}});
    svg.addEventListener('pointerdown',e=>{if(e.target.closest('[data-node],[data-toggle]')||e.button!==0)return;this.drag={x:e.clientX,y:e.clientY,cx:this.camera.x,cy:this.camera.y};this.moved=false;svg.setPointerCapture(e.pointerId);});
    svg.addEventListener('pointermove',e=>{if(!this.drag)return;const dx=e.clientX-this.drag.x,dy=e.clientY-this.drag.y;this.moved=Math.abs(dx)+Math.abs(dy)>3;this.camera.x=this.drag.cx+dx;this.camera.y=this.drag.cy+dy;this.apply();});
    const release=()=>{this.drag=null;setTimeout(()=>{this.moved=false;},0);};svg.addEventListener('pointerup',release);svg.addEventListener('pointercancel',release);
    svg.addEventListener('wheel',e=>{e.preventDefault();const r=svg.getBoundingClientRect();this.zoom(Math.exp(-e.deltaY*.0015),e.clientX-r.left,e.clientY-r.top);},{passive:false});
    new ResizeObserver(()=>{if(this.model&&!this.initialFit){this.fit();this.initialFit=true;}}).observe(svg);
    document.fonts.ready.then(()=>{if(this.model)this.render(this.model,this.collapsed,this.selected);});
  }
  render(model,collapsed,selected){
    if(this.model!==model){this.detailPositions=new Map();this.staticPositions=new Map();}
    this.model=model;this.collapsed=collapsed;this.selected=selected;const visible=visibleGraph(model,collapsed),connections=graphConnections(visible,selected,this.referenceMode);this.visible=visible;
    this.svg.replaceChildren();const defs=element('defs');const marker=element('marker',{id:'edge-arrow',viewBox:'0 0 10 10',refX:9,refY:5,markerWidth:5,markerHeight:5,orient:'auto-start-reverse'});marker.append(element('path',{d:'M0,0 L10,5 L0,10 Z',fill:'#a1a1aa'}));defs.append(marker);this.svg.append(defs);
    this.layer=element('g');this.svg.append(this.layer);
    // Measure actual rendered fonts, including fallback fonts. Keep format data
    // untouched: these rectangles exist only in the graph presentation.
    const probe=element('g',{class:'node',visibility:'hidden'});this.layer.append(probe);
    const measure=(value,cls)=>{const sample=element('text',{class:cls});sample.textContent=value;probe.append(sample);const width=sample.getComputedTextLength();sample.remove();return width;};
    this.boxes=new Map(visible.nodes.map(n=>{
      const titleLines=locale=>wrapText(t(n.label,locale),WIDTH-40,s=>measure(s,'node-title'));
      const subtitleLines=locale=>wrapText(t(n.subtitle,locale),WIDTH-40,s=>measure(s,'node-subtitle'));
      const titles=titleLines(),subtitles=subtitleLines();
      const height=44+Math.max(titleLines('nl').length,titleLines('en').length)*20+Math.max(subtitleLines('nl').length,subtitleLines('en').length)*17+12;
      return [n.id,{...n,x:n.x*1.7,y:n.y*1.05,width:WIDTH,height,titles,subtitles}];
    }));
    probe.remove();
    const detail=n=>['field','more'].includes(n.kind);
    for(const id of this.detailPositions.keys())if(!this.boxes.has(id))this.detailPositions.delete(id);
    const reserved=[...this.detailPositions].map(([id,p])=>({...this.boxes.get(id),...p}));
    for(const box of [...this.boxes.values()].filter(n=>!detail(n))){
      const collection=model.paging.collections.get(box.collectionId),parent=this.boxes.get(box.collectionId);
      if(collection&&collection.items.length>collection.pageSize&&parent){box.x=parent.x+WIDTH+190;box.y=parent.y+(box.collectionIndex-collection.start)*150;}
      const position=(!collection&&this.staticPositions.get(box.id))||placeDetailNode(box,reserved);
      Object.assign(box,{x:position.x,y:position.y});if(!collection)this.staticPositions.set(box.id,{x:box.x,y:box.y});reserved.push(box);
    }
    for(const [id,position] of this.detailPositions){const box=this.boxes.get(id);Object.assign(box,position);reserved.push(box);}
    for(const node of visible.nodes.filter(detail)){
      if(this.detailPositions.has(node.id))continue;
      const box=this.boxes.get(node.id),parent=this.boxes.get(node.parentId);
      const collection=model.paging.collections.get(node.collectionId),slot=collection?node.collectionIndex-collection.start:(node.fieldIndex??4);
      const placed=placeDetailNode({...box,x:parent.x+parent.width+190,y:parent.y+slot*140},reserved);
      Object.assign(box,{x:placed.x,y:placed.y});this.detailPositions.set(node.id,{x:box.x,y:box.y});reserved.push(box);
    }
    const boxes=visible.nodes.map(n=>this.boxes.get(n.id));
    const ifcx=boxes.filter(n=>n.domain==='ifcx'&&!n.concept);
    if(ifcx.length){
      const x=Math.min(...ifcx.map(n=>n.x))-28,y=Math.min(...ifcx.map(n=>n.y))-48;
      const right=Math.max(...ifcx.map(n=>n.x+n.width))+28,bottom=Math.max(...ifcx.map(n=>n.y+n.height))+28;
      this.layer.append(element('rect',{x,y,width:right-x,height:bottom-y,rx:16,class:'ifcx-region'}));
      text(this.layer,'IFCX · semantische graph',x+16,y+25,'ifcx-region-title');
    }
    const occupied=boxes.map(n=>({x:n.x-4,y:n.y-4,width:n.width+22,height:n.height+8}));
    const labels=[];
    const connected=new Set(connections.edges.filter(e=>e.source===selected||e.target===selected));
    for(const edge of connections.edges){const a=this.boxes.get(edge.source),b=this.boxes.get(edge.target);
      const route=routeEdge(a,b);
      const path=element('path',{d:route.path,class:'edge'+(!edge.structural?' reference':'')+(connected.has(edge)?' connected':''),'marker-end':'url(#edge-arrow)'});this.layer.append(path);
      if(!edge.compact&&(edge.structural||connected.has(edge)))labels.push({edge,path,connected:connected.has(edge),clearance:route.kind==='vertical'&&!edge.structural?0:6});
    }
    const leaders=element('g');this.layer.append(leaders);
    for(const node of boxes){const group=element('g',{class:'node '+node.domain+(node.concept?' concept':'')+(node.id===selected?' selected':''),transform:`translate(${node.x} ${node.y})`});
      const body=element('g',{'data-node':node.id,role:'button',tabindex:0,'aria-label':t(node.label)+' · '+t(node.subtitle)+(node.concept?' · concept':''),'aria-pressed':node.id===selected,class:'node-select'});
      body.append(element('rect',{width:WIDTH,height:node.height,rx:8,class:'node-box'}));body.append(element('rect',{x:0,y:15,width:4,height:node.height-30,rx:2,class:'node-strip'}));
      text(body,node.concept?'CONCEPT / '+(node.kind==='field'?'VELD':node.domain.toUpperCase()):node.kind==='field'?'VELD / IFCDR':node.kind==='group'?'WEERGAVEGROEP':node.domain.toUpperCase(),17,21,'node-type');
      node.titles.forEach((line,i)=>text(body,line,17,44+i*20,'node-title'));
      node.subtitles.forEach((line,i)=>text(body,line,17,44+node.titles.length*20+i*17,'node-subtitle'));
      const title=element('title');title.textContent=t(node.label)+' · '+t(node.subtitle);body.append(title);group.append(body);
      if(node.expandable){const control=element('g',{'data-toggle':node.id,role:'button',tabindex:0,'aria-expanded':!collapsed.has(node.id),'aria-label':t((collapsed.has(node.id)?'Uitklappen: ':'Inklappen: ')+node.label)});control.append(element('circle',{cx:WIDTH,cy:node.height/2,r:13,class:'expand-circle'}));text(control,collapsed.has(node.id)?'+':'−',WIDTH-5,node.height/2+6,'expand-label');group.append(control);}
      this.layer.append(group);
    }
    // Labels occupy reserved free rectangles, above edges and nodes in SVG order.
    // A short leader keeps displaced labels attached to the correct relation.
    this.labelBoxes=[];
    // Structural captions retain priority when selecting additional references.
    labels.sort((a,b)=>Number(b.edge.structural)-Number(a.edge.structural));
    for(const {edge,path,connected:isConnected,clearance} of labels){
      const group=element('g',{class:'edge-caption'+(isConnected?' connected':'')});this.layer.append(group);
      const label=text(group,'',0,0,'edge-label');
      const lines=wrapText(t(edge.relation),132,value=>{label.textContent=value;return label.getComputedTextLength();});
      label.replaceChildren();
      lines.forEach((line,i)=>{const span=element('tspan',{x:0,y:i*15});span.textContent=line;label.append(span);});
      const bounds=label.getBBox();
      const length=path.getTotalLength();
      // Prefer the destination side, after sibling connections have diverged.
      const points=[.88,.8,.7,.6,.5,.35].map(t=>path.getPointAtLength(length*t));
      const box=placeLabel(points,bounds.width+14,bounds.height+10,occupied,clearance);
      occupied.push(box);this.labelBoxes.push(box);
      const cx=box.x+box.width/2,cy=box.y+box.height/2;
      if(Math.hypot(cx-box.anchor.x,cy-box.anchor.y)>32){
        const leader=element('path',{d:`M${box.anchor.x} ${box.anchor.y} L${cx} ${cy}`,class:'label-leader'});
        leaders.append(leader);
      }
      const background=element('rect',{x:box.x,y:box.y,width:box.width,height:box.height,rx:4,class:'label-background'});
      group.insertBefore(background,label);label.setAttribute('transform',`translate(${box.x+7-bounds.x} ${box.y+5-bounds.y})`);
    }
    this.callbacks.countChanged?.(visible.nodes.length,model.nodes.length);this.callbacks.referenceCountChanged?.(connections.hiddenReferences);this.apply();
  }
  apply(){this.layer?.setAttribute('transform',`translate(${this.camera.x} ${this.camera.y}) scale(${this.camera.k})`);this.callbacks.zoomChanged?.(Math.round(this.camera.k*100));}
  zoom(factor,x=this.svg.clientWidth/2,y=this.svg.clientHeight/2){const old=this.camera.k,next=Math.max(.25,Math.min(2,old*factor));this.camera.x=x-(x-this.camera.x)*next/old;this.camera.y=y-(y-this.camera.y)*next/old;this.camera.k=next;this.apply();}
  fit(){if(this.visible?.nodes.length)this.fitBoxes([...this.visible.nodes.map(n=>this.boxes.get(n.id)),...this.labelBoxes]);}
  fitBoxes(boxes){if(!boxes.length)return;const minX=Math.min(...boxes.map(n=>n.x))-40,minY=Math.min(...boxes.map(n=>n.y))-65,maxX=Math.max(...boxes.map(n=>n.x+n.width))+40,maxY=Math.max(...boxes.map(n=>n.y+n.height))+100;const w=this.svg.clientWidth,h=this.svg.clientHeight;if(!w||!h)return;const k=Math.min(1.05,w/(maxX-minX),h/(maxY-minY));this.camera={k,x:(w-(maxX-minX)*k)/2-minX*k,y:(h-(maxY-minY)*k)/2-minY*k};this.apply();}
  focus(id){const n=this.boxes.get(id);if(!n)return;const{x,y,k}=this.camera,w=this.svg.clientWidth,h=this.svg.clientHeight;const left=x+n.x*k,top=y+n.y*k;if(left<20||left+n.width*k>w-25||top<30||top+n.height*k>h-80){this.camera.x=w/2-(n.x+n.width/2)*k;this.camera.y=h/2-(n.y+n.height/2)*k;this.apply();}}
  frame(ids){this.fitBoxes(ids.map(id=>this.boxes.get(id)).filter(Boolean));}
}
