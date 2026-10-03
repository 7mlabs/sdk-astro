#!/usr/bin/env node
// Development-only strict JSON Schema, geometry, calendar and reference validation.
const assert = require('node:assert/strict');const fs = require('node:fs');const path = require('node:path');
const Ajv2020 = require(process.env.AJV_2020_MODULE || 'ajv/dist/2020');
const files = process.argv.slice(2);assert(files.length, 'Usage: node scripts/check-forecast-schema.cjs <events-or-forecast-response.json> [more]');
const ajv = new Ajv2020({ strict: true, allErrors: true });
for (const name of ['natal-domains-request', 'natal-domains-response', 'events-request', 'events-response', 'forecast-request', 'forecast-response'])
  ajv.addSchema(JSON.parse(fs.readFileSync(path.join(__dirname, `../schemas/${name}.schema.json`), 'utf8')));
const requestEvents = ajv.getSchema('urn:7mlabs:astrology:events-request:1.0');
const requestForecast = ajv.getSchema('urn:7mlabs:astrology:forecast-request:1.0');
const validateEvents = ajv.getSchema('urn:7mlabs:astrology:events-response:1.0');
const validateForecast = ajv.getSchema('urn:7mlabs:astrology:forecast-response:1.0');
const bodies = ['sun', 'moon', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'uranus', 'neptune', 'pluto'];
const builtinIds = ['career', 'love', 'relationships', 'family', 'finance', 'identity', 'learning', 'creativity', 'innerLife', 'dailyLife'];
const eventTypes = ['ingress', 'station', 'lunarPhase', 'planetaryAspect', 'natalTransit', 'solarEclipse', 'lunarEclipse'];
const slow = ['mars', 'jupiter', 'saturn', 'uranus', 'neptune', 'pluto'];
const birth = { utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 }, location: { latitude: 10.8231, longitude: 106.6297 } };
const period = { kind: 'day', year: 2026, month: 3, day: 3, utcOffsetMinutes: 420 };
const sharedCases = [
 ['day defaults',true,{}],['month',true,{period:{kind:'month',year:2026,month:3}}],['year',true,{period:{kind:'year',year:2026}}],
 ['decimal integers',true,{period:{...period,year:2026.0,utcOffsetMinutes:420.0}}],['maximum east offset',true,{period:{...period,utcOffsetMinutes:840}}],
 ['maximum west offset',true,{period:{...period,utcOffsetMinutes:-840}}],['one body',true,{bodies:['saturn']}],['all bodies',true,{bodies}],
 ['all six global families',true,{eventTypes:eventTypes.filter(t=>t!=='natalTransit')}],['eclipse only',true,{eventTypes:['solarEclipse','lunarEclipse']}],
 ['disabled global events',true,{eventTypes:[]}],['disabled aspects',true,{aspectRules:[]}],['extended preset',true,{aspectPreset:'extended'}],
 ['null period',false,{period:null}],['missing period',false,{period:undefined}],['string year',false,{period:{...period,year:'2026'}}],
 ['fractional year',false,{period:{...period,year:2026.5}}],['year too old',false,{period:{...period,year:1799}}],
 ['year too new',false,{period:{...period,year:2400}}],['year with month',false,{period:{kind:'year',year:2026,month:3}}],
 ['year with null month',false,{period:{kind:'year',year:2026,month:null}}],['month with day',false,{period:{kind:'month',year:2026,month:3,day:1}}],
 ['month with null day',false,{period:{kind:'month',year:2026,month:3,day:null}}],['day missing month',false,{period:{kind:'day',year:2026,day:3}}],
 ['day missing day',false,{period:{kind:'day',year:2026,month:3}}],['bad month',false,{period:{...period,month:13}}],['bad day',false,{period:{...period,day:0}}],
 ['fractional offset',false,{period:{...period,utcOffsetMinutes:420.5}}],['offset too large',false,{period:{...period,utcOffsetMinutes:841}}],
 ['null offset',false,{period:{...period,utcOffsetMinutes:null}}],['IANA timezone field',false,{period:{...period,timezone:'Asia/Ho_Chi_Minh'}}],
 ['unknown period selector',false,{period:{...period,hour:12}}],['empty bodies',false,{bodies:[]}],['null bodies',false,{bodies:null}],
 ['duplicate body',false,{bodies:['sun','sun']}],['unsupported body',false,{bodies:['chiron']}],['null event types',false,{eventTypes:null}],
 ['duplicate event type',false,{eventTypes:['station','station']}],['personal family in globals',false,{eventTypes:['natalTransit']}],
 ['unsupported event type',false,{eventTypes:['solarReturn']}],['null aspect preset',false,{aspectPreset:null}],
 ['conflicting aspect options',false,{aspectPreset:'major',aspectRules:[]}],['negative angle',false,{aspectRules:[{angle:-1,maxOrb:1}]}],
 ['large orb',false,{aspectRules:[{angle:60,maxOrb:16}]}],['unknown rule field',false,{aspectRules:[{angle:60,maxOrb:1,label:'sextile'}]}],
 ['null aspect rules',false,{aspectRules:null}],['unknown top field',false,{language:'vi'}],
];
let requestCases=0;
for(const [name,expected,options]of sharedCases){
 assert.equal(requestEvents({operation:'events',period,...options}),expected,`events ${name}: ${JSON.stringify(requestEvents.errors)}`);
 assert.equal(requestForecast({operation:'forecast',birth,period,...options}),expected,`forecast ${name}: ${JSON.stringify(requestForecast.errors)}`);requestCases+=2;
}
const custom={id:'myDay',houses:[1],sections:[{id:'study',houses:[3,9],bodies:['mercury']}]};
for(const [name,expected,options]of[
 ['all ten domains',true,{domains:builtinIds}],['custom only',true,{domains:[],customProfiles:[custom]}],
 ['no exact natal roots',true,{includeNatalTransits:false}],['modern rulership',true,{rulership:'modern'}],
 ['fractional birth second',true,{birth:{...birth,utc:{...birth.utc,second:.5}}}],['missing birth',false,{birth:undefined}],['null birth',false,{birth:null}],
 ['birth nested operation',false,{birth:{...birth,operation:'natal'}}],['birth unknown field',false,{birth:{...birth,name:'N'}}],
 ['null transits flag',false,{includeNatalTransits:null}],['string transits flag',false,{includeNatalTransits:'yes'}],['null rulership',false,{rulership:null}],
 ['empty domains',false,{domains:[]}],['duplicate domains',false,{domains:['career','career']}],['pair domain',false,{domains:['attraction']}],
 ['empty custom array',false,{customProfiles:[]}],['reserved custom ID',false,{customProfiles:[{...custom,id:'career'}]}],
 ['prototype ID',false,{customProfiles:[{...custom,id:'constructor'}]}],['empty custom selectors',false,{customProfiles:[{id:'empty'}]}],
 ['fractional house selector',false,{customProfiles:[{id:'fraction',houses:[1.5]}]}],['empty section',false,{customProfiles:[{...custom,sections:[{id:'empty'}]}]}],
 ['too many custom profiles',false,{customProfiles:Array.from({length:9},(_,i)=>({id:`profile${i}`,houses:[1]}))}],
]){assert.equal(requestForecast({operation:'forecast',birth,period,...options}),expected,`${name}: ${JSON.stringify(requestForecast.errors)}`);requestCases++;}
for(const extra of[{birth},{domains:['career']},{includeNatalTransits:true},{rulership:'modern'},{customProfiles:[custom]}]){
 assert(!requestEvents({operation:'events',period,...extra}));requestCases++;
}
const errorEnvelope={schemaVersion:'1.0',engineVersion:'0.8.0-alpha.1',data:null,warnings:[],errors:[{code:'INVALID_INPUT',message:'Invalid UTC window'}]};
assert(validateEvents(errorEnvelope));assert(validateForecast(errorEnvelope));
const norm=n=>((n%360)+360)%360;const delta=(a,b)=>norm(a-b+180)-180;
const near=(a,b,tolerance=1e-7)=>assert(Math.abs(a-b)<=tolerance,`${a} != ${b}`);
const nearLongitude=(a,b,tolerance=1e-7)=>near(delta(a,b),0,tolerance);
const unique=xs=>assert.equal(new Set(xs).size,xs.length,'Duplicate reference');
const sameSet=(a,b)=>assert.deepEqual([...a].sort(),[...b].sort());
function mapOf(xs,key,label){const m=new Map(xs.map(x=>[x[key],x]));assert.equal(m.size,xs.length,`Duplicate ${label}`);return m;}
const contains=(map,ids,label)=>{unique(ids);for(const id of ids)assert(map.has(id),`Missing ${label}: ${id}`);};
// POSIX milliseconds have no leap-second slot; preserve its civil bucket for comparisons.
const clockMs=c=>{const second=Math.min(c.second,59.999);return Date.UTC(c.year,c.month-1,c.day,c.hour,c.minute,Math.floor(second))+Math.round((second%1)*1000);};
const civilDate=c=>`${c.year.toString().padStart(4,'0')}-${c.month.toString().padStart(2,'0')}-${c.day.toString().padStart(2,'0')}`;
const signNames=['Aries','Taurus','Gemini','Cancer','Leo','Virgo','Libra','Scorpio','Sagittarius','Capricorn','Aquarius','Pisces'];
function checkClock(utc,local,offset){assert.equal(local.utcOffsetMinutes,offset);near(clockMs(local)-clockMs(utc),offset*60000,1);}
function checkPosition(p){const i=Math.floor(p.longitude/30);assert.equal(p.sign,signNames[i]);near(p.degreeInSign,p.longitude-i*30);assert.equal(p.isRetrograde,p.speed<0);}
function checkShared(response){
 const {data,calculation}=response;const {period,snapshot,events}=data;const p=period;
 assert.equal(p.startUtc.second,0);assert.equal(p.endUtc.second,0);assert(p.endJulianDayUt1>p.startJulianDayUt1);near(p.durationDays,p.endJulianDayUt1-p.startJulianDayUt1);
 const startMonth=p.kind==='year'?1:p.month;const startDay=p.kind==='day'?p.day:1;
 const startCivil=Date.UTC(p.year,startMonth-1,startDay);const endCivil=p.kind==='day'?startCivil+86400000:p.kind==='month'?Date.UTC(p.year,p.month,1):Date.UTC(p.year+1,0,1);
 assert.equal(clockMs(p.startUtc),startCivil-p.utcOffsetMinutes*60000);assert.equal(clockMs(p.endUtc),endCivil-p.utcOffsetMinutes*60000);
 near(snapshot.julianDayUt1,(p.startJulianDayUt1+p.endJulianDayUt1)/2);checkClock(snapshot.utc,snapshot.local,p.utcOffsetMinutes);
 const positions=mapOf(snapshot.positions,'id','snapshot bodies');sameSet(positions.keys(),calculation.bodyIds);for(const v of positions.values())checkPosition(v);
 const eventMap=mapOf(events,'id','event IDs');let last=-Infinity;
 const counts=Object.fromEntries(eventTypes.map(t=>[t,0]));
 for(const event of events){
  assert(event.julianDayUt1>=p.startJulianDayUt1&&event.julianDayUt1<p.endJulianDayUt1);assert(event.julianDayUt1>=last);last=event.julianDayUt1;
  checkClock(event.utc,event.local,p.utcOffsetMinutes);assert(clockMs(event.local)>=startCivil&&clockMs(event.local)<endCivil);
  // UT1 and civil UTC can differ by leap/Delta-T corrections; gross timestamp mismatches fail.
  near(clockMs(event.utc),clockMs(p.startUtc)+(event.julianDayUt1-p.startJulianDayUt1)*86400000,3000);
  const ep=mapOf(event.positions,'id','event positions');sameSet(ep.keys(),event.bodyIds);for(const v of ep.values())checkPosition(v);counts[event.type]++;
  if(event.type==='natalTransit')assert.equal(calculation.includeNatalTransits,true);else assert(calculation.eventTypes.includes(event.type));
  if(!['lunarPhase','solarEclipse','lunarEclipse'].includes(event.type))for(const body of event.bodyIds)assert(calculation.bodyIds.includes(body));
  const eclipse=event.type==='solarEclipse'||event.type==='lunarEclipse';
  if(eclipse){assert.equal(event.precision.method,'swissEclipseSearch');unique(event.details.contacts.map(c=>c.name));
   for(const contact of event.details.contacts){checkClock(contact.utc,contact.local,p.utcOffsetMinutes);assert(contact.julianDayUt1>0);}
   const validNames=event.type==='solarEclipse'?['eclipseBegin','eclipseEnd','centralPhaseBegin','centralPhaseEnd']:['partialBegin','partialEnd','totalityBegin','totalityEnd','penumbralBegin','penumbralEnd'];
   for(const contact of event.details.contacts)assert(validNames.includes(contact.name));
   for(const prefix of ['eclipse','centralPhase','partial','totality','penumbral']){const begin=event.details.contacts.find(c=>c.name===`${prefix}Begin`);const end=event.details.contacts.find(c=>c.name===`${prefix}End`);if(begin&&end){assert(begin.julianDayUt1<=event.julianDayUt1);assert(end.julianDayUt1>=event.julianDayUt1);}}
   assert(event.details.eclipseType!=='penumbral'||event.type==='lunarEclipse');assert(!['annular','hybrid'].includes(event.details.eclipseType)||event.type==='solarEclipse');
  }else{
   assert.equal(event.precision.method,'bracketedRoot');assert(event.precision.bracketSeconds<=calculation.search.timeToleranceSeconds);
   assert(event.precision.residual<=(event.type==='station'?calculation.search.stationToleranceDegreesPerDay:calculation.search.angularToleranceDegrees));
   const a=event.positions[0];const detail=event.details;
   if(event.type==='station'){near(Math.abs(a.speed),event.precision.residual,1e-9);}
   if(event.type==='ingress'){nearLongitude(a.longitude,detail.boundaryLongitude,1.01e-6);near(norm(detail.boundaryLongitude)%30,0,1e-7);const upper=Math.round(detail.boundaryLongitude/30)%12;const lower=(upper+11)%12;assert.equal(detail.fromSign,signNames[detail.direction==='direct'?lower:upper]);assert.equal(detail.toSign,signNames[detail.direction==='direct'?upper:lower]);assert.equal(detail.direction,a.speed>0?'direct':'retrograde');}
   if(event.type==='lunarPhase'){const moon=ep.get('moon'),sun=ep.get('sun');nearLongitude(moon.longitude-sun.longitude,detail.angle,1.01e-6);assert.equal(detail.phase,['newMoon','firstQuarter','fullMoon','lastQuarter'][detail.angle/90]);}
   if(event.type==='planetaryAspect'){nearLongitude(event.positions[0].longitude-event.positions[1].longitude,detail.branchLongitude,1.01e-6);near(Math.abs(delta(event.positions[0].longitude,event.positions[1].longitude)),detail.angle,1.01e-6);assert(calculation.aspectRules.some(r=>r.angle===detail.angle));}
   if(event.type==='natalTransit'){nearLongitude(a.longitude-detail.targetLongitude,detail.branchLongitude,1.01e-6);near(Math.abs(delta(a.longitude,detail.targetLongitude)),detail.angle,1.01e-6);assert(calculation.aspectRules.some(r=>r.angle===detail.angle));}
  }
 }
 if(data.chartKind==='astronomicalEvents'){assert.equal(data.coverage.eventCount,events.length);assert.deepEqual(data.coverage.byType,counts);}
 return{events:eventMap,counts};
}
function houseFor(l,cusps){for(let i=0;i<12;i++)if(norm(l-cusps[i])<norm(cusps[(i+1)%12]-cusps[i]))return i+1;throw Error('No house');}
function checkForecast(response,shared){
 const {data,calculation}=response;const {subject,snapshot,period}=data;assert.equal(subject.id,'N');assert.equal(subject.context.rulership,calculation.rulership);
 const npoints=new Map(subject.context.points.map(p=>[`N:${p.id}`,p]));const nhouses=new Map(subject.context.houses.map(h=>[`N:${h.id}`,h]));
 const snapshotPoints=new Map(snapshot.positions.map(p=>[p.pointId,p]));unique([...snapshotPoints.keys()]);
 for(const p of snapshot.positions)assert.equal(p.pointId,`T:${p.id}`);
 const sa=mapOf(snapshot.aspects,'index','snapshot aspect');const sr=mapOf(snapshot.relations,'id','snapshot relation');assert.equal(sr.size,snapshot.positions.length*26);
 function expectedContacts(positions){const contacts=[];const relations=[];for(const p of positions)for(const[nid,n]of npoints){
  const tid=`T:${p.id}`,id=`${tid}|${nid}`,d=delta(p.longitude,n.longitude),separation=Math.abs(d),indexes=[];
  for(const rule of calculation.aspectRules){const orb=Math.abs(separation-rule.angle);if(orb<=rule.maxOrb){const index=contacts.length;indexes.push(index);const applying=orb<=1e-6||Math.abs(p.speed)<=1e-12?null:(separation-rule.angle)*p.speed*Math.sign(d)<0;
   contacts.push({index,relationId:id,transitPointId:tid,natalPointId:nid,angle:rule.angle,separation,orb,maxOrb:rule.maxOrb,applying});}}
  relations.push({id,transitPointId:tid,natalPointId:nid,signedDelta:d,separation,aspectIndexes:indexes});
 }return{contacts,relations};}
 function compareGeometry(actual,expected){assert.equal(actual.length,expected.length);for(let i=0;i<expected.length;i++)for(const[k,v]of Object.entries(expected[i])){
  if(typeof v==='number'){if(k==='signedDelta'&&Math.abs(Math.abs(v)-180)<1e-7)near(Math.abs(actual[i][k]),Math.abs(v));else near(actual[i][k],v);}else assert.deepEqual(actual[i][k],v);
 }}
 const sc=expectedContacts(snapshot.positions);compareGeometry(snapshot.aspects,sc.contacts);compareGeometry(snapshot.relations,sc.relations);
 function checkOverlays(overlays,positions){assert.equal(overlays.length,positions.length);const byId=new Map(positions.map(p=>[`T:${p.id}`,p]));unique(overlays.map(o=>o.transitPointId));for(const o of overlays){assert(byId.has(o.transitPointId));const number=houseFor(byId.get(o.transitPointId).longitude,subject.natal.houseCusps);assert.equal(o.natalHouseNumber,number);assert.equal(o.natalHouseId,`N:H${number}`);const house=nhouses.get(o.natalHouseId);assert.equal(o.natalRulerPointId,`N:${house.rulerBodyId}`);assert.deepEqual(o.natalOccupantPointIds,house.occupants.map(p=>`N:${p.id}`));}}
 checkOverlays(snapshot.houseOverlays,snapshot.positions);
 for(const event of data.events){const impact=event.personalImpact;checkOverlays(impact.houseOverlays,event.positions);const expected=expectedContacts(event.positions);compareGeometry(impact.contacts,expected.contacts);
  contains(npoints,impact.affectedNatalPointIds,'affected natal point');contains(nhouses,impact.affectedNatalHouseIds,'affected natal house');
  if(event.type==='natalTransit'){assert.equal(impact.primaryNatalPointId,event.details.targetPointId);assert.equal(impact.primaryAngle,event.details.angle);assert(npoints.has(event.details.targetPointId));nearLongitude(event.details.targetLongitude,npoints.get(event.details.targetPointId).longitude);}
  else{assert.equal(impact.primaryNatalPointId,null);assert.equal(impact.primaryAngle,null);}
  sameSet(impact.affectedNatalPointIds,new Set([...impact.contacts.map(c=>c.natalPointId),...(impact.primaryNatalPointId?[impact.primaryNatalPointId]:[])]));
  const affected=new Set(impact.houseOverlays.map(o=>o.natalHouseId)),rulers=[];
  for(const id of impact.affectedNatalPointIds){const p=npoints.get(id);affected.add(p.kind==='houseCusp'?id:`N:H${houseFor(p.longitude,subject.natal.houseCusps)}`);
   if(p.kind==='body'){const ruled=[...nhouses].filter(([,h])=>h.rulerBodyId===p.id).map(([id])=>id);for(const id of ruled)affected.add(id);if(ruled.length)rulers.push({natalPointId:id,natalHouseIds:ruled});}}
  sameSet(impact.affectedNatalHouseIds,affected);assert.deepEqual(impact.rulerLinks,rulers);
  const expectedHighlight=['solarEclipse','lunarEclipse'].includes(event.type)?['eclipse']:event.type==='station'?['station']:event.type==='lunarPhase'&&['newMoon','fullMoon'].includes(event.details.phase)?['newOrFullMoon']:event.type==='ingress'&&event.bodyIds.some(id=>slow.includes(id))?['slowBodyIngress']:event.type==='planetaryAspect'&&!event.bodyIds.includes('moon')&&event.bodyIds.some(id=>slow.includes(id))?['slowBodyPlanetaryAspect']:event.type==='natalTransit'&&!event.bodyIds.includes('moon')?['nonLunarNatalTransit']:[];
  assert.deepEqual(event.highlightReasons,expectedHighlight);
 }
 const catalog=mapOf(data.profileCatalog,'id','catalog');sameSet(catalog.keys(),builtinIds);assert.equal(calculation.customProfileCount,Object.keys(data.customDomains).length);
 const select=definition=>{const hs=new Set(definition.houses.map(n=>`N:H${n}`));const ps=new Set([...hs,...definition.angles.map(id=>`N:${id}`)]);
  for(const[id,p]of npoints)if(p.kind==='body'&&(definition.bodies.includes(p.id)||definition.houses.includes(p.house)||[...hs].some(h=>nhouses.get(h).rulerBodyId===p.id)))ps.add(id);return{points:ps,houses:hs};};
 const reasons=(event,selection)=>{const contacts=event.personalImpact.contacts.filter(c=>selection.points.has(c.natalPointId)),out=[];
  if(contacts.length)out.push({code:'natalContact',natalPointIds:[...new Set(contacts.map(c=>c.natalPointId))].sort(),natalHouseIds:[],aspectIndexes:contacts.map(c=>c.index)});
  const id=event.personalImpact.primaryNatalPointId;if(id&&selection.points.has(id))out.push({code:'exactNatalTarget',natalPointIds:[id],natalHouseIds:[],aspectIndexes:[]});
  const hs=[...new Set(event.personalImpact.houseOverlays.filter(o=>selection.houses.has(o.natalHouseId)).map(o=>o.natalHouseId))].sort();if(hs.length)out.push({code:'transitThroughFocusHouse',natalPointIds:[],natalHouseIds:hs,aspectIndexes:[]});return out;};
 const snapRefs=s=>({aspects:snapshot.aspects.filter(a=>s.points.has(a.natalPointId)).map(a=>a.index),overlays:snapshot.houseOverlays.filter(o=>s.houses.has(o.natalHouseId)).map(o=>o.transitPointId)});
 const views={...data.domains,...data.customDomains};
 for(const[origin,map]of[['builtin',data.domains],['custom',data.customDomains]])for(const[id,view]of Object.entries(map)){
  assert.equal(view.id,id);assert.equal(view.origin,origin);assert.equal(view.definition.id,id);assert.equal(view.profileVersion,view.definition.version);if(origin==='builtin')assert.deepEqual(view.definition,catalog.get(id));
  const primary=select(view.definition),union={points:new Set(primary.points),houses:new Set(primary.houses)},sections=mapOf(view.sections,'id','forecast sections');sameSet(sections.keys(),view.definition.sections.map(s=>s.id));
  for(const def of view.definition.sections){const selected=select(def),section=sections.get(def.id);for(const p of selected.points)union.points.add(p);for(const h of selected.houses)union.houses.add(h);
   sameSet(section.natalPointIds,selected.points);sameSet(section.natalHouseIds,selected.houses);const refs=snapRefs(selected);assert.deepEqual(section.snapshotAspectIndexes,refs.aspects);assert.deepEqual(section.snapshotOverlayPointIds,refs.overlays);
   const events=data.events.filter(e=>reasons(e,selected).length);assert.deepEqual(section.eventIds,events.map(e=>e.id));assert.deepEqual(section.highlightEventIds,events.filter(e=>e.highlightReasons.length).map(e=>e.id));}
  sameSet(view.primaryNatalPointIds,primary.points);sameSet(view.primaryNatalHouseIds,primary.houses);sameSet(view.natalPointIds,union.points);sameSet(view.natalHouseIds,union.houses);
  const references=mapOf(view.eventReferences,'eventId','view event references');const expected=data.events.filter(e=>reasons(e,union).length);assert.deepEqual([...references.keys()],expected.map(e=>e.id));
  for(const reference of references.values()){const event=shared.events.get(reference.eventId);assert(event);assert.equal(reference.primaryContact,!!reasons(event,primary).length);assert.deepEqual(reference.reasons,reasons(event,union));assert.deepEqual(reference.sectionIds,view.definition.sections.filter(s=>reasons(event,select(s)).length).map(s=>s.id));}
  const refs=snapRefs(union);assert.deepEqual(view.snapshotAspectIndexes,refs.aspects);assert.deepEqual(view.snapshotOverlayPointIds,refs.overlays);
  const highlights=expected.filter(e=>e.highlightReasons.length).map(e=>e.id);assert.deepEqual(view.highlightEventIds,highlights);
  assert.deepEqual(view.messageContext,{kind:{day:'dailyMessage',month:'monthlyOverview',year:'yearlyOverview'}[period.kind],snapshotAspectIndexes:refs.aspects,eventIds:expected.map(e=>e.id),highlightEventIds:highlights,natalHouseIds:[...union.houses].sort(),narrative:null});
 }
 const overview=data.overview;assert.equal(overview.eventCount,data.events.length);assert.deepEqual(overview.byType,shared.counts);
 assert.deepEqual(overview.domainEventCounts,Object.fromEntries(Object.entries(views).map(([id,v])=>[id,v.eventReferences.length])));
 const highlights=data.events.filter(e=>e.highlightReasons.length).map(e=>e.id);assert.deepEqual(overview.highlightEventIds,highlights);
 for(const[id]of nhouses)assert.equal(overview.natalHouseEventCounts[id],data.events.filter(e=>e.personalImpact.affectedNatalHouseIds.includes(id)).length);
 const days=new Map();for(const event of data.events){const date=civilDate(event.local);if(!days.has(date))days.set(date,[]);days.get(date).push(event);}
 assert.deepEqual(overview.days,[...days].sort(([a],[b])=>a.localeCompare(b)).map(([date,events])=>({date,eventIds:events.map(e=>e.id),highlightEventIds:events.filter(e=>e.highlightReasons.length).map(e=>e.id)})));
 const months=period.kind==='year'?Array.from({length:12},(_,i)=>i+1):[period.month];assert.deepEqual(overview.months,months.map(month=>{const events=data.events.filter(e=>e.local.month===month);return{year:period.year,month,eventIds:events.map(e=>e.id),highlightEventIds:events.filter(e=>e.highlightReasons.length).map(e=>e.id)};}));
}
let successCount=0,errorCount=0,structures=0,joins=0;
function checkAll(response){const shared=checkShared(response);if(response.data.chartKind==='individualForecast')checkForecast(response,shared);}
for(const file of files){
 const response=JSON.parse(fs.readFileSync(file,'utf8'));const forecast=response.data?.chartKind==='individualForecast';const validate=forecast?validateForecast:validateEvents;
 assert(validate(response),`${file}: ${JSON.stringify(validate.errors,null,2)}`);if(response.errors.length){errorCount++;continue;}checkAll(response);successCount++;
 const mutations=[
  ['wrong kind',v=>{v.data.chartKind='individualNatal';}],['IANA output',v=>{v.data.period.timezone='Asia/Ho_Chi_Minh';}],
  ['missing normalized period offset',v=>{delete v.data.period.utcOffsetMinutes;}],['wrong search interval',v=>{v.calculation.search.interval='inclusiveBoth';}],
  ['truncated events',v=>{v.calculation.search.truncated=true;}],['invented precision claim',v=>{v.calculation.search.absoluteAccuracySeconds=.25;}],
  ['snapshot synthetic house',v=>{v.data.snapshot.positions[0].house=1;}],['unknown snapshot field',v=>{v.data.snapshot.score=1;}],
  ['missing positions',v=>{v.data.snapshot.positions=[];}],['nonphysical speed',v=>{v.data.snapshot.positions[0].speed=null;}],
  ['missing event counter',v=>{delete (forecast?v.data.overview:v.data.coverage).byType.solarEclipse;}],
 ];
 if(response.data.events.length){
  mutations.push(['unknown event field',v=>{v.data.events[0].prediction='Good day';}]);
  mutations.push(['missing precision method',v=>{delete v.data.events[0].precision.method;}]);
  mutations.push(['unknown participant',v=>{v.data.events[0].bodyIds=['chiron'];}]);
  const eclipseIndex=response.data.events.findIndex(e=>e.type==='solarEclipse'||e.type==='lunarEclipse');
  if(eclipseIndex>=0){mutations.push(['local eclipse visibility claim',v=>{v.data.events[eclipseIndex].details.visibilityScope='birthLocation';}]);mutations.push(['eclipse angular root precision claim',v=>{v.data.events[eclipseIndex].precision={method:'bracketedRoot',bracketSeconds:.1,residual:0,residualUnit:'degrees'};}]);}
  const rootIndex=response.data.events.findIndex(e=>e.precision.method==='bracketedRoot');
  if(rootIndex>=0)mutations.push(['root precision swapped for eclipse',v=>{v.data.events[rootIndex].precision={method:'swissEclipseSearch',bracketSeconds:null,residual:null,residualUnit:null};}]);
 }
 const viewGroup=forecast?(Object.keys(response.data.domains).length?'domains':'customDomains'):null;
 const viewId=forecast?Object.keys(response.data[viewGroup])[0]:null;const view=v=>v.data[viewGroup][viewId];
 if(forecast){mutations.push(
  ['missing natal source',v=>{delete v.data.subject;}],['wrong subject count',v=>{v.data.subjectCount=2;}],
  ['moving natal targets',v=>{v.calculation.transitTargetMotion='birthVelocities';}],['missing snapshot relations',v=>{delete v.data.snapshot.relations;}],
  ['invalid transit namespace',v=>{v.data.snapshot.positions[0].pointId='sun';}],['missing overlay',v=>{delete v.data.snapshot.houseOverlays;}],
  ['missing catalog entry',v=>{v.data.profileCatalog.pop();}],['text narrative',v=>{view(v).messageContext.narrative='Happy day';}],
  ['missing message context',v=>{delete view(v).messageContext;}],['wrong forecast highlight policy',v=>{v.calculation.highlightPolicy.version='2.0';}],
  ['missing forecast section refs',v=>{if(view(v).sections.length)delete view(v).sections[0].eventIds;else view(v).sections=[{id:'bad'}];}],
 );if(response.data.events.length)mutations.push(['missing per-event personal impact',v=>{delete v.data.events[0].personalImpact;}]);}
 for(const[name,mutate]of mutations){const changed=structuredClone(response);mutate(changed);assert(!validate(changed),`${file}: accepted malformed ${name}`);structures++;}
 const semantic=[
  ['wrong snapshot retrograde',v=>{v.data.snapshot.positions[0].isRetrograde=!v.data.snapshot.positions[0].isRetrograde;}],
  ['wrong snapshot sign',v=>{const p=v.data.snapshot.positions[0];p.sign=p.sign==='Aries'?'Taurus':'Aries';}],
  ['wrong snapshot midpoint JD',v=>{v.data.snapshot.julianDayUt1+=.01;}],['wrong civil period UTC',v=>{v.data.period.startUtc.minute=1;}],
  ['wrong selected snapshot bodies',v=>{v.calculation.bodyIds=v.calculation.bodyIds.includes('pluto')?['sun']:['pluto'];}],
  ['wrong event count',v=>{(forecast?v.data.overview:v.data.coverage).eventCount+=1;}],
 ];
 if(response.data.events.length)semantic.push(['wrong event JD timestamp',v=>{v.data.events[0].julianDayUt1+=.01;}],['wrong local offset',v=>{v.data.events[0].local.utcOffsetMinutes+=1;}]);
 if(forecast){semantic.push(
  ['wrong natal house overlay',v=>{const o=v.data.snapshot.houseOverlays[0];o.natalHouseId=o.natalHouseId==='N:H1'?'N:H2':'N:H1';}],
  ['wrong snapshot event index',v=>{view(v).snapshotAspectIndexes=[999999];}],
  ['wrong message kind',v=>{view(v).messageContext.kind=view(v).messageContext.kind==='dailyMessage'?'yearlyOverview':'dailyMessage';}],
  ['wrong focus natal point',v=>{view(v).primaryNatalPointIds=['N:pluto'];}],
  ['wrong domain event count',v=>{v.data.overview.domainEventCounts[viewId]+=1;}],
  ['wrong natal house event count',v=>{v.data.overview.natalHouseEventCounts['N:H1']+=1;}],
 );if(response.data.events.length)semantic.push(
  ['dangling domain event',v=>{view(v).eventReferences=[{eventId:'missing',primaryContact:true,sectionIds:[],reasons:[{code:'exactNatalTarget',natalPointIds:['N:sun'],natalHouseIds:[],aspectIndexes:[]}]}];}],
  ['incorrect event highlight',v=>{v.data.events[0].highlightReasons=v.data.events[0].highlightReasons.length?[]:['eclipse'];}],
  ['wrong per-event overlay',v=>{const o=v.data.events[0].personalImpact.houseOverlays[0];o.natalRulerPointId=o.natalRulerPointId==='N:sun'?'N:moon':'N:sun';}],
 );if(view(response).sections.length)semantic.push(['dangling section event',v=>{view(v).sections[0].eventIds=['missing'];}]);}
 for(const[name,mutate]of semantic){const changed=structuredClone(response);mutate(changed);assert(validate(changed),`${file}: ${name} should be structurally valid: ${JSON.stringify(validate.errors)}`);assert.throws(()=>checkAll(changed),undefined,`${file}: semantic validator accepted ${name}`);joins++;}
}
console.log(`Events/forecast contracts: ${requestCases} request cases; ${files.length} responses (${successCount} success, ${errorCount} error); ${structures} malformed structures and ${joins} broken semantic joins rejected; error envelopes validated`);
