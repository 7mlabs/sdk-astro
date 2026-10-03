import { array,BODY_IDS,GLYPHS,COLORS,DOMAIN_NAMES,SECTION_NAMES,localId,pointName,number,degrees,degreeInSign,signName,endpoint1,endpoint2,factId,houseValue,dateLabel,getAstroViews,preferredChart,getDomainFacts,resolveLocalAdvanced } from './data.mjs';
import { el,button,table,boundedJson,disclosure } from './dom.mjs';
import { renderWheel } from './wheel.mjs';

let sequence = 0;
const EVENT_NAMES = { planetaryAspect:'Góc chiếu hành tinh',ingress:'Đổi cung',station:'Đổi chiều chuyển động',lunarPhase:'Pha Mặt Trăng',solarEclipse:'Nhật thực',lunarEclipse:'Nguyệt thực',natalTransit:'Transit natal' };
const motion = value => value === true ? 'Rx' : value === false ? 'D' : '—';
const applying = value => value === true ? 'Đang tiến tới' : value === false ? 'Đang rời xa' : '—';
function heading(text,description) { const h = el('div','astro-section-heading'); h.append(el('h3','',text)); if (description) h.append(el('p','astro-muted',description)); return h; }
function countLabel(count,text) { const span = el('span','astro-metric'); span.append(el('strong','',count),document.createTextNode(` ${text}`)); return span; }
function labeledSelect(label,items,value,onChange) {
  const wrap = el('label','astro-select-label',label); const select = el('select','astro-select');
  for (const item of items) { const option = el('option','',item.label); option.value = item.id; select.append(option); }
  select.value = value; select.addEventListener('change',() => onChange(select.value)); wrap.append(select); return wrap;
}
function pointCell(point) {
  const node = el('span','astro-point-name'); const id = localId(point.id); const glyph = el('span','astro-glyph',GLYPHS[id] || '');
  if (['ascendant','midheaven','descendant','imumCoeli'].includes(id)) {node.textContent=pointName(point.id);return node;}
  if (COLORS[id]) glyph.style.color = COLORS[id]; node.append(glyph,document.createTextNode(pointName(point.id))); return node;
}
function planetTable(points) {
  return table([
    {label:'Điểm',render:pointCell}, {label:'Cung',render:p => signName(p.sign)},
    {label:'Vị trí trong cung',className:'astro-numeric',render:p => degreeInSign(p.degreeInSign)},
    {label:'Nhà',render:p => p.house ? `H${p.house}` : '—'},
    {label:'Kinh độ',className:'astro-numeric',render:p => degrees(p.longitude)},
    {label:'Chuyển động',render:p => motion(p.isRetrograde)},
    {label:'Tốc độ °/ngày',className:'astro-numeric',render:p => number(p.speed,5)}
  ],points);
}
function houseTable(houses) {
  return table([
    {label:'Nhà',render:h => h.id || `H${houseValue(h).number}`},
    {label:'Cung cusp',render:h => signName(houseValue(h).sign)},
    {label:'Vị trí trong cung',className:'astro-numeric',render:h => degreeInSign(houseValue(h).degreeInSign)},
    {label:'Kinh độ cusp',className:'astro-numeric',render:h => degrees(houseValue(h).longitude)},
    {label:'Hành tinh trong nhà',render:h => array(houseValue(h).occupants).map(p => pointName(p.id)).join(', ') || '—'},
    {label:'Chủ tinh',render:h => houseValue(h).rulerBodyId ? pointName(houseValue(h).rulerBodyId) : '—'}
  ],houses);
}
function aspectTable(aspects,{limit,onMore,evidence = false} = {}) {
  const columns = [
    {label:'Tham chiếu',className:'astro-numeric',render:a => a.index === undefined ? '—' : `#${a.index}`},
    {label:'Điểm 1',render:a => pointName(endpoint1(a))}, {label:'Điểm 2',render:a => pointName(endpoint2(a))},
    {label:'Góc',className:'astro-numeric',render:a => degrees(a.angle)},
    {label:'Khoảng cách',className:'astro-numeric',render:a => degrees(a.separation)},
    {label:'Orb',className:'astro-numeric',render:a => degrees(a.orb)},
    {label:'Orb tối đa',className:'astro-numeric',render:a => degrees(a.maxOrb)},
    {label:'Applying',render:a => applying(a.applying)}
  ];
  if (evidence) columns.push({label:'Vai trò',render:a => a.primaryContact === true ? 'Chạm điểm chính' : a.primaryContact === false ? 'Hỗ trợ tiểu mục' : '—'}, {label:'Tiểu mục',render:a => array(a.sectionIds).join(', ') || '—'});
  return table(columns,aspects,{limit,onMore});
}
function wheelPanel(model,envelope,options) {
  const wrap = el('div','astro-chart-layout'); const chart = el('div','astro-wheel-area');
  const wheel = renderWheel(model,{maxLines:options.aspectLimit ?? 40}); chart.append(wheel.element);
  if (wheel.bodyAspectCount > wheel.visibleAspectCount) chart.append(el('p','astro-wheel-note',`Vòng vẽ ${wheel.visibleAspectCount} / ${wheel.bodyAspectCount} góc giữa hành tinh. Bảng Góc chiếu giữ toàn bộ contacts.`));
  if (model.layers) chart.append(el('p','astro-wheel-note',model.layers.map(layer => `${layer.prefix || layer.key}: ${layer.label}`).join(' · ')));
  wrap.append(chart); const rail = el('aside','astro-planet-rail'); rail.setAttribute('aria-label','Vị trí hành tinh');
  const primary = ['sun','moon','ascendant','midheaven'];
  const points = [...model.placements,...model.angles];
  const selected = primary.flatMap(id => points.filter(p => localId(p.id) === id));
  for (const point of selected) {
    const row = el('div','astro-planet-highlight'); row.append(pointCell(point));
    const value = el('div','astro-planet-position');value.append(el('strong','',signName(point.sign)),el('span','astro-numeric',`${degreeInSign(point.degreeInSign)}${point.isRetrograde === true ? ' Rx' : ''}`)); row.append(value);rail.append(row);
  }
  const remaining = model.placements.filter(p => !primary.includes(localId(p.id)));
  for (const point of remaining) { const row = el('div','astro-planet-compact');row.append(pointCell(point),el('span','astro-numeric',`${signName(point.sign)} ${degreeInSign(point.degreeInSign)}${point.isRetrograde === true ? ' Rx' : ''}`));rail.append(row); }
  wrap.append(rail);
  const footer = el('div','astro-chart-metrics');footer.append(countLabel(model.placements.length,'hành tinh'),countLabel(model.houses.length,'nhà'));
  const rules = array(envelope.calculation?.aspectRules);
  footer.append(el('span','astro-muted',rules.length ? `${rules.map(r => `${r.angle}°`).join(' · ')} · ${model.aspects.length} contacts` : `${model.aspects.length} contacts trong payload`));
  const surface = el('div','astro-chart-surface');surface.append(wrap,footer);return surface;
}
function primitiveSummary(data) {
  const surface = el('div','astro-query-summary');surface.append(heading('Kết quả truy vấn',`${data?.group || ''}${data?.action ? ` / ${data.action}` : ''}`));
  if (data === null || typeof data !== 'object') { surface.append(el('output','astro-value',String(data)));return surface; }
  const entries = Object.entries(data).filter(([,value]) => value === null || typeof value !== 'object');
  const list = el('dl','astro-value-list');for (const [key,value] of entries) {list.append(el('dt','',key),el('dd','astro-numeric',typeof value === 'number' ? number(value,6) : String(value)));}surface.append(list);
  for (const [key,value] of Object.entries(data).filter(([,value]) => value && typeof value === 'object')) surface.append(disclosure(key,value));
  return surface;
}
function domainPanel(envelope,model,state,update) {
  const wrap = el('div','astro-domain-panel'); const domain = model.domains.find(d => d.id === state.domain) || model.domains[0];
  if (!domain) return el('p','astro-empty','Payload này không có lĩnh vực.');
  const facts = getDomainFacts(model,domain.id,state.section); const { view,report,section } = facts;
  const controls = el('div','astro-filter-row');controls.append(labeledSelect('Lĩnh vực',model.domains.map(d => ({id:d.id,label:d.label})),domain.id,id => update({domain:id,section:''})));
  const sections = array(report.sections);controls.append(labeledSelect('Tiểu mục',[{id:'',label:'Toàn lĩnh vực'},...sections.map(s => ({id:s.id,label:SECTION_NAMES[s.id] || s.id}))],section?.id || '',id => update({section:id})));
  wrap.append(controls);
  const selected = section?.primaryPointIds ?? section?.natalPointIds ?? report.primaryPointIds ?? view.pointIds ?? view.primaryNatalPointIds;
  const focus = section?.focusHouseIds ?? section?.natalHouseIds ?? report.focusHouseIds ?? view.primaryNatalHouseIds ?? Object.entries(view.selections || {}).flatMap(([chartId,selection]) => array(selection.houses).map(h => `${chartId}:${h.id}`));
  wrap.append(heading(domain.label,'Dữ liệu và tham chiếu do engine trả; chưa tạo diễn giải hoặc điểm số.'));
  const summary = el('div','astro-domain-summary');summary.append(el('p','',`Điểm chính: ${array(selected).map(pointName).join(', ') || '—'}`),el('p','',`Nhà trọng tâm: ${array(focus).join(', ') || '—'}`));wrap.append(summary);
  if (view.eventReferences) {
    const eventIds = new Set(facts.eventIds); const events = array(envelope.data?.events).filter(event => eventIds.has(event.id));
    const highlights = new Set(section?.highlightEventIds ?? view.highlightEventIds);
    wrap.append(heading(`${events.length} sự kiện liên quan`,`${highlights.size} tham chiếu nổi bật do engine chọn.`),eventTable(events,highlights,{limit:state.limit,onMore:() => update({limit:state.limit + 80})}));
    if (facts.aspects.length) wrap.append(heading('Góc chiếu tại snapshot','Indexes thuộc snapshot; contacts trong từng sự kiện dùng indexes riêng.'),aspectTable(facts.aspects,{limit:state.limit,onMore:() => update({limit:state.limit + 80})}));
  } else {
    wrap.append(heading(`Nhà và dữ liệu hỗ trợ · ${facts.houses.length}`),houseTable(facts.houses));
    const rows = facts.bodyFacts.map(f => ({...f,facts:f.facts || f}));
    wrap.append(heading(`Trạng thái hành tinh · ${rows.length}`),table([
      {label:'Hành tinh',render:f => pointName(factId(f))}, {label:'Nguyên tố',render:f => f.facts.element},
      {label:'Tính chất',render:f => f.facts.modality}, {label:'Nhóm nhà',render:f => f.facts.houseType},
      {label:'Chuyển động',render:f => f.facts.motion ?? '—'},
      {label:'Dignity',render:f => {const d=f.facts.dignity;return d?.supported ? ['domicile','exaltation','detriment','fall'].filter(k => d[k]).join(', ') || 'none' : 'unsupported';}},
      {label:'Tương quan Mặt Trời',render:f => f.facts.solarCondition?.condition}
    ],rows));
    wrap.append(heading(`Góc chiếu có bằng chứng · ${facts.aspects.length}`),aspectTable(facts.aspects,{limit:state.limit,onMore:() => update({limit:state.limit + 80}),evidence:true}));
    for (const [key,label] of [['dispositorChains','Chuỗi chủ tinh'],['receptions','Receptions natal'],['aspectPatterns','Cấu hình góc natal']]) {
      let items = resolveLocalAdvanced(envelope,model,report,key);
      if (section) {
        const refKey = {dispositorChains:'dispositorChainRefs',receptions:'natalReceptionRefs',aspectPatterns:'natalAspectPatternRefs'}[key];
        const idKey = {dispositorChains:'dispositorChainIds',receptions:'receptionIds',aspectPatterns:'aspectPatternIds'}[key];
        const references = new Set(array(section[refKey]).map(ref => `${ref.chartId}:${ref.id}`)); const ids = new Set(array(section[idKey]));
        items = items.filter(item => references.has(`${item.chartId}:${item.fact.id}`) || ids.has(item.fact.id));
      }
      if (items.length) { const region=el('div','astro-advanced');region.append(heading(`${label} · ${items.length}`));for(const item of items) region.append(disclosure(`${item.chartId ? `${item.chartId} · ` : ''}${item.fact.id}`,item.fact));wrap.append(region); }
    }
    const overlayIds = new Set(array(section?.overlayIds ?? report.overlayIds));
    const overlays = [...array(model.context.overlaysAtoB),...array(model.context.overlaysBtoA)].filter(o => overlayIds.has(o.id));
    if (overlays.length) wrap.append(disclosure(`House overlays · ${overlays.length}`,overlays));
    const relationIds = new Set(array(section?.houseRulerRelationIds ?? report.houseRulerRelationIds));
    const rulers = array(model.context.houseRulerRelations).filter(r => relationIds.has(r.id));
    if (rulers.length) wrap.append(disclosure(`Liên kết chủ tinh các nhà · ${rulers.length}`,rulers));
  }
  wrap.append(disclosure('Định nghĩa selectors và phạm vi', {definition:view.definition,selection:section?.selectionRules ?? view.selectionRules,coverage:report.coverage,section:section?.id || null}));
  return wrap;
}
function eventTable(events,highlights,{limit,onMore} = {}) {
  return table([
    {label:'Thời gian local',className:'astro-numeric',render:event => dateLabel(event.local)},
    {label:'Sự kiện',render:event => EVENT_NAMES[event.type] || event.type},
    {label:'Hành tinh',render:event => array(event.bodyIds).map(pointName).join(', ')},
    {label:'Thông số',render:event => {const d=event.details || {};return [d.angle !== undefined ? degrees(d.angle) : '',d.targetPointId ? pointName(d.targetPointId) : '',d.toSign ?? d.direction ?? d.classification ?? ''].filter(Boolean).join(' · ') || '—';}},
    {label:'Nổi bật',render:event => highlights.has(event.id) ? 'Có' : '—'},
    {label:'Dữ liệu',render:event => disclosure('Xem',event,{jsonLimit:6000})}
  ],events,{limit,onMore});
}
function eventPanel(envelope,state,update) {
  const data = envelope.data;const events = array(data.events); const period = data.period;
  const wrap = el('div','astro-event-panel');const types = [...new Set(events.map(e => e.type))];
  const controls = el('div','astro-filter-row'); controls.append(labeledSelect('Loại sự kiện',[{id:'',label:'Tất cả'},...types.map(type => ({id:type,label:EVENT_NAMES[type] || type}))],state.eventType,type => update({eventType:type})));
  wrap.append(controls,heading(`${events.length} sự kiện trong kỳ`,period ? `${dateLabel(period.startUtc)} → ${dateLabel(period.endUtc)} UTC · mốc cuối không bao gồm.` : undefined));
  if (data.overview) {const summary=el('div','astro-event-counts'); for(const [type,count] of Object.entries(data.overview.byType || {})) summary.append(countLabel(count,EVENT_NAMES[type] || type));wrap.append(summary);}
  const filtered = events.filter(e => !state.eventType || e.type === state.eventType);
  wrap.append(eventTable(filtered,new Set(array(data.overview?.highlightEventIds)),{limit:state.limit,onMore:() => update({limit:state.limit + 80})}));
  if (data.snapshot) wrap.append(disclosure('Mốc snapshot', {utc:data.snapshot.utc,local:data.snapshot.local,julianDayUt1:data.snapshot.julianDayUt1}));
  if (data.coverage) wrap.append(disclosure('Phạm vi tìm sự kiện',data.coverage));
  return wrap;
}

