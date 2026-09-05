(()=>{
  if(window.__OR_ANALYTICS__)return;window.__OR_ANALYTICS__=true;
  window.va=window.va||function(){(window.vaq=window.vaq||[]).push(arguments)};
  const script=document.createElement('script');
  if(!document.querySelector('script[src="/_vercel/insights/script.js"]')){script.defer=true;script.src='/_vercel/insights/script.js';document.head.appendChild(script)}
  const clean=s=>String(s||'').replace(/\s+/g,' ').trim().slice(0,60);
  const safeTerm=s=>{const v=clean(s);if(!v)return'';if(/@|https?:\/\/|www\./i.test(v))return'[oculto]';return v.slice(0,40)};
  const send=(name,data={})=>{try{window.va('event',{name,data})}catch{}};
  const source=el=>el.closest('#queCorrerHoy')?'que-correr-hoy':el.closest('#semanaSimracing')?'semana':el.closest('#featuredNews')?'destacada':el.closest('#noticias')?'noticias':el.closest('#simHub')?'simuladores':el.closest('#eventos')?'eventos':location.pathname;

  document.addEventListener('submit',e=>{
    const f=e.target.closest('#globalSearchForm');if(!f)return;
    const input=f.querySelector('#globalSearchInput');const q=safeTerm(input?.value);
    if(q)send('Buscar',{termino:q,origen:'formulario'});
  },true);

  document.addEventListener('click',e=>{
    const target=e.target.closest('a,button');if(!target)return;
    if(target.matches('.search-shortcuts button[data-q]')){const q=safeTerm(target.dataset.q);if(q)send('Buscar',{termino:q,origen:'atajo'});return}
    if(target.matches('.fav-chip[data-sim]')){const sim=clean(target.dataset.sim);setTimeout(()=>send('Favorito',{simulador:sim,estado:target.classList.contains('active')?'activado':'desactivado'}),0);return}
    if(target.matches('[data-ics]')){send('Calendario',{tipo:'ics',origen:source(target)});return}
    if(target.tagName==='A'){
      const href=target.getAttribute('href')||'';
      if(/calendar\.google\.com/i.test(href)){send('Calendario',{tipo:'google',origen:source(target)});return}
      if(href.includes('/donde-correr.html')){send('AbrirAgenda',{origen:source(target)});return}
      if(href.includes('/simulador.html')){const u=new URL(href,location.origin);send('AbrirSimulador',{simulador:clean(u.searchParams.get('sim')||target.textContent),origen:source(target)});return}
      if(href.includes('/article.html')){send('AbrirNoticia',{origen:source(target)});return}
      try{const u=new URL(href,location.href);if(u.origin!==location.origin&&/^https?:$/.test(u.protocol)){let destino=u.hostname.replace(/^www\./,'');if(destino.includes('twitch.tv'))destino='Twitch';else if(destino.includes('youtube.com'))destino='YouTube';else if(destino==='x.com'||destino.includes('twitter.com'))destino='X';else if(destino.includes('simracing-pro.com'))destino='SRP';else if(destino.includes('onesimpro.com'))destino='OneSimPro';send('EnlaceExterno',{destino,origen:source(target)})}}catch{}
    }
  },true);
})();
