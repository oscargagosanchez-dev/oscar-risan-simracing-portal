(()=>{
  const upsert=(selector,attrs)=>{let el=document.head.querySelector(selector);if(!el){el=document.createElement(attrs.tag||'meta');document.head.appendChild(el)}for(const [k,v] of Object.entries(attrs)){if(k!=='tag'&&v!=null)el.setAttribute(k,v)}return el};
  const clean=s=>String(s||'').replace(/\s+/g,' ').trim();
  const set=({title,description,image,type='website',url}={})=>{
    const pageTitle=clean(title||document.title||'Oscar Risan SimRacing');
    const desc=clean(description||document.querySelector('meta[name="description"]')?.content||'Noticias, eventos y actualidad de SimRacing en castellano.').slice(0,220);
    const canonical=url||location.href;
    document.title=pageTitle;
    upsert('meta[name="description"]',{name:'description',content:desc});
    upsert('link[rel="canonical"]',{tag:'link',rel:'canonical',href:canonical});
    upsert('meta[property="og:site_name"]',{property:'og:site_name',content:'Oscar Risan SimRacing'});
    upsert('meta[property="og:locale"]',{property:'og:locale',content:'es_ES'});
    upsert('meta[property="og:type"]',{property:'og:type',content:type});
    upsert('meta[property="og:title"]',{property:'og:title',content:pageTitle});
    upsert('meta[property="og:description"]',{property:'og:description',content:desc});
    upsert('meta[property="og:url"]',{property:'og:url',content:canonical});
    upsert('meta[name="twitter:card"]',{name:'twitter:card',content:image?'summary_large_image':'summary'});
    upsert('meta[name="twitter:title"]',{name:'twitter:title',content:pageTitle});
    upsert('meta[name="twitter:description"]',{name:'twitter:description',content:desc});
    if(image){upsert('meta[property="og:image"]',{property:'og:image',content:image});upsert('meta[name="twitter:image"]',{name:'twitter:image',content:image})}
  };
  window.OR_SEO={set};
})();