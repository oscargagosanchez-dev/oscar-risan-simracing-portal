(()=>{
  const T=window.OR_TIMEZONE;
  const esc=s=>String(s||'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const norm=s=>String(s||'').normalize('NFD').replace(/[\u0300-\u036f]/g,'').trim().toLowerCase().replace(/[^a-z0-9]+/g,' ');
  const simKey=s=>{const v=norm(s);if(v==='le mans ultimate'||v==='lmu')return'lmu';if(v==='raceroom'||v==='race room'||v==='r3e')return'raceroom';if(v==='assetto corsa evo'||v==='ac evo'||v==='acevo')return'acevo';return v};
  let items=[];let mode='all';
  function match(e){if(mode==='all')return true;if(mode.startsWith('sim:'))return simKey(e.sim||e.label)===mode.slice(4);if(mode==='sprint'||mode==='endurance')return norm(e.type)===mode;return(e.tags||[]).some(t=>norm(t)===norm(mode))}
  function eventDate(e){return T?T.formatDate(e.start):new Date(e.start).toLocaleDateString('es-ES',{weekday:'short',day:'numeric',month:'short'})}
  function eventTime(e){if(T&&!T.exactEvent(e))return esc(e.timeLabel||e.time||'Horario por confirmar');if(T)return `${T.formatTime(e.start)} <small class="tz-inline">${esc(T.label())}</small>`;return new Date(e.start).toLocaleTimeString('es-ES',{hour:'2-digit',minute:'2-digit'})}
  function card(e){return `<article class="page-card"><span class="kicker">${esc(e.label||e.sim||'SIMRACING')} · ${esc(eventDate(e))}</span><h3>${esc(e.title)}</h3><p>${esc(e.track||'Circuito por confirmar')}</p><div class="facts"><span>🕒 ${eventTime(e)}</span><span>⏱ ${esc(e.duration||'No indicada')}</span><span>☁️ ${esc(e.weather||'No indicado')}</span><span>🏷 ${(e.tags||[]).map(esc).join(' · ')||esc(e.type||'Evento')}</span></div></article>`}
  function render(){const now=Date.now();const list=items.filter(e=>new Date(e.start).getTime()>=now).filter(match).sort((a,b)=>new Date(a.start)-new Date(b.start));document.querySelector('#fullEvents').innerHTML=list.length?list.map(card).join(''):'<div class="empty">No hay eventos publicados con este filtro.</div>'}
  const tz=document.querySelector('#agendaTimezone');if(T&&tz){tz.innerHTML=T.selectorHTML('orTimezoneAgenda');T.bind(tz.querySelector('select'),render)}
  const filters=document.querySelector('#fullFilters');filters?.addEventListener('click',ev=>{const b=ev.target.closest('button[data-f]');if(!b||!filters.contains(b))return;filters.querySelectorAll('button').forEach(x=>x.classList.toggle('active',x===b));const raw=b.dataset.f||'all';mode=raw==='Le Mans Ultimate'?'sim:lmu':raw==='RaceRoom'?'sim:raceroom':raw==='Assetto Corsa EVO'?'sim:acevo':raw;render()});
  fetch('/api/events').then(r=>r.json()).then(d=>{items=d.items||[];render()}).catch(()=>document.querySelector('#fullEvents').innerHTML='<div class="empty">No se pudo cargar la agenda.</div>');
})();
