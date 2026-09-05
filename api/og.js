import fs from 'node:fs';
import path from 'node:path';
export default function handler(req,res){
  try{
    const p=path.join(process.cwd(),'img-monza-gt3.js');
    const src=fs.readFileSync(p,'utf8');
    const m=src.match(/base64,([^']+)/);
    if(!m)throw new Error('image');
    const buf=Buffer.from(m[1],'base64');
    res.setHeader('Content-Type','image/webp');
    res.setHeader('Cache-Control','public, max-age=86400, s-maxage=86400');
    return res.status(200).send(buf);
  }catch{
    return res.status(404).end();
  }
}
