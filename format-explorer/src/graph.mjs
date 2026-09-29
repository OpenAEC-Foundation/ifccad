import { graphConnections, visibleGraph } from './model.mjs';
import { placeLabel, routeEdge, wrapText } from './layout.mjs';
import { familyLayout, branchBusY, branchRoute } from './family-layout.mjs';
import { t } from './i18n.mjs';
const NS='http://www.w3.org/2000/svg',WIDTH=260;
const element=(tag,attrs={})=>{const e=document.createElementNS(NS,tag);for(const[k,v]of Object.entries(attrs))e.setAttribute(k,v);return e;};
const text=(parent,value,x,y,cls)=>{const e=element('text',{x,y,class:cls});e.textContent=t(value);parent.append(e);return e;};
export class PackageGraph {
  constructor(svg,{select,toggle,scrollCollection,zoomChanged,countChanged,referenceCountChanged}){
    this.svg=svg;this.callbacks={select,toggle,scrollCollection,zoomChanged,countChanged,referenceCountChanged};this.camera={x:20,y:30,k:1};this.drag=null;this.referenceMode='focus';this.viewportBoxes=[];
    svg.addEventListener('click',e=>{if(this.moved)return;const toggleEl=e.target.closest('[data-toggle]'),node=e.target.closest('[data-node]');if(toggleEl)toggle(toggleEl.dataset.toggle);else if(node)select(node.dataset.node);});
    svg.addEventListener('keydown',e=>{const bar=e.target.closest('[data-scrollbar]');if(bar){const steps={ArrowUp:-1,ArrowDown:1,PageUp:-3,PageDown:3,Home:-Infinity,End:Infinity};if(Object.hasOwn(steps,e.key)){e.preventDefault();const id=bar.dataset.scrollbar,w=this.model.paging.graphWindow(id);const delta=e.key==='Home'?-w.start:e.key==='End'?w.total-w.start:steps[e.key];this.callbacks.scrollCollection?.(id,delta);this.svg.querySelector('[data-scrollbar="'+CSS.escape(id)+'"]')?.focus({preventScroll:true});}return;}if(e.key!=='Enter'&&e.key!==' ')return;const target=e.target.closest('[data-toggle],[data-node]');if(target){e.preventDefault();const attribute=target.hasAttribute('data-toggle')?'data-toggle':'data-node',id=target.getAttribute(attribute);attribute==='data-toggle'?toggle(id):select(id);svg.querySelector('['+attribute+'="'+CSS.escape(id)+'"]')?.focus({preventScroll:true});}});
    svg.addEventListener('pointerdown',e=>{const bar=e.target.closest('[data-scrollbar]');if(bar){const id=bar.dataset.scrollbar,frame=this.viewportBoxes.find(v=>v.id===id),w=this.model.paging.graphWindow(id);if(frame){const p=this.point(e),ratio=Math.max(0,Math.min(1,(p.y-frame.y-20)/(frame.height-40)));this.callbacks.scrollCollection?.(id,Math.round(ratio*Math.max(0,w.total-3))-w.start);}return;}if(e.target.closest('[data-node],[data-toggle]')||e.button!==0)return;this.drag={x:e.clientX,y:e.clientY,cx:this.camera.x,cy:this.camera.y};this.moved=false;svg.setPointerCapture(e.pointerId);});
    svg.addEventListener('pointermove',e=>{if(!this.drag)return;const dx=e.clientX-this.drag.x,dy=e.clientY-this.drag.y;this.moved=Math.abs(dx)+Math.abs(dy)>3;this.camera.x=this.drag.cx+dx;this.camera.y=this.drag.cy+dy;this.apply();});
    const release=()=>{this.drag=null;setTimeout(()=>{this.moved=false;},0);};svg.addEventListener('pointerup',release);svg.addEventListener('pointercancel',release);
    svg.addEventListener('wheel',e=>{e.preventDefault();const p=this.point(e),window=this.viewportBoxes.find(v=>p.x>=v.x&&p.x<=v.x+v.width&&p.y>=v.y&&p.y<=v.y+v.height);if(window){this.callbacks.scrollCollection?.(window.id,e.deltaY>0?1:-1);return;}const r=svg.getBoundingClientRect();this.zoom(Math.exp(-e.deltaY*.0015),e.clientX-r.left,e.clientY-r.top);},{passive:false});
    new ResizeObserver(()=>{if(this.model&&!this.initialFit){this.fit();this.initialFit=true;}}).observe(svg);
    document.fonts.ready.then(()=>{if(this.model)this.render(this.model,this.collapsed,this.selected);});
  }
  render(model,collapsed,selected){
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
    familyLayout(this.boxes,visible.edges,model.roots);
    const boxes=visible.nodes.map(n=>this.boxes.get(n.id));
    const ifcx=boxes.filter(n=>n.domain==='ifcx'&&!n.concept);
    if(ifcx.length){
      const x=Math.min(...ifcx.map(n=>n.x))-28,y=Math.min(...ifcx.map(n=>n.y))-48;
      const right=Math.max(...ifcx.map(n=>n.x+n.width))+28,bottom=Math.max(...ifcx.map(n=>n.y+n.height))+28;
      this.layer.append(element('rect',{x,y,width:right-x,height:bottom-y,rx:16,class:'ifcx-region'}));
      text(this.layer,'IFCX · semantische graph',x+16,y+25,'ifcx-region-title');
    }
    this.viewportBoxes=[];
    for(const [id,state] of model.paging.collections){
      if(state.items.length<=3||collapsed.has(id)||!this.boxes.has(id))continue;
      const rows=boxes.filter(n=>n.collectionId===id);if(!rows.length)continue;
      const x=Math.min(...rows.map(n=>n.x))-12,y=Math.min(...rows.map(n=>n.y))-14;
      const width=WIDTH+48,height=Math.max(3*145,Math.max(...rows.map(n=>n.y+n.height))-y+18);
      const frame={id,x,y,width,height};this.viewportBoxes.push(frame);
      const group=element('g',{class:'graph-window'});group.append(element('rect',{x,y,width,height,rx:10,class:'graph-window-frame'}));
      const sc=element('g',{'data-scrollbar':id,tabindex:0,role:'scrollbar','aria-label':t('Scroll door items'),'aria-valuemin':0,'aria-valuemax':Math.max(0,state.items.length-3),'aria-valuenow':state.graphStart});
      const trackX=x+width-21;sc.append(element('rect',{x:trackX,y:y+15,width:10,height:height-30,rx:5,class:'graph-window-track'}));
      const travel=height-80,thumbY=y+20+travel*(state.graphStart/Math.max(1,state.items.length-3));
      sc.append(element('rect',{x:trackX-2,y:thumbY,width:14,height:38,rx:6,class:'graph-window-thumb'}));group.append(sc);
      text(group,`${state.graphStart+1}–${Math.min(state.graphStart+3,state.items.length)} / ${state.items.length}`,x+8,y-7,'graph-window-count');
      if(selected&&model.byId.get(selected)?.collectionId===id&&!rows.some(n=>n.id===selected))text(group,'●',x+width-30,y-7,'graph-window-selection');
      this.layer.append(group);
    }
    const occupied=boxes.map(n=>({x:n.x-4,y:n.y-4,width:n.width+22,height:n.height+8}));
    const labels=[];
    const connected=new Set(connections.edges.filter(e=>e.source===selected||e.target===selected));
    const branched=new Set(),windowLinks=new Set();
    for(const edge of connections.edges){const a=this.boxes.get(edge.source),b=this.boxes.get(edge.target);
      const viewport=this.viewportBoxes.find(v=>v.id===edge.source);
      if(edge.structural&&viewport&&b.collectionId===edge.source){
        if(!windowLinks.has(edge.source)){
          windowLinks.add(edge.source);
          const sx=a.x+a.width,sy=a.y+a.height/2,tx=viewport.x,ty=viewport.y+38,middle=(sx+tx)/2;
          this.layer.append(element('path',{d:`M${sx} ${sy} C${middle} ${sy},${middle} ${ty},${tx} ${ty}`,class:'edge'+(edge.source===selected?' connected':''),'marker-end':'url(#edge-arrow)'}));
        }
        continue;
      }
      const siblings=connections.edges.filter(other=>other.structural&&other.source===edge.source&&this.boxes.has(other.target));
      const fan=edge.structural&&siblings.length>1&&['Drawing','drawing-resource'].includes(a.kind);
      const branchKey=edge.source;
      const busY=fan?branchBusY(a,siblings.map(other=>this.boxes.get(other.target))):0;
      if(fan&&!branched.has(branchKey)){this.layer.append(element('path',{d:`M${a.x+a.width/2} ${a.y+a.height} L${a.x+a.width/2} ${busY}`,class:'edge'}));branched.add(branchKey);}
      const route=fan?{kind:'branch',path:branchRoute(a,b,busY)}:routeEdge(a,b,edge.relation,edge.structural?undefined:boxes.filter(box=>box.id!==a.id&&box.id!==b.id));
      const path=element('path',{d:route.path,class:'edge'+(!edge.structural?' reference':'')+(connected.has(edge)?' connected':''),'marker-end':'url(#edge-arrow)'});this.layer.append(path);
      if(!edge.compact&&(edge.structural||connected.has(edge)))labels.push({edge,path,connected:connected.has(edge),clearance:route.kind==='vertical'&&!edge.structural?0:6});
    }
    const leaders=element('g');this.layer.append(leaders);
    for(const node of boxes){const group=element('g',{class:'node '+node.domain+(node.concept?' concept':'')+(node.id===selected?' selected':''),transform:`translate(${node.x} ${node.y})`});
      const body=element('g',{'data-node':node.id,role:'button',tabindex:0,'aria-label':t(node.label)+' · '+t(node.subtitle)+(node.concept?' · concept':''),'aria-pressed':node.id===selected,class:'node-select'});
      body.append(element('rect',{width:WIDTH,height:node.height,rx:8,class:'node-box'}));body.append(element('rect',{x:0,y:15,width:4,height:node.height-30,rx:2,class:'node-strip'}));
      text(body,node.concept?'CONCEPT / '+(node.kind==='field'?'VELD':node.domain.toUpperCase()):node.kind==='field'?'VELD / IFCDR':['group','workspace-group'].includes(node.kind)?'WEERGAVEGROEP':node.kind==='reference-link'?'VERWIJZING':node.domain.toUpperCase(),17,21,'node-type');
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
  point(event){const rect=this.svg.getBoundingClientRect();return {x:(event.clientX-rect.left-this.camera.x)/this.camera.k,y:(event.clientY-rect.top-this.camera.y)/this.camera.k};}
  apply(){this.layer?.setAttribute('transform',`translate(${this.camera.x} ${this.camera.y}) scale(${this.camera.k})`);this.callbacks.zoomChanged?.(Math.round(this.camera.k*100));}
  zoom(factor,x=this.svg.clientWidth/2,y=this.svg.clientHeight/2){const old=this.camera.k,next=Math.max(.25,Math.min(2,old*factor));this.camera.x=x-(x-this.camera.x)*next/old;this.camera.y=y-(y-this.camera.y)*next/old;this.camera.k=next;this.apply();}
  fit(){if(this.visible?.nodes.length)this.fitBoxes([...this.visible.nodes.map(n=>this.boxes.get(n.id)),...this.labelBoxes]);}
  fitBoxes(boxes){if(!boxes.length)return;const minX=Math.min(...boxes.map(n=>n.x))-40,minY=Math.min(...boxes.map(n=>n.y))-65,maxX=Math.max(...boxes.map(n=>n.x+n.width))+40,maxY=Math.max(...boxes.map(n=>n.y+n.height))+100;const w=this.svg.clientWidth,h=this.svg.clientHeight;if(!w||!h)return;const k=Math.min(1.05,w/(maxX-minX),h/(maxY-minY));this.camera={k,x:(w-(maxX-minX)*k)/2-minX*k,y:(h-(maxY-minY)*k)/2-minY*k};this.apply();}
  focus(id){const n=this.boxes.get(id);if(!n)return;this.keepVisible(n,20,80);}
  focusBranch(id){const parent=this.boxes.get(id),window=this.viewportBoxes.find(v=>v.id===id);
    const children=[...this.boxes.values()].filter(n=>n.collectionId===id);
    const parts=[parent,window,...children].filter(Boolean);if(!parts.length)return;
    const x=Math.min(...parts.map(n=>n.x)),y=Math.min(...parts.map(n=>n.y)),right=Math.max(...parts.map(n=>n.x+n.width)),bottom=Math.max(...parts.map(n=>n.y+n.height));
    this.keepVisible({x,y,width:right-x,height:bottom-y},20,80);
  }
  keepVisible(box,topPadding=20,bottomPadding=70){const c=this.camera,k=c.k,w=this.svg.clientWidth,h=this.svg.clientHeight;
    const left=c.x+box.x*k,right=c.x+(box.x+box.width)*k,top=c.y+box.y*k,bottom=c.y+(box.y+box.height)*k;
    let dx=right>w-25?w-25-right:0;
    if(left+dx<20&&box.width*k<w-45)dx=20-left;
    let dy=bottom>h-bottomPadding?h-bottomPadding-bottom:0;
    if(top+dy<topPadding&&box.height*k<h-topPadding-bottomPadding)dy=topPadding-top;
    c.x+=dx;c.y+=dy;
    this.apply();
  }
  frame(ids){this.fitBoxes(ids.map(id=>this.boxes.get(id)).filter(Boolean));}
}
