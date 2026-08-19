(()=>{
  const el=document.querySelector('#srpNews');
  if(!el)return;
  const esc=s=>String(s||'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  fetch('/api/srp').then(r=>r.json()).then(d=>{
    const items=d.items||[];
    el.innerHTML=items.length?items.map(x=>`<article class="srp-news-item"><span class="srp-news-date">${esc(x.date||'Actualidad SRP')}</span><b>${esc(x.title)}</b><a href="${esc(x.url)}" target="_blank" rel="noopener">Ver →</a></article>`).join(''):'<div class="empty">No hay novedades SRP disponibles ahora mismo.</div>';
  }).catch(()=>{el.innerHTML='<div class="empty">No se pudo cargar la actualidad SRP.</div>'});
})();
(()=>{if(document.querySelector('script[src="/portal-next.js"]'))return;const s=document.createElement('script');s.src='/portal-next.js';s.defer=true;document.body.appendChild(s)})();
