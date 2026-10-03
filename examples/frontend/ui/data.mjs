// Payload adapters only. This module never calculates astronomical positions.
export const BODY_IDS = ['sun','moon','mercury','venus','mars','jupiter','saturn','uranus','neptune','pluto'];
export const GLYPHS = { sun:'☉',moon:'☽',mercury:'☿',venus:'♀',mars:'♂',jupiter:'♃',saturn:'♄',uranus:'♅',neptune:'♆',pluto:'♇',ascendant:'ASC',midheaven:'MC',descendant:'DSC',imumCoeli:'IC' };
export const NAMES = { sun:'Mặt Trời',moon:'Mặt Trăng',mercury:'Sao Thủy',venus:'Sao Kim',mars:'Sao Hỏa',jupiter:'Sao Mộc',saturn:'Sao Thổ',uranus:'Thiên Vương',neptune:'Hải Vương',pluto:'Diêm Vương',ascendant:'ASC',midheaven:'MC',descendant:'DSC',imumCoeli:'IC' };
export const SIGNS = ['Aries','Taurus','Gemini','Cancer','Leo','Virgo','Libra','Scorpio','Sagittarius','Capricorn','Aquarius','Pisces'];
export const SIGN_NAMES = ['Bạch Dương','Kim Ngưu','Song Tử','Cự Giải','Sư Tử','Xử Nữ','Thiên Bình','Bọ Cạp','Nhân Mã','Ma Kết','Bảo Bình','Song Ngư'];
export const SIGN_GLYPHS = ['♈','♉','♊','♋','♌','♍','♎','♏','♐','♑','♒','♓'];
export const COLORS = { sun:'#f4c475',moon:'#7eb2ff',mercury:'#76d9e8',venus:'#9be2ca',mars:'#ee877f',jupiter:'#a9e0cb',saturn:'#bdce91',uranus:'#85d7d5',neptune:'#8de2d0',pluto:'#bdadee' };
export const DOMAIN_NAMES = { career:'Công việc',love:'Tình cảm',relationships:'Quan hệ',family:'Gia đình',finance:'Tài chính',identity:'Bản thân',learning:'Học tập',creativity:'Sáng tạo',innerLife:'Nội tâm',dailyLife:'Đời sống',attraction:'Thu hút',communication:'Giao tiếp',emotionalConnection:'Kết nối cảm xúc',longTerm:'Dài hạn',sharedResources:'Nguồn lực chung',homeFamily:'Tổ ấm / gia đình' };
export const SECTION_NAMES = { profession:'Nghề nghiệp',workHabits:'Thói quen làm việc',resources:'Nguồn lực',romance:'Lãng mạn',marriage:'Hôn nhân',intimacy:'Thân mật',communication:'Giao tiếp',partnerships:'Hợp tác',friendships:'Bạn bè',roots:'Gốc rễ',home:'Nhà ở',parenting:'Nuôi dưỡng',personalResources:'Nguồn lực cá nhân',income:'Thu nhập',coreIdentity:'Bản sắc',emotionalStyle:'Cảm xúc',personalPresentation:'Thể hiện bản thân',thinking:'Tư duy',education:'Giáo dục',exploration:'Khám phá',selfExpression:'Tự biểu đạt',creativeThinking:'Tư duy sáng tạo',play:'Vui chơi',emotionalRoots:'Gốc rễ cảm xúc',transformation:'Chuyển hóa',reflection:'Chiêm nghiệm',routines:'Thói quen',selfCare:'Chăm sóc bản thân',rest:'Nghỉ ngơi',chemistry:'Sức hút',dialogue:'Đối thoại',understanding:'Thấu hiểu',conflict:'Xung đột',emotionalNeeds:'Nhu cầu cảm xúc',emotionalSafety:'An toàn cảm xúc',empathy:'Đồng cảm',commitment:'Cam kết',sharedDirection:'Định hướng chung',resilience:'Sức bền',values:'Giá trị',jointResources:'Nguồn lực chung',practicalCooperation:'Hợp tác thực tế',domesticLife:'Đời sống gia đình',familyBonds:'Gắn kết gia đình' };
export const array = value => Array.isArray(value) ? value : [];
export const localId = id => typeof id === 'string' ? id.split(':').at(-1) : '';
export const pointName = id => {
  if (typeof id !== 'string') return '—';
  const local = localId(id); const name = NAMES[local] || local;
  return id.includes(':') ? `${id.slice(0,id.lastIndexOf(':'))} · ${name}` : name;
};
export const number = (value, places = 2) => typeof value === 'number' && Number.isFinite(value) ? value.toLocaleString('en-US',{minimumFractionDigits:places,maximumFractionDigits:places}) : '—';
export const degrees = value => typeof value === 'number' && Number.isFinite(value) ? `${number(value,2)}°` : '—';
export const degreeInSign = value => {
  if (typeof value !== 'number' || !Number.isFinite(value)) return '—';
  const total = Math.round(value * 60); return `${Math.floor(total / 60)}° ${String(total % 60).padStart(2,'0')}′`;
};
export const signName = sign => SIGN_NAMES[SIGNS.indexOf(sign)] || sign || '—';
export const endpoint1 = a => a.point1 ?? a.body1 ?? a.transitPointId;
export const endpoint2 = a => a.point2 ?? a.body2 ?? a.natalPointId;
export const factId = f => f.id ?? f.bodyId ?? f.facts?.bodyId;
export const houseValue = h => h.house && typeof h.house === 'object' ? h.house : h;
export const pointValue = p => p.placement ?? p;
export function dateLabel(value) {
  if (!value || typeof value !== 'object') return '—';
  const z = n => String(n ?? 0).padStart(2,'0');
  const date = `${value.year}-${z(value.month)}-${z(value.day)}`;
  return value.hour === undefined ? date : `${date} ${z(value.hour)}:${z(value.minute)}`;
}
function domainEntries(container) {
  return [...Object.entries(container?.domains || {}).map(([id,view]) => ({id,label:DOMAIN_NAMES[id] || id,view,custom:false})),
    ...Object.entries(container?.customDomains || {}).map(([id,view]) => ({id,label:`${id} · tùy chỉnh`,view,custom:true}))];
}
function single(key,label,chart,context = {},container = {},prefix = '') {
  const placements = array(chart?.placements ?? chart?.positions).filter(p => Number.isFinite(p.longitude));
  const angles = array(chart?.angles).filter(p => Number.isFinite(p.longitude));
  const houses = array(context?.houses).length ? array(context.houses) : array(chart?.houses);
  const points = array(context?.points).length ? array(context.points) : [...placements,...angles];
  return { key,label,chart,context,placements,angles,houses,points,
    aspects:Array.isArray(context?.aspects) ? array(context.aspects) : array(chart?.aspects),
    domains:domainEntries(container),prefix,layers:null };
}
export function getAstroViews(envelope) {
  const data = envelope?.data;
  if (!data || typeof data !== 'object') return [];
  const views = [];
  if (data.natal) views.push(single('natal','Lá số natal',data.natal,data.context,data));
  else if (Array.isArray(data.placements)) views.push(single('natal',data.harmonic ? `Harmonic ${data.harmonic}` : 'Lá số',data,data.context,data));
  if (Array.isArray(data.personA) && Array.isArray(data.personB)) {
    const a = single('A','Điểm cung cấp A',{placements:data.personA},{},{},'A');
    const b = single('B','Điểm cung cấp B',{placements:data.personB},{},{},'B');
    views.push({key:'synastry',label:'Synastry điểm A / B',chart:a.chart,context:{},
      placements:[...a.placements.map(p => ({...p,id:`A:${p.id}`})),...b.placements.map(p => ({...p,id:`B:${p.id}`}))],
      angles:[],houses:[],points:[],aspects:array(data.crossAspects).map(a => ({...a,point1:`A:${a.body1}`,point2:`B:${a.body2}`})),domains:[],layers:[a,b]},a,b);
  }
  for (const [id,subject] of Object.entries(data.subjects || {})) {
    if (subject.natal) views.push(single(id,`Natal ${id}`,subject.natal,subject.context,{},id));
  }
  const a = views.find(v => v.key === 'A'); const b = views.find(v => v.key === 'B');
  if (a && b && array(data.context?.points).some(p => p.chartId === 'A')) {
    views.unshift({key:'synastry',label:'Synastry A / B',chart:a.chart,context:data.context,
      placements:[...a.placements.map(p => ({...p,id:`A:${p.id}`,chartId:'A',localId:p.id})),...b.placements.map(p => ({...p,id:`B:${p.id}`,chartId:'B',localId:p.id}))],
      angles:[...a.angles.map(p => ({...p,id:`A:${p.id}`,chartId:'A'})),...b.angles.map(p => ({...p,id:`B:${p.id}`,chartId:'B'}))],
      houses:[...a.houses.map(h => ({...h,id:`A:${h.id || `H${h.number}`}`,chartId:'A'})),...b.houses.map(h => ({...h,id:`B:${h.id || `H${h.number}`}`,chartId:'B'}))],
      points:array(data.context.points),aspects:array(data.context.aspects),domains:domainEntries(data),layers:[a,b]});
  }
  if (data.composite?.chart) views.push(single('C','Composite C',data.composite.chart,data.composite.context,data.composite,'C'));
  if (data.subject?.natal) {
    const natal = single('N','Natal',data.subject.natal,data.subject.context,{},'N');
    const transit = single('T','Transit snapshot',{positions:data.snapshot?.positions}, {}, {}, 'T');
    views.push({key:'transits',label:'Natal / Transit',chart:natal.chart,context:{},
      placements:[...natal.placements.map(p => ({...p,id:`N:${p.id}`,chartId:'N',localId:p.id})),...transit.placements.map(p => ({...p,id:`T:${p.id}`,chartId:'T',localId:p.id}))],
      angles:natal.angles,houses:natal.houses,points:natal.points,aspects:array(data.snapshot?.aspects),domains:domainEntries(data),layers:[natal,transit]},natal,transit);
  } else if (data.snapshot?.positions) views.push(single('snapshot','Bầu trời snapshot',{positions:data.snapshot.positions}));
  if (!views.length && Array.isArray(data.points)) {
    const chart = {placements:data.points.filter(p => p.kind === 'body'),angles:data.points.filter(p => p.kind === 'angle'),houses:data.houses};
    views.push(single('query','Các điểm natal',chart,{points:data.points,houses:data.houses,aspects:data.aspects},data));
  } else if (!views.length && Array.isArray(data.positions)) views.push(single('query','Các điểm',data));
  return views;
}
export function preferredChart(envelope,views) {
  if (envelope?.data?.chartKind === 'compositeRelationship' || envelope?.calculation?.scope === 'midpoint-composite-data') return views.find(v => v.key === 'C')?.key || views[0]?.key;
  return views[0]?.key;
}
export function getDomainFacts(model,domainId,sectionId = '') {
  const domain = model?.domains.find(d => d.id === domainId);
  if (!domain) return null;
  const view = domain.view; const report = view.report || view;
  const section = array(report.sections).find(s => s.id === sectionId);
  const indexes = section ? new Set(array(section.aspectIndexes ?? section.snapshotAspectIndexes)) : null;
  const houseIds = section ? new Set(array(section.houseIds ?? section.relatedHouseIds ?? section.natalHouseIds)) : null;
  const bodyIds = section ? new Set(array(section.bodyFactIds)) : null;
  return {domain,view,report,section,
    aspects:array(report.aspectFacts ?? report.aspects ?? (view.snapshotAspectIndexes ? model.aspects.filter(a => view.snapshotAspectIndexes.includes(a.index)) : [])).filter(a => !indexes || indexes.has(a.index)),
    houses:array(report.houses).filter(h => !houseIds || houseIds.has(h.id)),
    bodyFacts:array(report.bodyFacts).filter(f => !bodyIds || bodyIds.has(factId(f))),
    eventIds:section ? array(section.eventIds) : array(view.eventReferences).map(ref => ref.eventId) };
}
export function resolveLocalAdvanced(envelope,model,report,key) {
  if (Array.isArray(report[key])) return report[key].map(fact => ({chartId:model.prefix || null,fact}));
  const refKey = {dispositorChains:'dispositorChainRefs',receptions:'natalReceptionRefs',aspectPatterns:'natalAspectPatternRefs'}[key];
  return array(report[refKey]).map(ref => ({chartId:ref.chartId,
    fact:array(envelope.data?.subjects?.[ref.chartId]?.context?.advanced?.[key]).find(fact => fact.id === ref.id)})).filter(item => item.fact);
}
