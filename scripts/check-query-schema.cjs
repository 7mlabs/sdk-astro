#!/usr/bin/env node
// Development-only strict structural and independently recomputed geometry/reference checks.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const Ajv2020=require(process.env.AJV_2020_MODULE||'ajv/dist/2020');
const ajv=new Ajv2020({strict:true,allErrors:true});
for(const name of ['query-request','query-response'])ajv.addSchema(JSON.parse(fs.readFileSync(path.join(__dirname,`../schemas/${name}.schema.json`),'utf8')));
const request=ajv.getSchema('urn:7mlabs:astrology:query-request:1.0'),response=ajv.getSchema('urn:7mlabs:astrology:query-response:1.0');
const bodies=['sun','moon','mercury','venus','mars','jupiter','saturn','uranus','neptune','pluto'];
const angles=['ascendant','midheaven','descendant','imumCoeli'];const houseIds=Array.from({length:12},(_,i)=>`H${i+1}`);const canonical=[...bodies,...angles,...houseIds];
const signs=['Aries','Taurus','Gemini','Cancer','Leo','Virgo','Libra','Scorpio','Sagittarius','Capricorn','Aquarius','Pisces'];
const birth={utc:{year:2000,month:1,day:1,hour:12,minute:0},location:{latitude:10.8231,longitude:106.6297}};
const routes=(group,action,options)=>({operation:'query',group,action,...options});
const cases=[
 ['normalize',true,routes('geometry','normalize',{longitude:-370})],['separation',true,routes('geometry','separation',{longitude1:350,longitude2:10})],
 ['midpoint',true,routes('geometry','midpoint',{longitude1:350,longitude2:10})],['antipodal branch',true,routes('geometry','midpoint',{longitude1:0,longitude2:180,antipodalPolicy:'lowerLongitude'})],
 ['between default static',true,routes('aspects','between',{positions:[{id:'a',longitude:10},{id:'b',longitude:87}],rule:{angle:77,maxOrb:0}})],
 ['locate',true,routes('houses','locate',{longitude:350,houseCusps:Array.from({length:12},(_,i)=>i*30)})],
];
const between=cases[4][2];for(const motionMode of ['none','relative','fixedSecond'])cases.push([`mode ${motionMode}`,true,{...between,motionMode,positions:[{id:'a',longitude:10,speed:1},{id:'b',longitude:87,speed:null}]}]);
for(const group of ['points','houses','aspects']){
 const input=routes(group,'inspect',{birth});cases.push([`${group} defaults`,true,input],[`${group} empty matching`,true,{...input,aspectRules:[]}],[`${group} extended modern`,true,{...input,aspectPreset:'extended',rulership:'modern'}]);
 for(const[field,value]of [['birth',null],['aspectPreset',null],['aspectRules',null],['rulership',null],['birth',{...birth,operation:'natal'}],['birth',{...birth,utc:{...birth.utc,year:2000.1}}],['birth',{...birth,utc:{...birth.utc,second:60}}]])cases.push([`${group} invalid ${field}`,false,{...input,[field]:value}]);
 cases.push([`${group} conflicting aspects`,false,{...input,aspectPreset:'major',aspectRules:[]}]);
 const selector=group==='houses'?'houseNumbers':'pointIds',valid=group==='houses'?[1,7]:['sun','H7'];
 cases.push([`${group} selectors`,true,{...input,[selector]:valid}],[`${group} empty selectors`,false,{...input,[selector]:[]}],[`${group} null selectors`,false,{...input,[selector]:null}],[`${group} duplicate selectors`,false,{...input,[selector]:[valid[0],valid[0]]}],[`${group} unknown selectors`,false,{...input,[selector]:group==='houses'?[13]:['chiron']}],[`${group} wrong selector group`,false,{...input,[group==='houses'?'pointIds':'houseNumbers']:valid}]);
}
cases.push(['two aspect points required',false,routes('aspects','inspect',{birth,pointIds:['sun']})],['single point allowed',true,routes('points','inspect',{birth,pointIds:['sun']})],['fractional house rejected',false,routes('houses','inspect',{birth,houseNumbers:[1.5]})],['decimal house integer',true,routes('houses','inspect',{birth,houseNumbers:[1.0,7.0]})],
 ['unknown group',false,routes('angles','normalize',{longitude:10})],['wrong group action',false,routes('points','locate',{longitude:10,houseCusps:[]})],['routing operation',false,{operation:'chart',group:'geometry',action:'normalize',longitude:10}],['null mode',false,{...between,motionMode:null}],['unknown mode',false,{...between,motionMode:'synastry'}],['null policy',false,routes('geometry','midpoint',{longitude1:0,longitude2:180,antipodalPolicy:null})],['unknown policy',false,routes('geometry','midpoint',{longitude1:0,longitude2:180,antipodalPolicy:'average'})],
 ['missing second longitude',false,routes('geometry','separation',{longitude1:10})],['null longitude',false,routes('geometry','normalize',{longitude:null})],['string longitude',false,routes('geometry','normalize',{longitude:'10'})],['missing rule',false,{...between,rule:undefined}],['one position',false,{...between,positions:[{id:'a',longitude:10}]}],['three positions',false,{...between,positions:[{id:'a',longitude:10},{id:'b',longitude:20},{id:'c',longitude:30}]}],['invalid angle',false,{...between,rule:{angle:181,maxOrb:1}}],['invalid orb',false,{...between,rule:{angle:77,maxOrb:16}}],['eleven cusps',false,routes('houses','locate',{longitude:10,houseCusps:Array(11).fill(0)})]);
