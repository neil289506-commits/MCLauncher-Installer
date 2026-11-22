const L=[["Minecraft Launcher(官方)", 1, 0], ["PrismLauncher", 1, 1], ["Modrinth App", 1, 0], ["CurseForge", 1, 0], ["XMCL", 1, 1], ["ATLauncher", 1, 1], ["GDLauncher", 1, 0], ["LabyMod", 1, 0], ["Lunar Client", 1, 0], ["Emerald Launcher (LCE)", 1, 0], ["MultiMC", 0, 1]];
let sel=0,ver='安裝版',way='idm',tm=null;
const kindBox=document.getElementById('kind');
function reset(){clearInterval(tm);pg.style.width='0';[...seg.children].forEach(s=>s.style.background='#0006');ms.textContent='已選擇:'+L[sel][0]+'('+ver+')'}
function kinds(){const [,a,b]=L[sel];const opts=[['安裝版',a],['可攜版',b]].filter(o=>o[1]).map(o=>o[0]);ver=opts[0];kindBox.innerHTML='';
 opts.forEach((k,i)=>{const s=document.createElement('button');s.className='tab'+(i?'':' on');s.textContent=k;s.onclick=()=>{ver=k;[...kindBox.children].forEach(c=>c.classList.toggle('on',c===s));reset()};kindBox.append(s)});reset()}
L.forEach((x,i)=>{const d=document.createElement('div');d.className='pick'+(i?'':' on');d.textContent=x[0];d.onclick=()=>{sel=i;[...ll.children].forEach((c,j)=>c.classList.toggle('on',j===i));kinds()};ll.append(d)});
document.querySelectorAll('[data-w]').forEach(b=>b.onclick=()=>{way=b.dataset.w;document.querySelectorAll('[data-w]').forEach(x=>x.classList.toggle('on',x===b))});
seg.innerHTML=Array(8).fill('<div style="height:8px;border-radius:4px;background:#0006"></div>').join('');
dl.onclick=()=>{reset();let p=0;ms.textContent=way==='idm'?'交給 IDM:/d 網址 /p 資料夾 /f 檔名 /n':'curl -sIL 取得大小 → 8 段 HTTP Range 平行下載…';
 tm=setInterval(()=>{p+=2+Math.random()*4;if(p>=100){p=100;clearInterval(tm);ms.textContent='✔ 下載完成:'+L[sel][0]+'('+ver+')'}
  pg.style.width=p+'%';if(way==='curl')[...seg.children].forEach((s,i)=>{s.style.background=p>(i+1)*12.5-6?'var(--a)':'#0006'})},120)};
kinds();
