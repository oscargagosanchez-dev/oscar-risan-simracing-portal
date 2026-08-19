(()=>{
  const esc=s=>String(s||'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot',"'":'&#39;'}[c]));
  const norm=s=>String(s||'').normalize('NFD').replace(/[\u0300-\u036f]/g,'').trim().toLowerCase().replace(/[^a-z0-9]+/g,' ');
  const simKey=s=>{
    const v=norm(s);
    if(v==='le mans ultimate'||v==='lmu')return'lmu';
    if(v==='raceroom'||v==='race room'||v==='r3e')return'raceroom';
    if(v==='assetto corsa evo'||v==='ac evo'||v==='acevo')return'acevo';
    return v;
  };
  const TZ_KEY='or-agenda-timezone-v2';
  let timezone='madrid';
  try{const saved=localStorage.getItem(TZ_KEY);if(['madrid','local','utc'].includes(saved))timezone=saved}catch{}
  let items=[];
  let mode='all';
  const tzSelect=document.querySelector('#agendaTz');
  const tzLabel=document.querySelector('#agendaTzLabel');
  if(tzSelect)tzSelect.value=timezone;
  function currentZone(){
    if(timezone==='madrid')return'Europe/Madrid';
    if(timezone==='utc')return'UTC';
    try{return Intl.DateTimeFormat().resolvedOptions().timeZone||undefined}catch{return undefined}
  }
  function zoneLabel(){
    if(timezone==='madrid')return'Europe/Madrid';
    if(timezone==='utc')return'UTC';
    return currentZone()||'Hora local';
  }
  function hasExactTime(e){
    const label=String(e.timeLabel||e.time||'').toLowerCase();
    if(e.datePrecision==='week'||e.datePrecision==='range')return false;
    if(/por confirmar|vigente|disponible durante|sin hora|horario/.test(label))return false;
    return Number.isFinite(new Date(e.start).getTime());
  }
  function formatDate(iso){
    const opts={weekday:'short',day:'numeric',month:'short'};
    const z=currentZone();if(z)opts.timeZone=z;
    return new Intl.DateTimeFormat('es-ES',opts).format(new Date(iso));
  }
  function formatTime(e){
    if(!hasExactTime(e))return esc(e.timeLabel||e.time||'Horario por confirmar');
    const opts={hour:'2-digit',minute:'2-digit',hour12:false};
    const z=currentZone();if(z)opts.timeZone=z;
    return `${new Intl.DateTimeFormat('es-ES',opts).format(new Date(e.start))} <small style="opacity:.65">${esc(zoneLabel())}</small>`;
  }
  function match(e){
    if(mode==='all')return true;
    if(mode.startsWith('sim:'))return simKey(e.sim||e.label)===mode.slice(4);
    if(mode==='sprint'||mode==='endurance')return norm(e.type)===mode;
    return (e.tags||[]).some(t=>norm(t)===norm(mode));
  }
  function card(e){
    return `<article class="page-card"><span class="kicker">${esc(e.label||e.sim||'SIMRACING')} · ${esc(formatDate(e.start))}</span><h3>${esc(e.title)}</h3><p>${esc(e.track||'Circuito por confirmar')}</p><div class="facts"><span>🕒 ${formatTime(e)}</span><span>⏱ ${esc(e.duration||'No indicada')}</span><span>☁️ ${esc(e.weather||'No indicado')}</span><span>🏷 ${(e.tags||[]).map(esc).join(' · ')||esc(e.type||'Evento')}</span></div></article>`;
  }
  function render(){
    const now=Date.now();
    const list=items.filter(e=>new Date(e.start).getTime()>=now).filter(match).sort((a,b)=>new Date(a.start)-new Date(b.start));
    document.querySelector('#fullEvents').innerHTML=list.length?list.map(card).join(''):'<div class="empty">No hay eventos publicados con este filtro.</div>';
    if(tzLabel)tzLabel.textContent=`Zona mostrada: ${zoneLabel()}`;
  }
  const filters=document.querySelector('#fullFilters');
  filters?.addEventListener('click',ev=>{
    const b=ev.target.closest('button[data-f]');
    if(!b||!filters.contains(b))return;
    filters.querySelectorAll('button').forEach(x=>x.classList.toggle('active',x===b));
    const raw=b.dataset.f||'all';
    mode=raw==='Le Mans Ultimate'?'sim:lmu':raw==='RaceRoom'?'sim:raceroom':raw==='Assetto Corsa EVO'?'sim:acevo':raw;
    render();
  });
  tzSelect?.addEventListener('change',()=>{
    timezone=['madrid','local','utc'].includes(tzSelect.value)?tzSelect.value:'madrid';
    try{localStorage.setItem(TZ_KEY,timezone)}catch{}
    render();
  });
  fetch('/api/events').then(r=>r.json()).then(d=>{items=d.items||[];render()}).catch(()=>document.querySelector('#fullEvents').innerHTML='<div class="empty">No se pudo cargar la agenda.</div>');
})();
