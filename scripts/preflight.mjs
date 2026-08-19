import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const required=['index.html','article.html','styles.css','app.js','vercel.json','api/events.js','api/news.js','api/youtube.js','api/health.js','img-spa-lmp2-lmgt3.js','img-spa-wet-multiclass.js','img-lemans-hypercar-lmgt3.js','img-monza-gt3.js','img-nurburgring-dtm.js','img-hockenheim-super-touring.js','img-silverstone-roadster.js','img-suzuka-gt.js'];
let failed=false;
for(const f of required){const p=path.join(root,f);if(!fs.existsSync(p)){console.error('MISSING',f);failed=true}else if(fs.statSync(p).size===0){console.error('EMPTY',f);failed=true}}
for(const f of ['api/events.js','api/news.js','api/youtube.js','api/health.js']){try{execFileSync(process.execPath,['--check',path.join(root,f)],{stdio:'pipe'})}catch(e){console.error('SYNTAX',f,String(e.stderr||e.message));failed=true}}
try{JSON.parse(fs.readFileSync(path.join(root,'vercel.json'),'utf8'))}catch(e){console.error('INVALID vercel.json',e.message);failed=true}
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');const app=fs.readFileSync(path.join(root,'app.js'),'utf8');const css=fs.readFileSync(path.join(root,'styles.css'),'utf8');
for(const needle of ['/api/events','/api/youtube','/api/news','function eventVisual','openEventDetails']){if(!app.includes(needle)){console.error('APP missing',needle);failed=true}}
for(const needle of ['/styles.css','/app.js','img-spa-lmp2-lmgt3.js','mobile-bottom','viewport-fit=cover']){if(!html.includes(needle)){console.error('HTML missing',needle);failed=true}}
for(const needle of ['@media(max-width:620px)','mobile-bottom','safe-area-inset-bottom']){if(!css.includes(needle)){console.error('CSS missing',needle);failed=true}}
console.log(failed?'PREFLIGHT FAILED':'PREFLIGHT OK');
process.exit(failed?1:0);
