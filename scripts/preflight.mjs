import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const required=['index.html','article.html','donde-correr.html','simulador.html','buscar.html','styles.css','mobile.css','news-editorial.css','srp.css','osp.css','portal-next.css','hoy.css','seo.js','robots.txt','sitemap.xml','app.js','news-format.js','srp.js','portal-next.js','hoy.js','donde-correr.js','simulador.js','buscar.js','vercel.json','api/events.js','api/news.js','api/srp.js','api/youtube.js','api/health.js','api/og.js','img-spa-lmp2-lmgt3.js','img-spa-wet-multiclass.js','img-lemans-hypercar-lmgt3.js','img-monza-gt3.js','img-nurburgring-dtm.js','img-hockenheim-super-touring.js','img-silverstone-roadster.js','img-suzuka-gt.js'];
let failed=false;
for(const f of required){const p=path.join(root,f);if(!fs.existsSync(p)){console.error('MISSING',f);failed=true}else if(fs.statSync(p).size===0){console.error('EMPTY',f);failed=true}}
for(const f of ['api/events.js','api/news.js','api/srp.js','api/youtube.js','api/health.js','api/og.js','srp.js','news-format.js','portal-next.js','hoy.js','donde-correr.js','simulador.js','buscar.js','seo.js']){try{execFileSync(process.execPath,['--check',path.join(root,f)],{stdio:'pipe'})}catch(e){console.error('SYNTAX',f,String(e.stderr||e.message));failed=true}}
try{JSON.parse(fs.readFileSync(path.join(root,'vercel.json'),'utf8'))}catch(e){console.error('INVALID vercel.json',e.message);failed=true}
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');const app=fs.readFileSync(path.join(root,'app.js'),'utf8');const css=fs.readFileSync(path.join(root,'styles.css'),'utf8');const srp=fs.readFileSync(path.join(root,'srp.css'),'utf8');const osp=fs.readFileSync(path.join(root,'osp.css'),'utf8');const next=fs.readFileSync(path.join(root,'portal-next.css'),'utf8');const srpjs=fs.readFileSync(path.join(root,'srp.js'),'utf8');const pnext=fs.readFileSync(path.join(root,'portal-next.js'),'utf8');const today=fs.readFileSync(path.join(root,'hoy.js'),'utf8');const agenda=fs.readFileSync(path.join(root,'donde-correr.js'),'utf8');const search=fs.readFileSync(path.join(root,'buscar.js'),'utf8');const seo=fs.readFileSync(path.join(root,'seo.js'),'utf8');
for(const needle of ['/api/events','/api/youtube','/api/news','function eventVisual','openEventDetails']){if(!app.includes(needle)){console.error('APP missing',needle);failed=true}}
for(const needle of ['/styles.css','/app.js','img-spa-lmp2-lmgt3.js','mobile-bottom','viewport-fit=cover','id="srp"','/srp.css','/srp.js','id="osp"','/osp.css']){if(!html.includes(needle)){console.error('HTML missing',needle);failed=true}}
for(const needle of ['@media(max-width:620px)','mobile-bottom','safe-area-inset-bottom']){if(!css.includes(needle)){console.error('CSS missing',needle);failed=true}}
for(const needle of ['.srp-section','.srp-products','.srp-news-item','@media(max-width:620px)']){if(!srp.includes(needle)){console.error('SRP CSS missing',needle);failed=true}}
for(const needle of ['.osp-section','.osp-products','.osp-resource','@media(max-width:620px)']){if(!osp.includes(needle)){console.error('OSP CSS missing',needle);failed=true}}
for(const needle of ['.featured-news','.sim-hub-grid','.racing-section','.page-grid','.fav-panel','.global-search','.calendar-actions']){if(!next.includes(needle)){console.error('NEXT CSS missing',needle);failed=true}}
if(!srpjs.includes('/portal-next.js')){console.error('SRP loader missing portal-next.js');failed=true}
for(const needle of ['or-favorite-sims-v1','/buscar.html','favoritePanel','og:image','application/ld+json','/hoy.js']){if(!pnext.includes(needle)){console.error('Portal next missing',needle);failed=true}}
for(const needle of ['queCorrerHoy','or-favorite-sims-v1','or-agenda-timezone-v2','/api/events']){if(!today.includes(needle)){console.error('Today module missing',needle);failed=true}}
for(const needle of ['Google Calendar','data-ics','or-agenda-timezone-v2']){if(!agenda.includes(needle)){console.error('Agenda missing',needle);failed=true}}
for(const needle of ['/api/news','/api/events','globalSearchInput']){if(!search.includes(needle)){console.error('Search missing',needle);failed=true}}
for(const needle of ['og:title','twitter:card','canonical']){if(!seo.includes(needle)){console.error('SEO helper missing',needle);failed=true}}
for(const [file,needles] of [['article.html',['og:type','twitter:card','/seo.js']],['donde-correr.html',['id="fullEvents"','id="agendaTz"','og:image','/donde-correr.js']],['simulador.html',['id="simEvents"','id="simNews"','og:image','/seo.js','/simulador.js']],['buscar.html',['id="globalSearchInput"','id="searchResults"','og:image','/buscar.js']]]){const txt=fs.readFileSync(path.join(root,file),'utf8');for(const needle of needles){if(!txt.includes(needle)){console.error(file,'missing',needle);failed=true}}}
if(!fs.readFileSync(path.join(root,'robots.txt'),'utf8').includes('sitemap.xml')){console.error('robots.txt missing sitemap');failed=true}
if(!fs.readFileSync(path.join(root,'sitemap.xml'),'utf8').includes('<urlset')){console.error('sitemap.xml invalid');failed=true}
console.log(failed?'PREFLIGHT FAILED':'PREFLIGHT OK');
process.exit(failed?1:0);
