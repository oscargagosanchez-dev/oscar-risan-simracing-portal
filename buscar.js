(()=>{
  const esc=s=>String(s||'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot',"'":'&#39;'}[c]));
  const norm=s=>String(s||'').normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase().replace(/[^a-z0-9]+/g,' ').trim();
  const input=document.querySelector('#globalSearchInput'),form=document.querySelector('#globalSearchForm'),results=document.querySelector('#searchResults'),summary=document.querySelector('#searchSummary');
  let news=[],events=[];
  const staticItems=[
    {type:'Simulador',title:'Le Mans Ultimate',text:'LMU endurance hypercar LMP2 LMGT3 WEC ELMS',url:'/simulador.html?sim=Le%20Mans%20Ultimate'},
    {type:'Simulador',title:'RaceRoom',text:'RaceRoom ranked touring DTM super touring R3E',url:'/simulador.html?sim=RaceRoom'},
    {type:'Simulador',title:'Assetto Corsa EVO',text:'Assetto Corsa EVO AC EVO Kunos daily races',url:'/simulador.html?sim=Assetto%20Corsa%20EVO'},
    {type:'Simulador',title:'Assetto Corsa Rally',text:'Assetto Corsa Rally AC Rally rally clásicos',url:'/simulador.html?sim=Assetto%20Corsa%20Rally'},
    {type:'Simulador',title:'Automobilista 2',text:'Automobilista 2 AMS2 Reiza',url:'/simulador.html?sim=Automobilista%202'},
    {type:'Simulador',title:'Euro Truck Simulator 2',text:'ETS2 Euro Truck Simulator 2 SCS',url:'/simulador.html?sim=Euro%20Truck%20Simulator%202'},
    {type:'Simulador',title:'American Truck Simulator',text:'ATS American Truck Simulator SCS',url:'/simulador.html?sim=American%20Truck%20Simulator'},
    {type:'Hardware',title:'SRP · SimRacing-Pro',text:'SRP pedales GT-S GT-R Formula-R freno de mano hardware',url:'/#srp'},
    {type:'Hardware',title:'OneSimPro',text:'OneSimPro OSP dashboard iFLAG 550 PRO GT EVO control box hardware',url:'/#osp'},
    {type:'Agenda',title:'¿Dónde correr? · Agenda+',text:'carreras eventos agenda calendario LMU RaceRoom AC EVO',url:'/donde-correr.html'}
  ];
  function score(text,q){const t=norm(text),parts=norm(q).split(' ').filter(Boolean);if(!parts.length)return 0;let s=0;for(const p of parts){if(t===p)s+=8;else if(t.includes(' '+p+' ')||t.startsWith(p+' ')||t.endsWith(' '+p))s+=5;else if(t.includes(p))s+=2;else return 0}return s}
  function run(q){q=String(q||'').trim();if(!q){results.innerHTML='';summary.textContent='Escribe algo para empezar.';return}
    const list=[];
    for(const n of news){const s=score([n.title,n.summary,n.category,n.source,(n.tags||[]).join(' ')].join(' '),q);if(s)list.push({score:s+3,type:'Noticia',title:n.title,text:[n.category,n.source].filter(Boolean).join(' · '),url:`/article.html?id=${encodeURIComponent(n.id)}`})}
    for(const e of events){const s=score([e.title,e.track,e.sim,e.label,e.category,e.class,e.classes,e.car,e.cars,(e.tags||[]).join(' ')].join(' '),q);if(s)list.push({score:s+2,type:'Carrera',title:e.title,text:[e.label||e.sim,e.track,e.duration].filter(Boolean).join(' · '),url:'/donde-correr.html'})}
    for(const x of staticItems){const s=score([x.title,x.text,x.type].join(' '),q);if(s)list.push({...x,score:s+1})}
    list.sort((a,b)=>b.score-a.score||a.title.localeCompare(b.title,'es'));
    const top=list.slice(0,40);summary.textContent=top.length?`${top.length} resultado${top.length===1?'':'s'} para “${q}”`:`No hay resultados para “${q}”.`;
    results.innerHTML=top.length?top.map(x=>`<a class="search-result" href="${esc(x.url)}"><span class="search-type">${esc(x.type)}</span><div><b>${esc(x.title)}</b><small>${esc(x.text||'')}</small></div><span>→</span></a>`).join(''):'<div class="empty">Prueba con otro término: un circuito, simulador, categoría o producto.</div>';
  }
  form?.addEventListener('submit',e=>{e.preventDefault();const q=input.value.trim();const u=new URL(location.href);if(q)u.searchParams.set('q',q);else u.searchParams.delete('q');history.replaceState(null,'',u);run(q)});
  document.querySelectorAll('.search-shortcuts [data-q]').forEach(b=>b.addEventListener('click',()=>{input.value=b.dataset.q||'';form.requestSubmit()}));
  Promise.all([fetch('/api/news').then(r=>r.json()).catch(()=>({items:[]})),fetch('/api/events').then(r=>r.json()).catch(()=>({items:[]}))]).then(([nd,ed])=>{news=nd.items||[];events=ed.items||[];const q=new URLSearchParams(location.search).get('q')||'';if(q){input.value=q;run(q)}});
})();