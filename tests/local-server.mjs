import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const events={items:[
{id:'spa',sim:'Le Mans Ultimate',label:'LMU',title:'2.4 Horas · LMP2 ELMS + LMGT3',track:'Spa-Francorchamps',duration:'144 min',start:new Date(Date.now()+3600000).toISOString(),time:'18:00',tags:['ENDURANCE','MULTICLASE','OFICIAL'],type:'endurance',weather:'Seco',fuel:'x1',tyres:'x1',details:{format:'Multiclase',practice:'20 min',qualifying:'15 min',race:'144 min'}},
{id:'rr',sim:'RaceRoom',label:'R3E',title:'DTM Ranked Weekly',track:'Nürburgring Grand Prix',duration:'Weekly Race',start:new Date(Date.now()+7200000).toISOString(),time:'20:00',tags:['WEEKLY','RANKED','OFICIAL'],type:'special',weather:'',details:{format:'Weekly Race'}},
{id:'ace',sim:'Assetto Corsa EVO',label:'AC EVO',title:'GT3 Daily Race',track:'Monza',duration:'20 min carrera',start:new Date(Date.now()+10800000).toISOString(),time:'21:00',tags:['DAILY','RANKED','OFICIAL'],type:'sprint',weather:'Clear',details:{format:'Quick Race'}}
],updatedAt:new Date().toISOString()};
const yt={items:[]}; const news={items:[]};
const mime={'.html':'text/html; charset=utf-8','.js':'text/javascript; charset=utf-8','.json':'application/json','.webp':'image/webp'};
const server=http.createServer((req,res)=>{
 if(req.url==='/api/events'){res.setHeader('content-type','application/json');return res.end(JSON.stringify(events))}
 if(req.url==='/api/youtube'){res.setHeader('content-type','application/json');return res.end(JSON.stringify(yt))}
 if(req.url==='/api/news'){res.setHeader('content-type','application/json');return res.end(JSON.stringify(news))}
 if(req.url==='/api/health'){res.setHeader('content-type','application/json');return res.end(JSON.stringify({ok:true}))}
 const rel=req.url==='/'?'index.html':decodeURIComponent(req.url.split('?')[0]).replace(/^\//,'');
 const p=path.normalize(path.join(root,rel)); if(!p.startsWith(root)||!fs.existsSync(p)){res.statusCode=404;return res.end('not found')}
 res.setHeader('content-type',mime[path.extname(p)]||'application/octet-stream');fs.createReadStream(p).pipe(res)
});
server.listen(4173,'127.0.0.1',()=>console.log('http://127.0.0.1:4173'));
