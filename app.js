const host=location.hostname,esc=s=>String(s||'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
document.querySelector('#tp').src=`https://player.twitch.tv/?channel=oscarrisan&parent=${encodeURIComponent(host)}&autoplay=false`;
document.querySelector('#tc').src=`https://www.twitch.tv/embed/oscarrisan/chat?parent=${encodeURIComponent(host)}&darkpopout`;
function mondayOf(d){const x=new Date(d);const day=(x.getDay()+6)%7;x.setHours(0,0,0,0);x.setDate(x.getDate()-day);return x}
function atWeek(dayOffset,time,weekOffset=0){const [h,m]=time.split(':').map(Number);const d=mondayOf(new Date());d.setDate(d.getDate()+dayOffset+(weekOffset*7));d.setHours(h,m,0,0);return d}
function endFrom(start,durationMin){return new Date(start.getTime()+durationMin*60000)}
function mk(e){const start=atWeek(e.day,e.time,e.week||0);return {...e,start:start.toISOString(),end:endFrom(start,e.durationMin||60).toISOString(),weekOffset:e.week||0,lastUpdated:e.lastUpdated||new Date(Date.now()-90*60000).toISOString()}}
let allEvents=[];
let selectedWeek='today';
function weekMatch(e,mode){const now=new Date(),s=new Date(e.start),m=mondayOf(now),n=new Date(m);n.setDate(n.getDate()+7),n2=new Date(n);n2.setDate(n2.getDate()+7);if(mode==='today')return s.toDateString()===now.toDateString();if(mode==='current')return s>=m&&s<n;return s>=n&&s<n2}
const A=window.EVENT_ASSETS||{};
const eventFilters=[['Todos','all'],['LMU','Le Mans Ultimate'],['RaceRoom','RaceRoom'],['AC EVO','Assetto Corsa EVO'],['Sprint','type:sprint'],['Endurance','type:endurance'],['Multiclase','tag:MULTICLASE'],['Ranked','tag:RANKED'],['Especial','type:special']];

const eventText=e=>[
  e.title,e.track,e.category,e.class,e.classes,e.car,e.cars,e.label,e.sim,
  ...(Array.isArray(e.tags)?e.tags:[]),
  ...Object.values(e.details||{}).filter(v=>typeof v==='string')
].filter(Boolean).join(' ');
const EVENT_VISUALS=[
  {track:/spa/i, cars:/(lmp2|lmgt3|gte|gt3|prototype|hypercar|wec|elms)/i, src:A['spa-lmp2-lmgt3']},
  {track:/spa/i, cars:/(wet|rain|lluvia|mojado|multiclass|multiclase)/i, src:A['spa-wet-multiclass']},
  {track:/le mans|circuit de la sarthe/i, cars:/(hypercar|lmgt3|gte|lmp2|wec)/i, src:A['lemans-hypercar-lmgt3']},
  {track:/n[uü]rburgring|nordschleife/i, cars:/(dtm|touring|gt3|german touring)/i, src:A['nurburgring-dtm']},
  {track:/hockenheim/i, cars:/(super touring|touring|wtcc|tcr|dtm)/i, src:A['hockenheim-super-touring']},
  {track:/monza/i, cars:/(gt3|lmgt3|gte|gt trophy|gt)/i, src:A['monza-gt3']},
  {track:/silverstone/i, cars:/(roadster|mx-5|mazda|road car|touring)/i, src:A['silverstone-roadster']},
  {track:/suzuka/i, cars:/(gt3|gte|gt4|gt|touring)/i, src:A['suzuka-gt']},
];
const CIRCUIT_VISUALS=[
  [/spa/i,A['spa-lmp2-lmgt3']],
  [/le mans|circuit de la sarthe/i,A['lemans-hypercar-lmgt3']],
  [/n[uü]rburgring|nordschleife/i,A['nurburgring-dtm']],
  [/hockenheim/i,A['hockenheim-super-touring']],
  [/monza/i,A['monza-gt3']],
  [/silverstone/i,A['silverstone-roadster']],
  [/suzuka/i,A['suzuka-gt']],
];
function eventVisual(e){
  const track=String(e.track||'');
  const text=eventText(e);
  const exact=EVENT_VISUALS.find(v=>v.track.test(track)&&v.cars.test(text));
  if(exact)return exact.src;
  const circuit=CIRCUIT_VISUALS.find(([rx])=>rx.test(track));
  if(circuit)return circuit[1];
  if(/lmp|hypercar|prototype/i.test(text))return A['lemans-hypercar-lmgt3'];
  if(/gt3|lmgt3|gte|gt4|\bgt\b/i.test(text))return A['monza-gt3'];
  if(/touring|dtm|wtcc|tcr/i.test(text))return A['nurburgring-dtm'];
  return A['monza-gt3'];
}

function eventChipClass(t){if(t==='ENDURANCE')return'c-end';if(t==='MULTICLASE')return'c-multi';if(t==='OFICIAL')return'c-off';if(t==='SPRINT')return'c-sprint';if(t==='RANKED')return'c-ranked';return'c-special'}
function eventCard(e){const visual=eventVisual(e);return `<article class="event-card"><div class="event-img"><img src="${esc(visual)}" alt="${esc(e.track)} · ${esc(e.title)}" onerror="if(this.dataset.fallback!=='1'){this.dataset.fallback='1';this.src=window.EVENT_ASSETS['monza-gt3'] }"><div class="event-overlay"></div><span class="event-sim">${esc(e.label||e.sim)}</span><span class="event-time">${esc(e.time)}</span><div class="event-content"><h3>${esc(e.title)}</h3><p>${esc(e.track)}</p><div class="chips">${e.tags.map(t=>`<span class="chip ${eventChipClass(t)}">${esc(t)}</span>`).join('')}</div></div></div><div class="event-meta"><div>🕒 ${esc(e.duration)}</div><div>☁️ ${esc(e.weather)}</div>${e.fuel?`<div>⛽ ${esc(e.fuel)}</div>`:''}${e.tyres?`<div>🛞 ${esc(e.tyres)}</div>`:''}</div><button class="event-btn" type="button" onclick="openEventDetails('${e.id}')">Ver detalles</button></article>`}
function openEventDetails(id){const e=allEvents.find(x=>x.id===id);if(!e)return;const mi=document.querySelector('#modalEventImage');mi.src=eventVisual(e);mi.alt=`${e.track||''} · ${e.title||''}`;mi.onerror=()=>{mi.onerror=null;mi.src=window.EVENT_ASSETS['monza-gt3']};document.querySelector('#modalSim').textContent=e.sim;document.querySelector('#modalTitle').textContent=e.title;document.querySelector('#modalTrack').textContent=e.track||'';document.querySelector('#modalTags').innerHTML=(e.tags||[]).map(t=>`<span class="chip ${eventChipClass(t)}">${esc(t)}</span>`).join('');const d=e.details||{};const values=[['Fecha',new Date(e.start).toLocaleDateString('es-ES',{weekday:'long',day:'numeric',month:'long',year:'numeric'})],['Hora',e.timeLabel||e.time||new Date(e.start).toLocaleTimeString('es-ES',{hour:'2-digit',minute:'2-digit'})],['Duración',e.duration||'No indicada'],['Formato',d.format||e.type||'No indicado'],['Práctica',d.practice||'No indicada'],['Clasificación',d.qualifying||'No indicada'],['Carrera',d.race||e.duration||'No indicada'],['Condiciones',e.weather||'No indicadas'],['Combustible',e.fuel||'No indicado'],['Neumáticos',e.tyres||'No indicado']].filter(x=>x[1]);document.querySelector('#modalDetails').innerHTML=values.map(([k,v])=>`<div class="detail-box"><small>${esc(k)}</small><b>${esc(v)}</b></div>`).join('');document.querySelector('#modalSource').textContent=`Fuente: ${e.source||'fuente oficial'}. Información mostrada dentro de Oscar Risan SimRacing.`;const m=document.querySelector('#eventModal');m.classList.add('open');m.setAttribute('aria-hidden','false');document.body.style.overflow='hidden'}
function closeEventDetails(){const m=document.querySelector('#eventModal');m.classList.remove('open');m.setAttribute('aria-hidden','true');document.body.style.overflow=''}
document.querySelector('#closeEventModal').onclick=closeEventDetails;document.querySelector('#eventModal').addEventListener('click',e=>{if(e.target.id==='eventModal')closeEventDetails()});document.addEventListener('keydown',e=>{if(e.key==='Escape')closeEventDetails()});
function filterEvents(mode){let base=allEvents.filter(e=>weekMatch(e,selectedWeek));if(mode==='all')return base;if(mode.startsWith('type:'))return base.filter(e=>e.type===mode.split(':')[1]);if(mode.startsWith('tag:'))return base.filter(e=>e.tags.includes(mode.split(':')[1]));return base.filter(e=>e.sim===mode)}
function renderEvents(mode='all'){const list=filterEvents(mode);document.querySelector('#eventCards').innerHTML=list.slice(0,4).map(eventCard).join('')||'<div class="empty">No hay eventos publicados para este periodo/filtro.</div>';renderAgenda();}
document.querySelector('#eventnav').innerHTML=eventFilters.map(([label,val],i)=>`<button class="${i===0?'active':''}" data-f="${esc(val)}">${esc(label)}</button>`).join('');document.querySelectorAll('#eventnav button').forEach(b=>b.onclick=()=>{document.querySelectorAll('#eventnav button').forEach(x=>x.classList.toggle('active',x===b));renderEvents(b.dataset.f);document.querySelector('#eventos').scrollIntoView({behavior:'smooth'})});
document.querySelectorAll('#weekTabs .week-tab').forEach(b=>b.onclick=()=>{selectedWeek=b.dataset.week;document.querySelectorAll('#weekTabs .week-tab').forEach(x=>x.classList.toggle('active',x===b));renderEvents(document.querySelector('#eventnav button.active')?.dataset.f||'all')});
function relativeUpdate(iso){const mins=Math.max(0,Math.round((Date.now()-new Date(iso).getTime())/60000));if(mins<60)return `hace ${mins||1} min`;const h=Math.round(mins/60);return h<24?`hace ${h} h`:`hace ${Math.round(h/24)} d`}
function renderFreshness(){const latest={};for(const e of allEvents){if(!latest[e.sim]||new Date(e.lastUpdated)>new Date(latest[e.sim]))latest[e.sim]=e.lastUpdated}document.querySelector('#freshness').innerHTML=Object.entries(latest).slice(0,6).map(([sim,t])=>`<span class="fresh-badge"><strong>${esc(sim)}</strong> · actualizado ${relativeUpdate(t)}</span>`).join('')}
renderFreshness();
const sList=document.querySelector('#scheduleList');const simPill=e=>`<span class="sim-pill ${e.sim==='Le Mans Ultimate'?'lmu':e.sim==='Automobilista 2'?'ams2':e.sim==='RaceRoom'?'r3e':e.sim==='Euro Truck Simulator 2'?'ets2':e.sim==='American Truck Simulator'?'ats':'ace'}">${esc(e.label)}</span>`;
function renderAgenda(){const activeFilter=document.querySelector('#eventnav button.active')?.dataset.f||'all';const base=filterEvents(activeFilter);sList.innerHTML=[...base].sort((a,b)=>new Date(a.start)-new Date(b.start)).slice(0,12).map(e=>`<div class="s-item"><div class="s-time">${new Date(e.start).toLocaleTimeString('es-ES',{hour:'2-digit',minute:'2-digit'})}</div><div class="s-main"><b>${esc(e.title)}</b><small>${new Date(e.start).toLocaleDateString('es-ES',{weekday:'short',day:'numeric',month:'short'})} · ${esc(e.track)} · ${esc(e.duration)}</small></div><div class="s-side">${simPill(e)}</div></div>`).join('')||'<div class="empty">No hay eventos publicados para este periodo/filtro.</div>'}
const dt=new Date();document.querySelector('#todayDate').textContent=dt.toLocaleDateString('es-ES',{weekday:'long',day:'numeric',month:'long',year:'numeric'});
let all=[];const F=[['Todas','Todas'],['LMU','Le Mans Ultimate'],['AMS2','Automobilista 2'],['AC EVO','Assetto Corsa EVO'],['AC Rally','Assetto Corsa Rally'],['RaceRoom','RaceRoom'],['ETS2','Euro Truck Simulator 2'],['ATS','American Truck Simulator'],['Hardware','Hardware']],marks={'Le Mans Ultimate':'LMU','Automobilista 2':'AMS2','Assetto Corsa EVO':'AC EVO','Assetto Corsa Rally':'AC RALLY','RaceRoom':'R3E','Euro Truck Simulator 2':'ETS2','American Truck Simulator':'ATS','Hardware':'HW','Trucking':'TRUCK'};function ok(n,c){if(c==='Todas')return true;if(c==='Euro Truck Simulator 2')return n.category===c||(n.category==='Trucking'&&/euro truck|ets2/i.test((n.title||'')+' '+(n.summary||'')));if(c==='American Truck Simulator')return n.category===c||(n.category==='Trucking'&&/american truck|\bats\b/i.test((n.title||'')+' '+(n.summary||'')));return String(n.category||'').toLowerCase()===c.toLowerCase()}function tease(s,n=205){s=String(s||'').replace(/\s+/g,' ').trim();return s.length>n?s.slice(0,n).replace(/\s+\S*$/,'')+'…':s}function card(n){return `<article class="card">${n.image?`<a href="/article.html?id=${encodeURIComponent(n.id)}"><img src="${esc(n.image)}" alt="" onerror="this.remove()"></a>`:''}<div class="body"><div class="meta"><span><span class="logo" data-c="${esc(n.category)}">${esc(marks[n.category]||'SIM')}</span>${esc(n.category)}</span><time>${n.date?new Date(n.date).toLocaleDateString('es-ES'):''}</time></div><div class="source">${esc(n.source)}</div><div class="tags">${(n.tags||[]).map(t=>`<span class="tag">${esc(t)}</span>`).join('')}</div><h3><a href="/article.html?id=${encodeURIComponent(n.id)}">${esc(n.title)}</a></h3><p>${esc(tease(n.summary))}</p><a class="read" href="/article.html?id=${encodeURIComponent(n.id)}">Leer noticia →</a></div></article>`}function render(c='Todas'){const x=all.filter(n=>ok(n,c));document.querySelector('#newsTitle').textContent=c==='Todas'?'Últimas noticias':c;document.querySelector('#news').innerHTML=x.slice(0,18).map(card).join('')||'<div class="empty">No hay noticias disponibles de esta categoría ahora mismo.</div>'}const cn=document.createElement('div');cn.className='catnav';cn.innerHTML=F.map(([l,c],i)=>`<button class="${i?'':'active'}" data-c="${esc(c)}">${esc(l)}</button>`).join('');document.querySelector('#noticias').before(cn);cn.querySelectorAll('button').forEach(b=>b.onclick=()=>{cn.querySelectorAll('button').forEach(x=>x.classList.toggle('active',x===b));render(b.dataset.c);document.querySelector('#noticias').scrollIntoView({behavior:'smooth'})});fetch('/api/news').then(r=>r.json()).then(d=>{all=d.items||[];render()}).catch(()=>document.querySelector('#news').innerHTML='<div class="empty">No se pudieron cargar las noticias.</div>');
fetch('/api/youtube').then(r=>r.json()).then(d=>{const v=d.items||[],el=document.querySelector('#yv');el.innerHTML=v.length?v.slice(0,6).map(x=>`<article class="video"><a class="thumb" href="${esc(x.url)}" target="_blank"><img src="${esc(x.thumbnail)}" alt="${esc(x.title)}"><span class="play">▶</span></a><div class="vb"><h3><a href="${esc(x.url)}" target="_blank">${esc(x.title)}</a></h3><time>${x.published?new Date(x.published).toLocaleDateString('es-ES'):''}</time></div></article>`).join(''):'<div class="empty">No se han podido cargar los vídeos. <a href="https://www.youtube.com/@oscarrisansimracing6480" target="_blank">Abrir canal →</a></div>'}).catch(()=>document.querySelector('#yv').innerHTML='<div class="empty">No se han podido cargar los vídeos.</div>');
fetch('/api/events').then(r=>r.json()).then(d=>{
  allEvents=(d.items||[]).map(e=>{
    const start=new Date(e.start);
    return {...e,
      time:e.time||e.timeLabel||start.toLocaleTimeString('es-ES',{hour:'2-digit',minute:'2-digit'}),
      tags:Array.isArray(e.tags)?e.tags:[],
      type:e.type||'special',
      duration:e.duration||'',
      weather:e.weather||'',
      fuel:e.fuel||'',
      tyres:e.tyres||'',
      img:e.img||'https://images.unsplash.com/photo-1503376780353-7e6692767b70?auto=format&fit=crop&w=1200&q=80',
      lastUpdated:e.lastUpdated||d.updatedAt||new Date().toISOString()
    };
  });
  renderFreshness();
  renderEvents();
}).catch(()=>{
  allEvents=[];
  renderFreshness();
  renderEvents();
});
