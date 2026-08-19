const EVENTS_URL='https://simracing-pro.com/es/category/events/';
const FALLBACK=[
  {title:'Sim Gaming Expo Chicago 2026: Meet SRP® SimRacing Pro',date:'16 jun 2026',url:'https://simracing-pro.com/es/category/events/'},
  {title:'SRP® At Sim Gaming Expo',date:'28 may 2026',url:'https://simracing-pro.com/es/category/events/'},
  {title:'GOD LEAGUE SRP RD2: Guía de Hungaroring y horarios',date:'16 may 2026',url:'https://simracing-pro.com/es/category/events/'}
];
function clean(s=''){return String(s).replace(/<[^>]+>/g,' ').replace(/&nbsp;/g,' ').replace(/&amp;/g,'&').replace(/&#8211;|&ndash;/g,'–').replace(/&#8212;|&mdash;/g,'—').replace(/\s+/g,' ').trim()}
function abs(u=''){try{return new URL(u,EVENTS_URL).toString()}catch{return EVENTS_URL}}
function parse(html=''){
  const blocks=String(html).match(/<article[\s\S]*?<\/article>/gi)||[];
  const out=[];
  for(const b of blocks){
    const h=b.match(/<h[1-4][^>]*>[\s\S]*?<a[^>]+href=["']([^"']+)["'][^>]*>([\s\S]*?)<\/a>[\s\S]*?<\/h[1-4]>/i)||b.match(/<a[^>]+href=["']([^"']+)["'][^>]*>([\s\S]*?)<\/a>/i);
    if(!h)continue;
    const title=clean(h[2]); if(title.length<8)continue;
    const dm=b.match(/(?:<time[^>]*>|class=["'][^"']*(?:date|posted-on)[^"']*["'][^>]*>)([\s\S]*?)(?:<\/time>|<\/[^>]+>)/i);
    out.push({title,date:clean(dm?.[1]||''),url:abs(h[1])});
    if(out.length>=4)break;
  }
  return out;
}
export default async function handler(req,res){
  res.setHeader('Cache-Control','s-maxage=1800, stale-while-revalidate=7200');
  let items=[];
  try{
    const r=await fetch(EVENTS_URL,{headers:{'user-agent':'Mozilla/5.0 OscarRisanPortal/1.0'},signal:AbortSignal.timeout(7000)});
    if(r.ok)items=parse(await r.text());
  }catch{}
  if(!items.length)items=FALLBACK;
  res.status(200).json({items:items.slice(0,3),source:'SRP / SimRacing-Pro',updatedAt:new Date().toISOString(),mode:items===FALLBACK?'fallback':'live'});
}
