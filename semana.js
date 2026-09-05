(()=>{
  const esc=s=>String(s||'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const FAV_KEY='or-favorite-sims-v1';
  const TZ_KEY='or-agenda-timezone-v2';
  const zone=()=>{try{const v=localStorage.getItem(TZ_KEY);if(v==='utc')return'UTC';if(v==='local')return Intl.DateTimeFormat().resolvedOptions().timeZone||undefined}catch{}return'Europe/Madrid'};
  const favs=()=>{try{const v=JSON.parse(localStorage.getItem(FAV_KEY)||'[]');return Array.isArray(v)?v:[]}catch{return[]}};
  const fmtDate=(d,z)=>new Intl.DateTimeFormat('es-ES',{timeZone:z,weekday:'short',day:'numeric',month:'short'}).format(d);
  const fmtTime=(d,z)=>new Intl.DateTimeFormat('es-ES',{timeZone:z,hour:'2-digit',minute:'2-digit',hour12:false}).format(d);
  const exact=e=>{const t=String(e.timeLabel||e.time||'').toLowerCase();return e.datePrecision!=='week'&&e.datePrecision!=='range'&&!/por confirmar|vigente|disponible durante|sin hora|horario/.test(t)&&Number.isFinite(new Date(e.start).getTime())};
  function weekBounds(z){
    const now=new Date();
    const p=new Intl.DateTimeFormat('en-US',{timeZone:z,weekday:'short',year:'numeric',month:'2-digit',day:'2-digit'}).formatToParts(now);
    const o=Object.fromEntries(p.map(x=>[x.type,x.value]));
    const base=new Date(`${o.year}-${o.month}-${o.day}T12:00:00Z`);
    const dow={Sun:0,Mon:1,Tue:2,Wed:3,Thu:4,Fri:5,Sat:6}[o.weekday]??1;
    const diff=(dow+6)%7;
    const start=new Date(base);start.setUTCDate(start.getUTCDate()-diff);start.setUTCHours(0,0,0,0);
    const end=new Date(start);end.setUTCDate(end.getUTCDate()+7);return[start,end];
  }
  function newsScore(n,f){
    let s=0;const txt=`${n.title||''} ${(n.tags||[]).join(' ')}`.toLowerCase();
    if(f.includes(n.category))s+=5;
    if(/actualiz|update|patch|hotfix/.test(txt))s+=4;
    if(/contenido|content|dlc|coche|car|circuit|track/.test(txt))s+=3;
    if(/evento|event|championship|competition/.test(txt))s+=2;
    if(/traxion|overtake/i.test(n.source||''))s+=1;
    s+=Math.max(0,7-Math.floor((Date.now()-new Date(n.date).getTime())/86400000));
    return s;
  }
  function eventScore(e,f){
    let s=0;const txt=`${e.type||''} ${(e.tags||[]).join(' ')}`.toLowerCase();
    if(f.includes(e.sim||e.label))s+=6;
    if(/especial|special/.test(txt))s+=4;
    if(/endurance|resistencia/.test(txt))s+=3;
    if(/multiclase|multiclass/.test(txt))s+=2;
    if(/ranked/.test(txt))s+=1;
    return s;
  }
  function addStyle(){if(document.querySelector('link[href="/semana.css"]'))return;const l=document.createElement('link');l.rel='stylesheet';l.href='/semana.css';document.head.appendChild(l)}
  function render(news,events){
    if(document.querySelector('#semanaSimracing'))return;
    const anchor=document.querySelector('#noticias')||document.querySelector('#canales');if(!anchor)return;
    const z=zone(),f=favs(),now=Date.now(),[weekStart,weekEnd]=weekBounds(z),sevenDays=now-7*86400000;
    const selectedNews=(news||[]).filter(n=>{const t=new Date(n.date).getTime();return Number.isFinite(t)&&t>=sevenDays&&t<=now+3600000}).sort((a,b)=>newsScore(b,f)-newsScore(a,f)||new Date(b.date)-new Date(a.date)).slice(0,4);
    const selectedEvents=(events||[]).filter(e=>{const t=new Date(e.start).getTime();return Number.isFinite(t)&&t>=Math.max(now,weekStart.getTime())&&t<weekEnd.getTime()}).sort((a,b)=>eventScore(b,f)-eventScore(a,f)||new Date(a.start)-new Date(b.start)).slice(0,4);
    const sec=document.createElement('section');sec.id='semanaSimracing';sec.className='week-brief sec';
    const newsHtml=selectedNews.map(n=>`<a class="week-news-item" href="/article.html?id=${encodeURIComponent(n.id)}"><span>${esc(n.category||'SimRacing')}</span><b>${esc(n.title)}</b><small>${n.date?esc(fmtDate(new Date(n.date),z)):''} · ${esc(n.source||'Fuente')}</small></a>`).join('');
    const eventsHtml=selectedEvents.map(e=>{const d=new Date(e.start);return`<a class="week-event-item" href="/donde-correr.html"><div class="week-event-time"><b>${exact(e)?esc(fmtTime(d,z)):'—'}</b><small>${esc(fmtDate(d,z))}</small></div><div><span>${esc(e.label||e.sim||'Evento')}</span><b>${esc(e.title||'Carrera')}</b><small>${esc(e.track||'Circuito por confirmar')} · ${esc(e.duration||'Duración no indicada')}</small></div></a>`}).join('');
    const lead=selectedNews[0];
    sec.innerHTML=`<div class="week-head"><div><span class="ey">ESTA SEMANA EN SIMRACING</span><h2>Lo importante, reunido en un vistazo</h2></div><div class="week-zone">Actualizado automáticamente · ${esc(z||'local')}</div></div><div class="week-layout"><div class="week-main"><span class="week-label">RESUMEN SEMANAL</span>${lead?`<h3>${esc(lead.title)}</h3><p>${esc((lead.summary||'').slice(0,280))}${(lead.summary||'').length>280?'…':''}</p><a class="portal-action primary" href="/article.html?id=${encodeURIComponent(lead.id)}">Leer destacada →</a>`:'<h3>Sin noticias destacadas esta semana</h3><p>Cuando haya novedades recientes, aparecerán aquí automáticamente.</p>'}</div><div class="week-column"><div class="week-column-head"><b>Noticias clave</b><a href="#noticias">Ver noticias →</a></div>${newsHtml||'<div class="week-empty">No hay noticias recientes disponibles.</div>'}</div><div class="week-column"><div class="week-column-head"><b>Carreras de la semana</b><a href="/donde-correr.html">Agenda+ →</a></div>${eventsHtml||'<div class="week-empty">No hay más eventos publicados esta semana.</div>'}</div></div>`;
    anchor.before(sec);
  }
  addStyle();
  Promise.all([fetch('/api/news').then(r=>r.json()),fetch('/api/events').then(r=>r.json())]).then(([n,e])=>render(n.items||[],e.items||[])).catch(()=>{});
})();