/** Render one unchanged engine envelope. The caller owns provenance and full JSON download. */
export function mountAstroResult(element,envelope,options = {}) {
  if (!element?.replaceChildren) throw new TypeError('element must be a DOM element');
  if (!envelope || typeof envelope !== 'object' || Array.isArray(envelope)) throw new TypeError('envelope must be an engine result object');
  const id = `astro-result-${++sequence}`;const models=getAstroViews(envelope);const errors=array(envelope.errors);const warnings=array(envelope.warnings);
  const state = {chart:options.initialChart || preferredChart(envelope,models),tab:options.initialTab || 'chart',domain:options.initialDomain || '',section:'',eventType:'',limit:80};let destroyed=false;
  const update = patch => {Object.assign(state,patch);if (!('limit' in patch)) state.limit=80;render();};
  function render() {
    if (destroyed) return;const previouslyFocused = element.contains(document.activeElement) ? document.activeElement?.getAttribute('data-astro-focus') : null;
    const root = el('section','astro-result');root.dataset.chart=state.chart || '';root.dataset.tab=state.tab;
    if (errors.length) {const alert=el('div','astro-engine-error');alert.setAttribute('role','alert');alert.append(heading('Engine từ chối request'));for(const error of errors) alert.append(el('p','',`${error.code}: ${error.message}`));root.append(alert,boundedJson(envelope,options.jsonPreviewLength));element.replaceChildren(root);return;}
    const model=models.find(m => m.key === state.chart) || models[0];if(model) state.chart=model.key;
    if(model && !model.domains.some(d => d.id === state.domain)) {state.domain=model.domains[0]?.id || '';state.section='';}
    const tabs=[['chart',model?.placements.length || model?.angles.length ? 'Lá số' : 'Kết quả'],['planets','Hành tinh'],['houses','Nhà'],['aspects','Góc chiếu']].filter(([key]) => key === 'chart' || key === 'planets' && (model?.placements.length || model?.angles.length) || key === 'houses' && model?.houses.length || key === 'aspects' && model);
    if(model?.domains.length) tabs.push(['domains','Lĩnh vực']);if(Array.isArray(envelope.data?.events)) tabs.push(['events','Sự kiện']);tabs.push(['json','JSON']);
    if(!tabs.some(([key]) => key === state.tab)) state.tab='chart';root.dataset.tab=state.tab;
    const tablist=el('div','astro-tabs');tablist.setAttribute('role','tablist');tablist.setAttribute('aria-label','Xem kết quả engine');
    tabs.forEach(([key,label],index) => {const tab=button(label,`astro-tab${state.tab === key ? ' is-active' : ''}`,() => update({tab:key}));tab.id=`${id}-tab-${key}`;tab.setAttribute('role','tab');tab.setAttribute('aria-selected',String(state.tab===key));tab.setAttribute('aria-controls',`${id}-panel`);tab.tabIndex=state.tab===key ? 0 : -1;tab.dataset.astroFocus=`tab-${key}`;
      tab.addEventListener('keydown',event => {if(!['ArrowLeft','ArrowRight','Home','End'].includes(event.key)) return;event.preventDefault();const next=event.key==='Home'?0:event.key==='End'?tabs.length-1:(index+(event.key==='ArrowRight'?1:-1)+tabs.length)%tabs.length;update({tab:tabs[next][0]});element.querySelector(`[data-astro-focus="tab-${tabs[next][0]}"]`)?.focus();});tablist.append(tab);});root.append(tablist);
    if(models.length>1 && !['events','json'].includes(state.tab)) {const choices=el('div','astro-chart-choices');choices.setAttribute('aria-label','Chọn lá số');for(const m of models){const choice=button(m.label,`astro-chart-choice${m.key===state.chart?' is-active':''}`,()=>update({chart:m.key,section:''}));choice.setAttribute('aria-pressed',String(m.key===state.chart));choice.dataset.astroFocus=`chart-${m.key}`;choices.append(choice);}root.append(choices);}
    const panel=el('div','astro-tab-panel');panel.id=`${id}-panel`;panel.setAttribute('role','tabpanel');panel.setAttribute('aria-labelledby',`${id}-tab-${state.tab}`);
    if(state.tab==='chart') panel.append(model?.placements.length || model?.angles.length ? wheelPanel(model,envelope,options) : primitiveSummary(envelope.data));
    if(state.tab==='planets') {panel.append(heading(`${model.placements.length} hành tinh · ${model.angles.length} góc trục`),planetTable([...model.placements,...model.angles]));}
    if(state.tab==='houses') panel.append(heading(`${model.houses.length} nhà`,'Cusp, hành tinh trong nhà và chủ tinh hiển thị khi payload cung cấp.'),houseTable(model.houses));
    if(state.tab==='aspects') panel.append(heading(`${model.aspects.length} góc chiếu`,'— nghĩa là engine không cung cấp giá trị hoặc không áp dụng cho chart này.'),aspectTable(model.aspects,{limit:state.limit,onMore:()=>update({limit:state.limit+80})}));
    if(state.tab==='domains') panel.append(domainPanel(envelope,model,state,update));
    if(state.tab==='events') panel.append(eventPanel(envelope,state,update));
    if(state.tab==='json') {if(options.onDownload) panel.append(button('Tải JSON đầy đủ','astro-small-button',()=>options.onDownload(envelope)));panel.append(boundedJson(envelope,options.jsonPreviewLength ?? 24000));}
    root.append(panel);
    if(warnings.length) {const notes=el('details','astro-calculation-notes');notes.append(el('summary','',`Ghi chú tính toán · ${warnings.length}`));for(const warning of warnings) notes.append(el('p','astro-warning',`${warning.code ? `${warning.code}: ` : ''}${warning.message || String(warning)}`));root.append(notes);}
    element.replaceChildren(root);if(previouslyFocused) root.querySelector(`[data-astro-focus="${previouslyFocused}"]`)?.focus();
  }
  render();return Object.freeze({destroy(){destroyed=true;element.replaceChildren();},selectChart(chart){if(!models.some(m=>m.key===chart)) throw new RangeError('Unknown chart');update({chart,section:''});},selectDomain(domain,section=''){const model=models.find(m=>m.key===state.chart);if(!model?.domains.some(d=>d.id===domain)) throw new RangeError('Unknown domain');update({domain,section,tab:'domains'});},selectTab(tab){update({tab});},getState(){return {...state};}});
}

