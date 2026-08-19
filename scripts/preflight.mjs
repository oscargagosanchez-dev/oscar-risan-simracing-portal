import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const required=['index.html','article.html','donde-correr.html','simulador.html','styles.css','mobile.css','news-editorial.css','srp.css','osp.css','portal-next.css','timezone.css','app.js','news-format.js','srp.js','portal-next.js','timezone.js','timezone-main.js','donde-correr.js','simulador.js','vercel.json','api/events.js','api/news.js','api/srp.js','api/youtube.js','api/health.js','img-spa-lmp2-lmgt3.js','img-spa-wet-multiclass.js','img-lemans-hypercar-lmgt3.js','img-monza-gt3.js','img-nurburgring-dtm.js','img-hockenheim-super-touring.js','img-silverstone-roadster.js','img-suzuka-gt.js'];
let failed=false;
for(const f of required){const p=path.join(root,f);if(!fs.existsSync(p)){console.error('MISSING',f);failed=true}else if(fs.statSync(p).size===0){console.error('EMPTY',f);failed=true}}
for(const f of ['api/events.js','api/news.js','api/srp.js','api/youtube.js','api/health.js','srp.js','news-format.js','portal-next.js','timezone.js','timezone-main.js','donde-correr.js','simulador.js']){try{execFileSync(process.execPath,['--check',path.join(root,f)],{stdio:'pipe'})}catch(e){console.error('SYNTAX',f,String(e.stderr||e.message));failed=true}}
try{JSON.parse(fs.readFileSync(path.join(root,'vercel.json'),'utf8'))}catch(e){console.error('INVALID vercel.json',e.message);failed=true}
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');const app=fs.readFileSync(path.join(root,'app.js'),'utf8');const css=fs.readFileSync(path.join(root,'styles.css'),'utf8');const srp=fs.readFileSync(path.join(root,'srp.css'),'utf8');const osp=fs.readFileSync(path.join(root,'osp.css'),'utf8');const next=fs.readFileSync(path.join(root,'portal-next.css'),'utf8');const srpjs=fs.readFileSync(path.join(root,'srp.js'),'utf8');const tz=fs.readFileSync(path.join(root,'timezone.js'),'utf8');
for(const needle of ['/api/events','/api/youtube','/api/news','function eventVisual','openEventDetails']){if(!app.includes(needle)){console.error('APP missing',needle);failed=true}}
for(const needle of ['/styles.css','/app.js','img-spa-lmp2-lmgt3.js','mobile-bottom','viewport-fit=cover','id="srp"','/srp.css','/srp.js','id="osp"','/osp.css']){if(!html.includes(needle)){console.error('HTML missing',needle);failed=true}}
for(const needle of ['@media(max-width:620px)','mobile-bottom','safe-area-inset-bottom']){if(!css.includes(needle)){console.error('CSS missing',needle);failed=true}}
for(const needle of ['.srp-section','.srp-products','.srp-news-item','@media(max-width:620px)']){if(!srp.includes(needle)){console.error('SRP CSS missing',needle);failed=true}}
for(const needle of ['.osp-section','.osp-products','.osp-resource','@media(max-width:620px)']){if(!osp.includes(needle)){console.error('OSP CSS missing',needle);failed=true}}
for(const needle of ['.featured-news','.sim-hub-grid','.racing-section','.page-grid']){if(!next.includes(needle)){console.error('NEXT CSS missing',needle);failed=true}}
for(const needle of ['/portal-next.js','/timezone.js','/timezone-main.js']){if(!srpjs.includes(needle)){console.error('SRP loader missing',needle);failed=true}}
for(const needle of ['Europe/Madrid','Hora local','localStorage','OR_TIMEZONE']){if(!tz.includes(needle)){console.error('Timezone module missing',needle);failed=true}}
for(const [file,needles] of [['donde-correr.html',['id="fullEvents"','id="agendaTimezone"','/timezone.js','/donde-correr.js']],['simulador.html',['id="simEvents"','id="simNews"','/simulador.js']]]){const txt=fs.readFileSync(path.join(root,file),'utf8');for(const needle of needles){if(!txt.includes(needle)){console.error(file,'missing',needle);failed=true}}}
console.log(failed?'PREFLIGHT FAILED':'PREFLIGHT OK');
process.exit(failed?1:0);
