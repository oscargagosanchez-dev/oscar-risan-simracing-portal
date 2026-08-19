const SOURCES=[
  {appid:'2399420',category:'Le Mans Ultimate',label:'Le Mans Ultimate'},
  {appid:'211500',category:'RaceRoom',label:'RaceRoom'},
  {appid:'3058630',category:'Assetto Corsa EVO',label:'Assetto Corsa EVO'},
  {appid:'1066890',category:'Automobilista 2',label:'Automobilista 2'},
  {appid:'227300',category:'Euro Truck Simulator 2',label:'Euro Truck Simulator 2'},
  {appid:'270880',category:'American Truck Simulator',label:'American Truck Simulator'}
];
function cleanHtml(s=''){return String(s).replace(/<script[\s\S]*?<\/script>/gi,' ').replace(/<style[\s\S]*?<\/style>/gi,' ').replace(/<[^>]+>/g,' ').replace(/&nbsp;/g,' ').replace(/&amp;/g,'&').replace(/&quot;/g,'"').replace(/&#39;/g,"'").replace(/\s+/g,' ').trim()}
function imageFrom(html=''){const patterns=[/<img[^>]+src=["']([^"']+)["']/i,/https?:\/\/[^\s"']+\.(?:jpg|jpeg|png|webp)/i];for(const p of patterns){const m=String(html).match(p);if(m)return m[1]||m[0]}return ''}
function tagsFor(category,title=''){const t=(title||'').toLowerCase(),tags=[];if(/update|patch|hotfix|actualiz/.test(t))tags.push('ACTUALIZACIÓN');if(/dlc|content|car|track|circuit|season/.test(t))tags.push('CONTENIDO');if(/event|race|championship|competition/.test(t))tags.push('EVENTO');if(category==='RaceRoom')tags.push('R3E');if(category==='Le Mans Ultimate')tags.push('LMU');if(category==='Assetto Corsa EVO')tags.push('AC EVO');return [...new Set(tags)].slice(0,3)}
function looksSpanish(s=''){const t=` ${String(s).toLowerCase()} `;const hits=[' el ',' la ',' los ',' las ',' de ',' del ',' para ',' con ',' una ',' un ',' y ',' nueva ',' actualización ',' carrera ',' circuito '].filter(w=>t.includes(w)).length;return hits>=3}
async function translateText(text=''){
 const original=String(text||'').trim();
 if(!original||looksSpanish(original))return original;
 try{
  const url=`https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl=es&dt=t&q=${encodeURIComponent(original.slice(0,900))}`;
  const r=await fetch(url,{headers:{'user-agent':'Mozilla/5.0 OscarRisanPortal/1.0'},signal:AbortSignal.timeout(4500)});
  if(!r.ok)throw new Error('translate');
  const j=await r.json();
  const out=(Array.isArray(j?.[0])?j[0].map(x=>Array.isArray(x)?x[0]:'').join(''):'').trim();
  return out||original;
 }catch{return original}
}
async function translateItem(item){
 const originalTitle=item.title,originalSummary=item.summary;
 const [title,summary]=await Promise.all([translateText(originalTitle),translateText(originalSummary)]);
 return {...item,title,summary,originalTitle,originalSummary,language:'es'};
}
async function steamNews(src){try{const u=`https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid=${src.appid}&count=8&maxlength=1200&format=json`;const r=await fetch(u,{headers:{'user-agent':'Mozilla/5.0 OscarRisanPortal/1.0'},signal:AbortSignal.timeout(7000)});if(!r.ok)throw new Error('steam');const j=await r.json();return (j?.appnews?.newsitems||[]).map(n=>({
 id:`steam-${src.appid}-${n.gid}`,
 category:src.category,
 title:cleanHtml(n.title),
 summary:cleanHtml(n.contents).slice(0,520),
 source:src.label,
 sourceUrl:n.url||`https://store.steampowered.com/news/app/${src.appid}`,
 image:imageFrom(n.contents),
 date:new Date((n.date||0)*1000).toISOString(),
 tags:tagsFor(src.category,n.title)
})).filter(x=>x.title)}catch{return []}}
const FALLBACK=[
 {id:'fallback-lmu-v14',category:'Le Mans Ultimate',title:'Le Mans Ultimate V1.4 y sus últimos hotfixes',summary:'Le Mans Ultimate ha recibido la actualización V1.4 y posteriores correcciones centradas en online, cambios de piloto, físicas, Race Watch y el calendario WEC 2026.',source:'Le Mans Ultimate / Steam',sourceUrl:'https://steamcommunity.com/app/2399420/announcements/',image:'',date:'2026-07-30T10:00:00.000Z',tags:['LMU','ACTUALIZACIÓN'],language:'es'},
 {id:'fallback-acevo-08',category:'Assetto Corsa EVO',title:'Assetto Corsa EVO Early Access 0.8 disponible',summary:'Kunos continúa ampliando Assetto Corsa EVO con la versión Early Access 0.8 y nuevas mejoras para el simulador.',source:'Assetto Corsa',sourceUrl:'https://assettocorsa.gg/assetto-corsa-evo/',image:'',date:'2026-07-08T10:00:00.000Z',tags:['AC EVO','ACTUALIZACIÓN'],language:'es'}
];
async function translateInBatches(items,batchSize=6){const out=[];for(let i=0;i<items.length;i+=batchSize){const batch=items.slice(i,i+batchSize);out.push(...await Promise.all(batch.map(translateItem)))}return out}
export default async function handler(req,res){
 res.setHeader('Cache-Control','s-maxage=900, stale-while-revalidate=3600');
 const settled=await Promise.all(SOURCES.map(steamNews));
 let items=settled.flat().sort((a,b)=>new Date(b.date)-new Date(a.date)).slice(0,36);
 if(!items.length)return res.status(200).json({items:FALLBACK,updatedAt:new Date().toISOString(),sourceMode:'fallback',translated:true});
 items=await translateInBatches(items,6);
 return res.status(200).json({items,updatedAt:new Date().toISOString(),sourceMode:'live',translated:true});
}
