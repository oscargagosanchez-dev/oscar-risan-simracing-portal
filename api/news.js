const SOURCES=[
  {appid:'2399420',category:'Le Mans Ultimate',label:'Le Mans Ultimate'},
  {appid:'211500',category:'RaceRoom',label:'RaceRoom'},
  {appid:'3058630',category:'Assetto Corsa EVO',label:'Assetto Corsa EVO'},
  {appid:'3917090',category:'Assetto Corsa Rally',label:'Assetto Corsa Rally'},
  {appid:'1066890',category:'Automobilista 2',label:'Automobilista 2'},
  {appid:'227300',category:'Euro Truck Simulator 2',label:'Euro Truck Simulator 2'},
  {appid:'270880',category:'American Truck Simulator',label:'American Truck Simulator'}
];
const UA={'user-agent':'Mozilla/5.0 OscarRisanPortal/2.0 (+https://oscar-risan-simracing-portal.vercel.app)'};
function decodeHtml(s=''){return String(s).replace(/&#(\d+);/g,(_,n)=>String.fromCodePoint(+n)).replace(/&#x([0-9a-f]+);/gi,(_,n)=>String.fromCodePoint(parseInt(n,16))).replace(/&nbsp;|&#160;/gi,' ').replace(/&amp;/gi,'&').replace(/&quot;/gi,'"').replace(/&#39;|&apos;/gi,"'").replace(/&ndash;|&mdash;/gi,'–').replace(/&hellip;/gi,'…')}
function cleanHtml(s=''){return decodeHtml(String(s).replace(/<script[\s\S]*?<\/script>/gi,' ').replace(/<style[\s\S]*?<\/style>/gi,' ').replace(/<[^>]+>/g,' ')).replace(/\s+/g,' ').trim()}
function imageFrom(html=''){const patterns=[/<img[^>]+src=["']([^"']+)["']/i,/<img[^>]+data-src=["']([^"']+)["']/i,/https?:\/\/[^\s"']+\.(?:jpg|jpeg|png|webp)(?:\?[^\s"']*)?/i];for(const p of patterns){const m=String(html).match(p);if(m)return decodeHtml(m[1]||m[0])}return ''}
function tagsFor(category,title=''){const t=(title||'').toLowerCase(),tags=[];if(/update|patch|hotfix|actualiz|version|v\d/.test(t))tags.push('ACTUALIZACIÓN');if(/dlc|content|car|cars|track|circuit|season|coche|circuito/.test(t))tags.push('CONTENIDO');if(/event|race|championship|competition|esports|racing/.test(t))tags.push('EVENTO');if(/hardware|wheel|pedal|base|cockpit|shifter|handbrake|moza|fanatec|simucube|asetek|conspit|sim-lab/.test(t))tags.push('HARDWARE');if(category==='RaceRoom')tags.push('R3E');if(category==='Le Mans Ultimate')tags.push('LMU');if(category==='Assetto Corsa EVO')tags.push('AC EVO');if(category==='Assetto Corsa Rally')tags.push('AC RALLY');return [...new Set(tags)].slice(0,3)}
function looksSpanish(s=''){const t=` ${String(s).toLowerCase()} `;const hits=[' el ',' la ',' los ',' las ',' de ',' del ',' para ',' con ',' una ',' un ',' y ',' nueva ',' actualización ',' carrera ',' circuito '].filter(w=>t.includes(w)).length;return hits>=3}
async function translateText(text=''){
 const original=String(text||'').trim();
 if(!original||looksSpanish(original))return original;
 try{
  const url=`https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl=es&dt=t&q=${encodeURIComponent(original.slice(0,900))}`;
  const r=await fetch(url,{headers:UA,signal:AbortSignal.timeout(4500)});
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
async function getText(url,timeout=7000){const r=await fetch(url,{headers:UA,signal:AbortSignal.timeout(timeout)});if(!r.ok)throw new Error(`${r.status} ${url}`);return await r.text()}
async function steamNews(src){try{const u=`https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid=${src.appid}&count=20&maxlength=1200&format=json`;const r=await fetch(u,{headers:UA,signal:AbortSignal.timeout(7000)});if(!r.ok)throw new Error('steam');const j=await r.json();return (j?.appnews?.newsitems||[]).map(n=>({
 id:`steam-${src.appid}-${n.gid}`,
 category:src.category,
 title:cleanHtml(n.title),
 summary:cleanHtml(n.contents).slice(0,520),
 source:src.label,
 sourceUrl:n.url||`https://store.steampowered.com/news/app/${src.appid}`,
 image:imageFrom(n.contents),
 date:new Date((n.date||0)*1000).toISOString(),
 tags:tagsFor(src.category,n.title),
 sourcePriority:3
})).filter(x=>x.title)}catch{return []}}
function categoryFromText(input=''){
 const t=cleanHtml(input).toLowerCase();
 if(/assetto corsa rally|ac rally/.test(t))return'Assetto Corsa Rally';
 if(/assetto corsa evo|ac evo/.test(t))return'Assetto Corsa EVO';
 if(/le mans ultimate|\blmu\b/.test(t))return'Le Mans Ultimate';
 if(/raceroom|race room|\br3e\b/.test(t))return'RaceRoom';
 if(/automobilista 2|\bams2\b/.test(t))return'Automobilista 2';
 if(/euro truck simulator 2|\bets2\b/.test(t))return'Euro Truck Simulator 2';
 if(/american truck simulator|\bats\b/.test(t))return'American Truck Simulator';
 if(/hardware|equipment|wheel|wheelbase|pedal|cockpit|shifter|handbrake|dashboard|simucube|fanatec|moza|asetek|sim-lab|heusinkveld|conspit|simagic|thrustmaster/.test(t))return'Hardware';
 return'SimRacing';
}
async function traxionNews(){
 try{
  const u='https://traxion.gg/wp-json/wp/v2/posts?per_page=40&_embed=1&_fields=id,date,link,title,excerpt,content,_embedded';
  const r=await fetch(u,{headers:UA,signal:AbortSignal.timeout(7500)});
  if(!r.ok)throw new Error('traxion-wp');
  const posts=await r.json();
  if(!Array.isArray(posts))throw new Error('traxion-shape');
  return posts.map(p=>{
   const title=cleanHtml(p?.title?.rendered||'');
   const excerpt=cleanHtml(p?.excerpt?.rendered||p?.content?.rendered||'').slice(0,520);
   const terms=(p?._embedded?.['wp:term']||[]).flat().map(x=>x?.name).filter(Boolean).join(' ');
   const category=categoryFromText(`${terms} ${title} ${excerpt}`);
   const media=p?._embedded?.['wp:featuredmedia']?.[0];
   const image=media?.source_url||media?.media_details?.sizes?.large?.source_url||imageFrom(p?.content?.rendered||'');
   return {id:`traxion-${p.id}`,category,title,summary:excerpt,source:'Traxion',sourceUrl:p.link||'https://traxion.gg/category/news/',image,date:p.date?new Date(p.date).toISOString():new Date().toISOString(),tags:tagsFor(category,title),sourcePriority:2};
  }).filter(x=>x.title&&x.sourceUrl);
 }catch{
  try{
   const xml=await getText('https://traxion.gg/feed/',7500);const out=[];const blocks=xml.match(/<item>[\s\S]*?<\/item>/gi)||[];
   for(const block of blocks.slice(0,40)){
    const grab=tag=>{const m=block.match(new RegExp(`<${tag}[^>]*>([\\s\\S]*?)<\\/${tag}>`,'i'));return m?cleanHtml(m[1].replace(/<!\[CDATA\[|\]\]>/g,'')):''};
    const title=grab('title'),summary=grab('description').slice(0,520),link=grab('link'),category=categoryFromText(`${grab('category')} ${title} ${summary}`);if(!title)continue;
    const dateRaw=grab('pubDate');out.push({id:`traxion-feed-${Buffer.from(link||title).toString('base64url').slice(0,32)}`,category,title,summary,source:'Traxion',sourceUrl:link||'https://traxion.gg/category/news/',image:imageFrom(block),date:dateRaw?new Date(dateRaw).toISOString():new Date().toISOString(),tags:tagsFor(category,title),sourcePriority:2});
   }
   return out;
  }catch{return []}
 }
}
function absoluteOvertake(href=''){if(!href)return'';if(/^https?:\/\//i.test(href))return href;return`https://www.overtake.gg${href.startsWith('/')?'':'/'}${href}`}
async function overtakeNews(){
 try{
  const html=await getText('https://www.overtake.gg/news/',8000),out=[],seen=new Set();
  const rx=/<a\b[^>]*href=["']([^"']*\/news\/[^"'#?]+\.[0-9]+\/?)["'][^>]*>([\s\S]*?)<\/a>/gi;let m;
  while((m=rx.exec(html))&&out.length<40){
   const url=absoluteOvertake(decodeHtml(m[1]));if(seen.has(url))continue;const title=cleanHtml(m[2]);if(title.length<20||/^(image|comments?|read more)$/i.test(title))continue;
   const start=Math.max(0,m.index-900),end=Math.min(html.length,rx.lastIndex+1200),context=html.slice(start,end);const category=categoryFromText(context+' '+title);
   const dateMatch=context.match(/<time[^>]+datetime=["']([^"']+)["']/i);const metaDesc=context.match(/(?:article-body|message-body|contentRow-snippet|articlePreview-description)[^>]*>([\s\S]{20,900}?)<\//i);let summary=cleanHtml(metaDesc?.[1]||'');if(!summary){const text=cleanHtml(context);const pos=text.toLowerCase().indexOf(title.toLowerCase());summary=(pos>=0?text.slice(pos+title.length):text).replace(/^(today|yesterday|monday|tuesday|wednesday|thursday|friday|saturday|sunday)[^A-Z]*/i,'').slice(0,520)}
   const image=imageFrom(context);seen.add(url);out.push({id:`overtake-${url.match(/\.(\d+)\/?$/)?.[1]||Buffer.from(url).toString('base64url').slice(0,24)}`,category,title,summary,source:'OverTake',sourceUrl:url,image,date:dateMatch?new Date(dateMatch[1]).toISOString():new Date().toISOString(),tags:tagsFor(category,title),sourcePriority:2});
  }
  return out;
 }catch{return []}
}
function normTitle(s=''){return cleanHtml(s).toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g,'').replace(/\b(the|a|an|and|or|of|for|to|in|on|with|de|del|la|el|los|las|un|una|y|para|con|en)\b/g,' ').replace(/[^a-z0-9]+/g,' ').replace(/\s+/g,' ').trim()}
function similarTitle(a,b){const A=new Set(normTitle(a).split(' ').filter(x=>x.length>2)),B=new Set(normTitle(b).split(' ').filter(x=>x.length>2));if(!A.size||!B.size)return false;let common=0;for(const x of A)if(B.has(x))common++;const score=common/Math.min(A.size,B.size);return score>=0.68}
function dedupe(items){const sorted=[...items].sort((a,b)=>new Date(b.date)-new Date(a.date)||((b.sourcePriority||0)-(a.sourcePriority||0))),out=[];for(const item of sorted){const duplicate=out.find(x=>x.category===item.category&&Math.abs(new Date(x.date)-new Date(item.date))<5*864e5&&similarTitle(x.title,item.title));if(!duplicate)out.push(item);else if((item.sourcePriority||0)>(duplicate.sourcePriority||0)){const i=out.indexOf(duplicate);out[i]=item}}return out}
const FALLBACK=[
 {id:'fallback-lmu-v14',category:'Le Mans Ultimate',title:'Le Mans Ultimate V1.4 y sus últimos hotfixes',summary:'Le Mans Ultimate ha recibido la actualización V1.4 y posteriores correcciones centradas en online, cambios de piloto, físicas, Race Watch y el calendario WEC 2026.',source:'Le Mans Ultimate / Steam',sourceUrl:'https://steamcommunity.com/app/2399420/announcements/',image:'',date:'2026-07-30T10:00:00.000Z',tags:['LMU','ACTUALIZACIÓN'],language:'es'},
 {id:'fallback-acevo-08',category:'Assetto Corsa EVO',title:'Assetto Corsa EVO Early Access 0.8 disponible',summary:'Kunos continúa ampliando Assetto Corsa EVO con la versión Early Access 0.8 y nuevas mejoras para el simulador.',source:'Assetto Corsa',sourceUrl:'https://assettocorsa.gg/assetto-corsa-evo/',image:'',date:'2026-07-08T10:00:00.000Z',tags:['AC EVO','ACTUALIZACIÓN'],language:'es'}
];
async function translateInBatches(items,batchSize=8){const out=[];for(let i=0;i<items.length;i+=batchSize){const batch=items.slice(i,i+batchSize);out.push(...await Promise.all(batch.map(translateItem)))}return out}
export default async function handler(req,res){
 res.setHeader('Cache-Control','s-maxage=600, stale-while-revalidate=1800');
 const settled=await Promise.all([...SOURCES.map(steamNews),traxionNews(),overtakeNews()]);
 const steam=settled.slice(0,SOURCES.length).flat(),traxion=settled[SOURCES.length]||[],overtake=settled[SOURCES.length+1]||[];
 let items=dedupe([...steam,...traxion,...overtake]).sort((a,b)=>new Date(b.date)-new Date(a.date)).slice(0,100);
 if(!items.length)return res.status(200).json({items:FALLBACK,updatedAt:new Date().toISOString(),sourceMode:'fallback',translated:true,sources:{steam:0,traxion:0,overtake:0}});
 items=await translateInBatches(items,8);
 return res.status(200).json({items,updatedAt:new Date().toISOString(),sourceMode:'live',translated:true,sources:{steam:steam.length,traxion:traxion.length,overtake:overtake.length}});
}
