(()=>{
const KEY='or-timezone-mode';
const MADRID='Europe/Madrid';
const LOCAL=Intl.DateTimeFormat().resolvedOptions().timeZone||'UTC';
function stored(){try{return localStorage.getItem(KEY)||'madrid'}catch{return'madrid'}}
function zone(mode=stored()){return mode==='utc'?'UTC':mode==='local'?LOCAL:MADRID}
function label(mode=stored()){return mode==='utc'?'UTC':mode==='local'?`Hora local · ${LOCAL}`:'España · Europe/Madrid'}
function formatTime(iso,mode=stored()){const d=new Date(iso);if(Number.isNaN(d.getTime()))return'';return new Intl.DateTimeFormat('es-ES',{timeZone:zone(mode),hour:'2-digit',minute:'2-digit',hour12:false}).format(d)}
function formatDate(iso,mode=stored(),opts={weekday:'short',day:'numeric',month:'short'}){const d=new Date(iso);if(Number.isNaN(d.getTime()))return'';return new Intl.DateTimeFormat('es-ES',{timeZone:zone(mode),...opts}).format(d)}
function exactEvent(e){const p=String(e?.datePrecision||'').toLowerCase(),t=String(e?.timeLabel||e?.time||'').toLowerCase();return !['week','range'].includes(p)&&!/por confirmar|disponible durante|vigente|sin hora|horario.*confirmar/.test(t)}
function save(mode){try{localStorage.setItem(KEY,mode)}catch{}window.dispatchEvent(new CustomEvent('or-timezone-change',{detail:{mode}}))}
function selectorHTML(id='orTimezone'){const m=stored();return `<div class="tz-control"><label for="${id}">Horario</label><select id="${id}" aria-label="Zona horaria"><option value="madrid"${m==='madrid'?' selected':''}>España</option><option value="local"${m==='local'?' selected':''}>Hora local</option><option value="utc"${m==='utc'?' selected':''}>UTC</option></select><small class="tz-label">${label(m)}</small></div>`}
function bind(select,cb){if(!select)return;select.value=stored();const root=select.closest('.tz-control');const sync=()=>{select.value=stored();const l=root?.querySelector('.tz-label');if(l)l.textContent=label()};select.addEventListener('change',()=>{save(select.value);sync();cb?.()});window.addEventListener('or-timezone-change',()=>{sync();cb?.()})}
window.OR_TIMEZONE={stored,zone,label,formatTime,formatDate,exactEvent,save,selectorHTML,bind,LOCAL,MADRID};
function addStyle(){if(document.querySelector('link[href="/timezone.css"]'))return;const l=document.createElement('link');l.rel='stylesheet';l.href='/timezone.css';document.head.appendChild(l)}
addStyle();
})();