for(const q of cases.filter(c=>c[1]).slice(0,8).map(c=>c[2]))cases.push([`${q.group}.${q.action} unknown field`,false,{...q,unexpected:true}]);
for(const[name,expected,input]of cases)assert.equal(request(input),expected,`${name}: ${JSON.stringify(request.errors)}`);
const files=process.argv.slice(2);assert(files.length,'Usage: node scripts/check-query-schema.cjs <query-response.json> [more]');
const norm=n=>{const r=((n%360)+360)%360;return r>=360?0:r;};const delta=(first,second)=>norm(norm(second)-norm(first)+180)-180;
const near=(a,b,t=1e-7)=>assert(Math.abs(a-b)<=t,`${a} != ${b}`);const nearDelta=(actual,expected)=>Math.abs(Math.abs(expected)-180)<1e-7?near(Math.abs(actual),Math.abs(expected)):near(actual,expected);
const unique=ids=>assert.equal(new Set(ids).size,ids.length,'Duplicate ID/reference');const sameSet=(a,b)=>assert.deepEqual([...a].sort(),[...b].sort());
const asMap=(items,key)=>{const map=new Map(items.map(x=>[x[key],x]));assert.equal(map.size,items.length,'Duplicate key');return map;};
function zodiac(p){const index=Math.floor(p.longitude/30);assert.equal(p.signIndex,index);assert.equal(p.sign,signs[index]);near(p.degreeInSign,p.longitude-index*30);}
function position(p){zodiac(p);assert.equal(p.isRetrograde,p.speed===null?null:p.speed<0);}
function checkInspect(data,calculation){
 const {selection,coverage}=data;const points=asMap(data.points,'id'),houses=asMap(data.houses,'id'),aspects=asMap(data.aspects,'index'),relations=asMap(data.relations,'id');
 const primary=new Set(data.primaryPointIds);unique(data.primaryPointIds);unique(data.primaryHouseIds);unique(selection.pointIds);unique(selection.houseNumbers);
 const policy=data.group==='aspects'?'bothSelectedEndpoints':'atLeastOneSelectedEndpoint';assert.equal(selection.aspectSelection,policy);assert.equal(calculation.selectionPolicy,policy);
 const expectedPrimary=new Set(selection.pointIds);
 const allCusps=houseIds.every(id=>points.has(id))?houseIds.map(id=>points.get(id).longitude):null;
 const assignHouse=longitude=>allCusps.findIndex((c,i)=>norm(longitude-c)<norm(allCusps[(i+1)%12]-c))+1;
 const rulers=calculation.rulership==='modern'?['mars','venus','mercury','moon','sun','mercury','venus','pluto','jupiter','saturn','uranus','neptune']:['mars','venus','mercury','moon','sun','mercury','venus','mars','jupiter','saturn','saturn','jupiter'];
 if(data.group==='houses')for(const house of data.houses){expectedPrimary.add(house.id);expectedPrimary.add(house.rulerBodyId);for(const p of house.occupants)expectedPrimary.add(p.id);}
 sameSet(primary,expectedPrimary);sameSet(data.primaryHouseIds,data.group==='houses'?selection.houseNumbers.map(n=>`H${n}`):selection.pointIds.filter(id=>houseIds.includes(id)));sameSet(houses.keys(),data.primaryHouseIds);
 for(const p of data.points){assert(canonical.includes(p.id));zodiac(p);if(p.kind==='body'){position(p);assert(bodies.includes(p.id));}else if(p.kind==='houseCusp'){assert.equal(p.id,`H${p.houseNumber}`);assert.equal(p.speed,null);}else{assert(angles.includes(p.id));assert.equal(p.speed,null);}}
 for(const id of primary)assert(points.has(id),`Missing primary ${id}`);
 if(allCusps){const widths=allCusps.map((c,i)=>norm(allCusps[(i+1)%12]-c));assert(widths.every(w=>w>1e-9));near(widths.reduce((a,b)=>a+b),360);for(const p of data.points.filter(p=>p.kind==='body'))assert.equal(p.house,assignHouse(p.longitude));}
 for(const h of data.houses){zodiac(h);assert.equal(h.id,`H${h.number}`);assert.equal(h.rulerBodyId,rulers[h.signIndex]);near(h.longitude,points.get(h.id).longitude);assert(points.has(h.rulerBodyId));assert.equal(h.rulerPlacement.id,h.rulerBodyId);unique(h.occupants.map(p=>p.id));
  for(const p of [...h.occupants,h.rulerPlacement]){position(p);const body=points.get(p.id);assert(body&&body.kind==='body');for(const[k,v]of Object.entries(p))assert.deepEqual(body[k],v);}
  for(const p of h.occupants)assert.equal(p.house,h.number);
  if(bodies.every(id=>points.has(id)))sameSet(h.occupants.map(p=>p.id),bodies.filter(id=>points.get(id).house===h.number));
 }
 const selected=(a,b)=>policy==='bothSelectedEndpoints'?primary.has(a)&&primary.has(b):primary.has(a)||primary.has(b);
 const requiredPoints=new Set(primary);for(const h of data.houses){requiredPoints.add(h.rulerBodyId);for(const p of h.occupants)requiredPoints.add(p.id);}
 const expectedPairs=[];for(let i=0;i<canonical.length;i++)for(let j=i+1;j<canonical.length;j++)if(selected(canonical[i],canonical[j]))expectedPairs.push(`${canonical[i]}:${canonical[j]}`);
 assert.deepEqual([...relations.keys()],expectedPairs);
 let index=0;for(const relation of data.relations){const a=points.get(relation.point1),b=points.get(relation.point2);assert(a&&b,'Missing relation endpoint');requiredPoints.add(a.id);requiredPoints.add(b.id);assert.equal(relation.id,`${a.id}:${b.id}`);assert(selected(a.id,b.id));const d=delta(a.longitude,b.longitude),separation=Math.abs(d);nearDelta(relation.signedDelta,d);near(relation.separation,separation);
  const matched=calculation.aspectRules.filter(r=>Math.abs(separation-r.angle)<=r.maxOrb);assert.equal(relation.aspectIndexes.length,matched.length);
  for(let i=0;i<matched.length;i++){const rule=matched[i],actual=aspects.get(index),orb=Math.abs(separation-rule.angle);assert(actual,'Missing local aspect');assert.equal(relation.aspectIndexes[i],index);assert.equal(actual.index,index);assert.equal(actual.point1,a.id);assert.equal(actual.point2,b.id);near(actual.angle,rule.angle);near(actual.separation,separation);near(actual.orb,orb);near(actual.maxOrb,rule.maxOrb);
   const applying=orb<=1e-12||a.speed===null||b.speed===null?null:(separation-rule.angle)*(b.speed-a.speed)*Math.sign(d)<0;assert.equal(actual.applying,applying);index++;}
 }
 assert.equal(index,data.aspects.length);sameSet(points.keys(),requiredPoints);
 assert.deepEqual(coverage,{primaryPoints:primary.size,supportingPoints:points.size-primary.size,selectedHouses:houses.size,pointPairs:relations.size,matchedAspects:aspects.size});
}
function check(response){const d=response.data,c=response.calculation;assert.equal(d.group,c.group);assert.equal(d.action,c.action);
 if(d.action==='inspect'){assert.equal(c.computationalScope,'fullNatalThenSelection');checkInspect(d,c);return;}
 assert.equal(c.computationalScope,'pureGeometry');
 if(d.group==='geometry'){
  if(d.action==='normalize'){near(d.longitude,norm(d.inputLongitude));zodiac(d);}
  else if(d.action==='separation'){const value=delta(d.longitude1,d.longitude2);nearDelta(d.signedDelta,value);near(d.separation,Math.abs(value));}
  else{const lower=Math.min(d.longitude1,d.longitude2),difference=Math.abs(d.longitude1-d.longitude2),antipodal=Math.abs(difference-180)<=c.toleranceDegrees;assert.equal(d.antipodal,antipodal);assert.equal(d.antipodalPolicy,c.antipodalPolicy);assert.equal(d.resolution,antipodal?'lowerLongitude':'unambiguous');if(antipodal)assert.equal(d.antipodalPolicy,'lowerLongitude');const middle=norm(lower+difference/2);const longitude=antipodal?Math.min(middle,norm(middle+180)):norm(middle+(difference>180?180:0));near(d.longitude,longitude);}
 }else if(d.group==='aspects'){
  const[a,b]=d.positions;assert.notEqual(a.id,b.id);for(const p of d.positions){position(p);assert.equal(p.house,null);}
  const value=delta(a.longitude,b.longitude),sep=Math.abs(value),offset=sep-d.rule.angle,orb=Math.abs(offset);nearDelta(d.relation.signedDelta,value);near(d.relation.separation,sep);assert.equal(d.relation.id,`${a.id}:${b.id}`);assert.equal(d.relation.point1,a.id);assert.equal(d.relation.point2,b.id);assert.equal(d.aspect.point1,a.id);assert.equal(d.aspect.point2,b.id);near(d.aspect.angle,d.rule.angle);near(d.aspect.maxOrb,d.rule.maxOrb);near(d.aspect.orb,orb);assert.equal(d.aspect.matched,orb<=d.rule.maxOrb);assert.equal(d.motionMode,c.motionMode);
  const velocity=d.motionMode==='fixedSecond'&&a.speed!==null?-a.speed:d.motionMode==='relative'&&a.speed!==null&&b.speed!==null?b.speed-a.speed:null;
  const rate=velocity===null||sep<=1e-12||Math.abs(sep-180)<=1e-12?null:Math.sign(value)*velocity;if(rate===null)assert.equal(d.distanceRate,null);else near(d.distanceRate,rate);
  assert.equal(d.aspect.applying,rate===null||orb<=1e-12?null:Math.sign(offset)*Math.sign(rate)<0);
 }else{
  const widths=d.houseCusps.map((c,i)=>norm(d.houseCusps[(i+1)%12]-c));assert(widths.every(w=>w>1e-9));near(widths.reduce((a,b)=>a+b),360);const number=d.houseCusps.findIndex((c,i)=>norm(d.longitude-c)<widths[i])+1;assert(number>0);assert.equal(d.house.number,number);assert.equal(d.house.id,`H${number}`);near(d.house.longitude,d.houseCusps[number-1]);near(d.house.widthDegrees,widths[number-1]);zodiac(d.house);
 }
}
let success=0,errors=0,malformed=0,semantic=0;
for(const file of files){const result=JSON.parse(fs.readFileSync(file,'utf8'));assert(response(result),`${file}: ${JSON.stringify(response.errors)}`);if(result.errors.length){assert.equal(result.data,null);errors++;continue;}check(result);success++;
 const badStructure=fn=>{const v=structuredClone(result);fn(v);assert(!response(v),'Malformed output accepted');malformed++;};
 const badJoin=fn=>{const v=structuredClone(result);fn(v);assert(response(v),JSON.stringify(response.errors));assert.throws(()=>check(v),'Broken semantic join accepted');semantic++;};
 badStructure(v=>v.data.unexpected=true);badStructure(v=>v.calculation.unexpected=true);badStructure(v=>v.data.group='unknown');badStructure(v=>delete v.calculation.provider);
 badJoin(v=>v.calculation={scope:'on-demand-query',computationalScope:'pureGeometry',group:'geometry',action:v.data.action==='normalize'?'separation':'normalize',provider:'supplied-positions',angleUnit:'degrees'});
 // Metadata routing mutations must remain structurally valid: replace the entire pure calculation when needed.
 if(result.data.action==='inspect'){
  badStructure(v=>v.data.selection.unexpected=true);badStructure(v=>v.data.points[0].unexpected=true);badStructure(v=>v.data.coverage.primaryPoints=-1);
  badJoin(v=>v.data.coverage.supportingPoints++);badJoin(v=>v.data.points[0].longitude=norm(v.data.points[0].longitude+1));
  if(result.data.relations.length){badJoin(v=>v.data.relations[0].signedDelta=v.data.relations[0].signedDelta===0?1:0);badJoin(v=>v.data.relations[0].point2='sun');}
  if(result.data.aspects.length){badJoin(v=>v.data.aspects[0].orb++);badJoin(v=>v.data.relations.find(r=>r.aspectIndexes.length).aspectIndexes=[]);}
 }else if(result.data.group==='geometry'){
  if(result.data.action==='normalize')badJoin(v=>v.data.longitude=norm(v.data.longitude+1));
  else if(result.data.action==='separation')badJoin(v=>v.data.separation=v.data.separation===0?1:0);
  else{badJoin(v=>v.data.longitude=norm(v.data.longitude+1));badJoin(v=>v.data.antipodal=!v.data.antipodal);}
 }else if(result.data.group==='aspects'){
  badStructure(v=>v.data.aspect.matched=null);badJoin(v=>v.data.aspect.matched=!v.data.aspect.matched);badJoin(v=>v.data.aspect.orb++);badJoin(v=>v.data.relation.point1='missing');badJoin(v=>v.data.aspect.applying=v.data.aspect.applying===true?false:true);badJoin(v=>v.data.distanceRate=v.data.distanceRate===null?1:v.data.distanceRate+1);
 }else{badJoin(v=>v.data.house.widthDegrees++);badJoin(v=>v.data.house.number=v.data.house.number===12?1:v.data.house.number+1);}
}
console.log(`Query contracts: ${cases.length} request cases; ${files.length} responses (${success} success, ${errors} error); ${malformed} malformed structures and ${semantic} broken semantic joins rejected`);
