(()=>{
  const el=document.querySelector('#srpNews');
  if(!el)return;
  const esc=s=>String(s||'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  fetch('/api/srp').then(r=>r.json()).then(d=>{
    const items=d.items||[];
    el.innerHTML=items.length?items.map(x=>`<article class="srp-news-item"><span class="srp-news-date">${esc(x.date||'Actualidad SRP')}</span><b>${esc(x.title)}</b><a href="${esc(x.url)}" target="_blank" rel="noopener">Ver →</a></article>`).join(''):'<div class="empty">No hay novedades SRP disponibles ahora mismo.</div>';
  }).catch(()=>{el.innerHTML='<div class="empty">No se pudo cargar la actualidad SRP.</div>'});
})();
(()=>{if(!document.querySelector('script[src="/portal-next.js"]')){const p=document.createElement('script');p.src='/portal-next.js';p.defer=true;document.body.appendChild(p)}if(document.querySelector('script[src="/timezone.js"]'))return;const t=document.createElement('script');t.src='/timezone.js';t.onload=()=>{if(document.querySelector('script[src="/timezone-main.js"]'))return;const m=document.createElement('script');m.src='/timezone-main.js';m.defer=true;document.body.appendChild(m)};document.body.appendChild(t)})();
