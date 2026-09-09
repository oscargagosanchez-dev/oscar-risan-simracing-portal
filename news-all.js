(()=>{
  function install(){
    if(typeof render!=='function'||typeof card!=='function'||typeof ok!=='function')return false;
    render=function(c='Todas'){
      const x=all.filter(n=>ok(n,c));
      document.querySelector('#newsTitle').textContent=c==='Todas'?'Últimas noticias':c;
      document.querySelector('#news').innerHTML=x.map(card).join('')||'<div class="empty">No hay noticias disponibles de esta categoría ahora mismo.</div>';
    };
    render(document.querySelector('.catnav button.active')?.dataset.c||'Todas');
    return true;
  }
  if(install())return;
  let tries=0;const t=setInterval(()=>{if(install()||++tries>30)clearInterval(t)},100);
})();