/** Inject an asyncCalculate bridge. This module does not import native addons or calculate in the browser. */
export function createAstroUI({element,calculate,...options}) {
  if(typeof calculate !== 'function') throw new TypeError('calculate must be an injected async function');
  if(!element?.replaceChildren) throw new TypeError('element must be a DOM element');
  let generation=0;let closed=false;let view;
  return Object.freeze({
    async calculateRequest(request) {
      if(closed) throw new Error('Astro UI is closed');const current=++generation;view?.destroy();view=undefined;
      const pending=el('p','astro-loading','Đang tính bằng engine…');pending.setAttribute('role','status');element.replaceChildren(pending);element.setAttribute('aria-busy','true');
      try {const envelope=await calculate(request);if(!closed && current===generation){element.setAttribute('aria-busy','false');view=mountAstroResult(element,envelope,options);}return envelope;}
      catch(error){if(!closed && current===generation){element.setAttribute('aria-busy','false');const alert=el('p','astro-engine-error',`Host không thực hiện được request: ${error.message || String(error)}`);alert.setAttribute('role','alert');element.replaceChildren(alert);}throw error;}
    },
    render(envelope){if(closed) throw new Error('Astro UI is closed');++generation;element.setAttribute('aria-busy','false');view?.destroy();view=mountAstroResult(element,envelope,options);return view;},
    close(){closed=true;++generation;view?.destroy();element.removeAttribute('aria-busy');element.replaceChildren();}
  });
}
