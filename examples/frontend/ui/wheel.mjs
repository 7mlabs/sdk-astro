import { BODY_IDS,GLYPHS,COLORS,SIGN_GLYPHS,localId,endpoint1,endpoint2,number } from './data.mjs';
const NS = 'http://www.w3.org/2000/svg';
function svg(tag,attributes = {},text) { const node = document.createElementNS(NS,tag); for (const [key,value] of Object.entries(attributes)) node.setAttribute(key,String(value)); if (text !== undefined) node.textContent = String(text); return node; }
function position(longitude,radius,ascendant) { const angle = (180 + ascendant - longitude) * Math.PI / 180; return [320 + radius * Math.cos(angle),320 + radius * Math.sin(angle)]; }
function line(root,a,b,attrs = {}) { root.append(svg('line',{x1:a[0],y1:a[1],x2:b[0],y2:b[1],...attrs})); }
function circle(root,r,attrs = {}) { root.append(svg('circle',{cx:320,cy:320,r,fill:'none',...attrs})); }
function labelAngles(placements) {
  const sorted = placements.map((p,index) => ({index,longitude:p.longitude,angle:p.longitude})).sort((a,b) => a.longitude - b.longitude);
  if (sorted.length < 2) return new Map(sorted.map(p => [p.index,p.angle]));
  let gap = -1; let start = 0;
  for (let i = 0; i < sorted.length; i++) { const size = (sorted[(i + 1) % sorted.length].longitude - sorted[i].longitude + 360) % 360; if (size > gap) { gap = size; start = (i + 1) % sorted.length; } }
  const rotated = [...sorted.slice(start),...sorted.slice(0,start)];
  for (let i = 0; i < rotated.length; i++) { if (i && rotated[i].longitude < rotated[i - 1].longitude) rotated[i].longitude += 360; rotated[i].angle = i ? Math.max(rotated[i].longitude,rotated[i - 1].angle + 12) : rotated[i].longitude; }
  const shift = rotated.reduce((total,p) => total + p.angle - p.longitude,0) / rotated.length;
  return new Map(rotated.map(p => [p.index,((p.angle - shift) % 360 + 360) % 360]));
}
export function renderWheel(model,{aspects = model.aspects,maxLines = 40,primaryPointIds = []} = {}) {
  const root = svg('svg',{viewBox:'0 0 640 640',class:'astro-wheel',role:'img','aria-label':`Vòng ${model.label}`});
  root.append(svg('title',{},model.label),svg('desc',{},'Kinh độ hành tinh và cusp lấy trực tiếp từ payload. ASC được đặt bên trái nếu có. Nhãn hành tinh có thể giãn để đọc, đường nối trỏ về kinh độ thật.'));
  const layers = model.layers || [model]; const reference = layers[0];
  const asc = reference.angles.find(p => localId(p.id) === 'ascendant')?.longitude || 0;
  for (const r of [275,228,164,124]) circle(root,r,{stroke:'var(--astro-line)', 'stroke-width':r === 275 ? 1.05 : .7});
  const signColors = ['#ed8a80','#a9dec7','#79d5e5','#7faaf2'];
  for (let longitude = 0; longitude < 360; longitude++) {
    const radius = longitude % 10 === 0 ? 219 : longitude % 5 === 0 ? 223 : 225;
    line(root,position(longitude,radius,asc),position(longitude,228,asc),{stroke:'var(--astro-tick)','stroke-width':.6});
  }
  for (let sign = 0; sign < 12; sign++) {
    line(root,position(sign * 30,228,asc),position(sign * 30,275,asc),{stroke:'var(--astro-line)','stroke-width':.85});
    const p = position(sign * 30 + 15,253,asc);
    root.append(svg('text',{x:p[0],y:p[1] + 7,'text-anchor':'middle',class:'astro-wheel-sign',fill:signColors[sign % 4]},SIGN_GLYPHS[sign]));
  }
  layers.forEach((layer,index) => {
    const houses = layer.houses || []; const inner = index ? 130 : 124; const outer = index ? 191 : 228;
    const cusps = Array.isArray(layer.chart?.houseCusps) ? layer.chart.houseCusps : houses.map(h => h.longitude);
    cusps.forEach((longitude,i) => {
      if (!Number.isFinite(longitude)) return;
      line(root,position(longitude,inner,asc),position(longitude,outer,asc),{stroke:index ? '#739bc1' : 'var(--astro-line)','stroke-width':.9,...(index ? {'stroke-dasharray':'3 5'} : {})});
      const next = cusps[(i + 1) % cusps.length];
      if (!Number.isFinite(next)) return;
      const center = longitude + ((next - longitude + 360) % 360) / 2;
      const p = position(center,index ? 140 : 154,asc);
      root.append(svg('text',{x:p[0],y:p[1] + 4,'text-anchor':'middle',class:'astro-wheel-house',fill:index ? '#83b2d9' : 'var(--astro-muted)'},index ? `${layer.prefix || layer.key}${i + 1}` : i + 1));
    });
  });
  const positions = new Map();
  layers.forEach((layer,index) => layer.placements.forEach(p => {
    const id = layers.length > 1 ? `${layer.prefix || layer.key}:${localId(p.id)}` : p.id;
    positions.set(id,{...p,radius:index ? 111 : 124});
  }));
  const bodyAspects = aspects.filter(a => BODY_IDS.includes(localId(endpoint1(a))) && BODY_IDS.includes(localId(endpoint2(a))) && positions.has(endpoint1(a)) && positions.has(endpoint2(a)));
  for (const aspect of bodyAspects.slice(0,maxLines)) {
    const a = positions.get(endpoint1(aspect)); const b = positions.get(endpoint2(aspect));
    const color = [60,120].includes(aspect.angle) ? '#87d9ca' : [90,180].includes(aspect.angle) ? '#e58c83' : '#86bce6';
    line(root,position(a.longitude,a.radius,asc),position(b.longitude,b.radius,asc),{stroke:color,'stroke-width':.9,opacity:.62,...([90,180].includes(aspect.angle) ? {'stroke-dasharray':'4 5'} : {})});
  }
  const selected = new Set(primaryPointIds);
  layers.forEach((layer,index) => {
    const points = layer.placements; const spread = labelAngles(points);
    const bodyRadius = index ? 181 : 201; const labelRadius = index ? 180 : 203;
    points.forEach((point,i) => {
      const id = layers.length > 1 ? `${layer.prefix || layer.key}:${localId(point.id)}` : point.id;
      const color = COLORS[localId(point.id)] || 'var(--astro-text)';
      const real = position(point.longitude,bodyRadius,asc); const projected = position(spread.get(i),labelRadius,asc);
      if (Math.abs((spread.get(i) - point.longitude + 540) % 360 - 180) > 1.5) line(root,real,projected,{stroke:color,'stroke-width':.65,opacity:.65});
      root.append(svg('circle',{cx:real[0],cy:real[1],r:selected.has(id) ? 2.6 : 1.5,fill:color}));
      const label = svg('text',{x:projected[0],y:projected[1] + 8,'text-anchor':'middle',fill:color,class:'astro-wheel-body',...(index ? {'font-size':19} : {})},GLYPHS[localId(point.id)] || localId(point.id));
      label.append(svg('title',{},`${id}: ${number(point.longitude,4)}°`)); root.append(label);
      if (layers.length > 1) root.append(svg('text',{x:projected[0] + 10,y:projected[1] - 7,'text-anchor':'start',fill:color,class:'astro-wheel-chart-label'},layer.prefix || layer.key));
    });
  });
  for (const id of ['ascendant','midheaven']) {
    const angle = reference.angles.find(p => localId(p.id) === id);
    if (!angle) continue;
    line(root,position(angle.longitude,164,asc),position(angle.longitude,285,asc),{stroke:'var(--astro-text)','stroke-width':1.25,opacity:.78});
    const p = position(angle.longitude,301,asc);
    root.append(svg('text',{x:p[0],y:p[1] + 4,'text-anchor':'middle',fill:'var(--astro-text)',class:'astro-wheel-angle'},GLYPHS[id]));
  }
  return {element:root,visibleAspectCount:Math.min(bodyAspects.length,maxLines),bodyAspectCount:bodyAspects.length};
}
