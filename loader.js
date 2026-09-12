(()=>{
const ROOT=location.pathname.includes('/apps/')?'../':'';
const PARTS=['data/v16-00.txt','data/v16-01.txt','data/v16-02.txt','data/v16-03.txt'];
const getText=async p=>{const r=await fetch(ROOT+p);if(!r.ok)throw new Error(p+' '+r.status);return r.text();};
const ungzip=async s=>{const b=Uint8Array.from(atob(s),c=>c.charCodeAt(0));const ds=new DecompressionStream('gzip');return new Response(new Blob([b]).stream().pipeThrough(ds)).text();};
const b64blob=(s,type)=>{const b=Uint8Array.from(atob(s),c=>c.charCodeAt(0));return URL.createObjectURL(new Blob([b],{type}));};
(async()=>{
 const packed=(await Promise.all(PARTS.map(getText))).join('');
 const B=JSON.parse(await ungzip(packed));
 let page=document.documentElement.dataset.page||'index.html';
 if(page==='home.html') page='index.html';
 let h=B.texts[page];
 if(!h) throw new Error('Página no incluida: '+page);
 const css='<style>'+B.texts['styles.css']+'</style>';
 h=h.replace('<link rel="stylesheet" href="../styles.css">',css).replace('<link rel="stylesheet" href="styles.css">',css);
 if(h.includes('<script src="../app-template.js"></script>')){
   const js=B.texts['app-template.js'].replace(/<\/script/gi,'<\\/script');
   h=h.replace('<script src="../app-template.js"></script>','<script>'+js+'<\/script>');
 }
 for(const [name,b64] of Object.entries(B.images)){
   const u=b64blob(b64,'image/webp');
   h=h.split('../assets/'+name).join(u);
   h=h.split('assets/'+name).join(u);
 }
 const zipUrl=b64blob(B.zip,'application/zip');
 h=h.split('../downloads/LMU_Discord_Bot_TEAM_v2.4_FINAL_PUBLIC.zip').join(zipUrl);
 h=h.split('downloads/LMU_Discord_Bot_TEAM_v2.4_FINAL_PUBLIC.zip').join(zipUrl);
 const guideUrl=URL.createObjectURL(new Blob([B.texts['guide.txt']],{type:'text/plain;charset=utf-8'}));
 for(const n of ['INSTRUCCIONES_LMU_DISCORD_BOT_TEAM_v2.4_PUBLIC.txt','INSTRUCCIONES_LMU_DISCORD_Bot_TEAM_v2.4_PUBLIC.txt']){
   h=h.split('../downloads/'+n).join(guideUrl);
   h=h.split('downloads/'+n).join(guideUrl);
 }
 document.open();document.write(h);document.close();
})().catch(e=>{document.body.innerHTML='<pre style="white-space:pre-wrap;color:#fff;background:#070a0f;padding:24px;font:14px/1.5 monospace">Error al cargar Risan Simracing Tools: '+String(e)+'</pre>';});
})();