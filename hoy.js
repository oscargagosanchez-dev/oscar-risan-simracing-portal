(()=>{
  const esc=s=>String(s||'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot',"'":'&#39;'}[c]));
  const FAV_KEY='or-favorite-sims-v1';
  const TZ_KEY='or-agenda-timezone-v2';
  const zone=()=>{try{const v=localStorage.getItem(TZ_KEY);if(v==='utc')return'UTC';if(v==='local')return Intl.DateTimeFormat().resolvedOptions().timeZone||undefined}catch{}return'Europe/Madrid'};
  const favs=()=>{try{const v=JSON.parse(localStorage.getItem(FAV_KEY)||'[]');return Array.isArray(v)?v:[]}catch{return[]}};
  const dateKey=(d,z)=>{const p=new Intl.DateTimeFormat('en-CA',{timeZone:z,year:'numeric',month:'2-digit',day:'2-digit'}).formatToParts(d);const o=Object.fromEntries(p.map(x=>[x.type,x.value]));return`${o.year}-${o.month}-${o.day}`};
  const fmtTime=(d,z)=>new Intl.DateTimeFormat('es-ES',{timeZone:z,hour:'2-digit',minute:'2-digit',hour12:false}).format(d);
  const exact=e=>{const t=String(e.timeLabel||e.time||'').toLowerCase();return e.datePrecision!=='week'&&e.datePrecision!=='range'&&!/por confirmar|vigente|disponible durante|sin hora|horario/.test(t)&&Number.isFinite(new Date(e.start).getTime())};
  function addStyle(){if(document.querySelector('link[href="/hoy.css"]'))return;const l=document.createElement('link');l.rel='stylesheet';l.href='/hoy.css';document.head.appendChild(l)}
  function render(items){
    if(document.querySelector('#queCorrerHoy'))return;
    const eventos=document.querySelector('#eventos');if(!eventos)return;
    const z=zone(),now=new Date(),today=dateKey(now,z),favorite=favs();
    const list=(items||[]).filter(e=>{const d=new Date(e.start);return Number.isFinite(d.getTime())&&dateKey(d,z)===today&&d.getTime()>=now.getTime()}).sort((a,b)=>{const af=favorite.includes(a.sim||a.label)?1:0,bf=favorite.includes(b.sim||b.label)?1:0;return bf-af||new Date(a.start)-new Date(b.start)}).slice(0,3);
    const sec=document.createElement('section');sec.id='queCorrerHoy';sec.className='today-races';
    const cards=list.map((e,i)=>{const d=new Date(e.start),fav=favorite.includes(e.sim||e.label),tags=(e.tags||[]).slice(0,3);return`<article class="today-card ${i===0?'lead':''}"><div class="today-top"><span>${i===0?'RECOMENDADO':'HOY'}${fav?' · ★ FAVORITO':''}</span><b>${exact(e)?esc(fmtTime(d,z)):esc(e.timeLabel||'Horario por confirmar')}</b></div><h3>${esc(e.title||'Evento')}</h3><p>${esc(e.track||'Circuito por confirmar')} · ${esc(e.duration||'Duración no indicada')}</p><div class="today-tags"><span>${esc(e.label||e.sim||'SimRacing')}</span>${tags.map(t=>`<span>${esc(t)}</span>`).join('')}</div><div class="today-actions"><a href="/donde-correr.html">Ver en Agenda+ →</a>${e.sim?`<a href="/simulador.html?sim=${encodeURIComponent(e.sim)}">${esc(e.sim)} →</a>`:''}</div></article>`}).join('');
    sec.innerHTML=`<div class="today-head"><div><span class="ey">QUÉ CORRER HOY</span><h2>${list.length?'Tus mejores opciones para hoy':'No quedan carreras publicadas para hoy'}</h2></div><div class="today-zone">Horario · ${esc(z||'local')}</div></div>${list.length?`<div class="today-grid">${cards}</div>`:`<div class="today-empty"><p>La agenda no tiene más eventos futuros publicados para hoy en esta zona horaria.</p><a class="portal-action primary" href="/donde-correr.html">Ver próximas carreras →</a></div>`}`;
    eventos.before(sec);
  }
  addStyle();
  fetch('/api/events').then(r=>r.json()).then(d=>render(d.items||[])).catch(()=>{});
})();
