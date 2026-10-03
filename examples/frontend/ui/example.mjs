import { createAstroUI } from './index.mjs';
const birth = {utc:{year:2000,month:1,day:1,hour:12,minute:0,second:0},location:{latitude:10.8231,longitude:106.6297},houseSystem:'placidus'};
const other = {utc:{year:2001,month:6,day:15,hour:6,minute:30,second:0},location:{latitude:21.0285,longitude:105.8542},houseSystem:'wholeSign'};
const samples = {
  natal:{operation:'natal',...birth},natalDomains:{operation:'natalDomains',...birth},
  couple:{operation:'couple',personA:birth,personB:other},composite:{operation:'composite',personA:birth,personB:other},
  forecast:{operation:'forecast',birth,period:{kind:'day',year:2026,month:3,day:3,utcOffsetMinutes:420}},
  events:{operation:'events',period:{kind:'month',year:2026,month:3,utcOffsetMinutes:420}},
  query:{operation:'query',group:'geometry',action:'normalize',longitude:-10},
  chart:{operation:'chart',positions:[{id:'sun',longitude:350,speed:1},{id:'moon',longitude:10,speed:13}]},
  harmonic:{operation:'harmonic',harmonic:3,positions:[{id:'sun',longitude:350,speed:1},{id:'moon',longitude:10,speed:13}]},
  synastry:{operation:'synastry',positions:[{id:'sun',longitude:350,speed:1},{id:'moon',longitude:10,speed:13}],otherPositions:[{id:'venus',longitude:130}]}
};
const request = document.getElementById('request');const sample = document.getElementById('sample');const run = document.getElementById('run');const status = document.getElementById('request-status');const download = document.getElementById('download');let result;
function setSample() {request.value=JSON.stringify(samples[sample.value],null,2);status.textContent='Dữ liệu sinh là mẫu thử. UTC và tọa độ được truyền đúng như JSON.';}
function downloadJSON(envelope) {const url=URL.createObjectURL(new Blob([JSON.stringify(envelope,null,2)],{type:'application/json'}));const link=document.createElement('a');link.href=url;link.download='astro-result.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
const ui=createAstroUI({element:document.getElementById('astro-result'),onDownload:downloadJSON,calculate:async raw => {
  const response=await fetch('/api/astro/calculate',{method:'POST',headers:{'Content-Type':'application/json'},body:typeof raw==='string'?raw:JSON.stringify(raw)});
  const envelope=await response.json();if(!response.ok) throw new Error(envelope.error?.message || `HTTP ${response.status}`);return envelope;
}});
sample.addEventListener('change',setSample);setSample();download.addEventListener('click',()=>result && downloadJSON(result));
document.getElementById('request-form').addEventListener('submit',async event => {
  event.preventDefault();run.disabled=true;download.hidden=true;const started=performance.now();status.textContent='Đang tính trên native worker…';
  try {result=await ui.calculateRequest(request.value);download.hidden=false;status.textContent=`${result.errors?.length ? 'Engine trả lỗi đầu vào' : 'Tính xong'} · ${Math.round(performance.now()-started)} ms · ${result.engineVersion || 'engine'}`;}
  catch(error){status.textContent=error.message;}finally{run.disabled=false;}
});
try {const response=await fetch('/api/astro/status');const host=await response.json();document.getElementById('host-status').textContent=host.available?`Engine ${host.engineVersion} · local worker`:`Engine chưa sẵn sàng: ${host.reason || host.error?.message || 'native package unavailable'}`;run.disabled=!host.available;}catch(error){document.getElementById('host-status').textContent=`Host chưa sẵn sàng: ${error.message}`;}
window.addEventListener('pagehide',()=>ui.close(),{once:true});
