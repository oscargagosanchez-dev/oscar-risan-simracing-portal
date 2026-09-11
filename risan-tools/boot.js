(()=>{
const ROOT=location.pathname.includes('/apps/')?'../':'';
const ungzip=async s=>{const b=Uint8Array.from(atob(s),c=>c.charCodeAt(0));const ds=new DecompressionStream('gzip');return await new Response(new Blob([b]).stream().pipeThrough(ds)).text();};
const text=async p=>{const r=await fetch(ROOT+p); if(!r.ok) throw new Error(p+' '+r.status); return r.text();};
const join=async a=>(await Promise.all(a.map(x=>text('data/'+x)))).join('');
(async()=>{
const P=await fetch(ROOT+'data/pages.json').then(r=>r.json());
const S=await fetch(ROOT+'data/small-assets.json').then(r=>r.json());
const page=document.documentElement.dataset.page||'main';
let h=await ungzip(P[page]);
const css=await ungzip(P._style); h=h.replace('__STYLE__','<style>'+css+'</style>');
if(h.includes('__TEMPLATE__')){const t=await ungzip(P._template); h=h.replace('__TEMPLATE__','<script>'+t.replace(/<\/script/gi,'<\\/script')+'<\/script>');}
const assets={'race-alerts-config.webp':'data:image/webp;base64,'+S['race-alerts-config.webp'],'team-radio-config.webp':'data:image/webp;base64,'+S['team-radio-config.webp'],'race-alerts-discord.webp':'data:image/webp;base64,'+S['race-alerts-discord.webp'],'team-radio-v471.webp':'data:image/webp;base64,'+S['team-radio-v471.webp'],'lmu-hub-preview-laguna.webp':'data:image/webp;base64,'+await join(["hub-laguna-00.txt", "hub-laguna-01.txt", "hub-laguna-02.txt", "hub-laguna-03.txt", "hub-laguna-04.txt"]),'lmu-hub-preview-sebring.webp':'data:image/webp;base64,'+await join(["hub-sebring-00.txt", "hub-sebring-01.txt", "hub-sebring-02.txt", "hub-sebring-03.txt", "hub-sebring-04.txt"])};
for(const [n,u] of Object.entries(assets)){h=h.split('assets/'+n).join(u);h=h.split('../assets/'+n).join(u);}
const rz=await text('data/results.txt'); h=h.split('downloads/LMU_Discord_Bot_TEAM_v2.4_FINAL_PUBLIC.zip').join('data:application/zip;base64,'+rz);h=h.split('../downloads/LMU_Discord_Bot_TEAM_v2.4_FINAL_PUBLIC.zip').join('data:application/zip;base64,'+rz);
h=h.split('downloads/INSTRUCCIONES_LMU_DISCORD_Bot_TEAM_v2.4_PUBLIC.txt').join(ROOT+'guide.txt');h=h.split('../downloads/INSTRUCCIONES_LMU_DISCORD_Bot_TEAM_v2.4_PUBLIC.txt').join(ROOT+'guide.txt');
h=h.split('downloads/INSTRUCCIONES_LMU_DISCORD_BOT_TEAM_v2.4_PUBLIC.txt').join(ROOT+'guide.txt');h=h.split('../downloads/INSTRUCCIONES_LMU_DISCORD_BOT_TEAM_v2.4_PUBLIC.txt').join(ROOT+'guide.txt');
document.open();document.write(h);document.close();
})().catch(e=>document.body.innerHTML='<pre style="color:white;background:#070a0f;padding:20px">Error al cargar Risan Simracing Tools: '+e+'</pre>');
})();