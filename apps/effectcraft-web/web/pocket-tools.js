/* EffectCraft Pocket Tools */
(function () {
"use strict";
let overlay = null;
let currentDpi = 1050;

const items = [
["Keyboard","Open virtual keyboard","keyboard"],
["Plug-ins","WASM Effects · /plugins/","plugins"],
["Extensions","ScriptUI Panels · /extensions/","extensions"],
["AE (.aex) Bridge","After Effects C SDK bridge","bridge"],
["Import AE Project","AEP / AEPX / JSON","import"],
["Interface DPI","Application interface scale","dpi"],
["Play / Stop Preview","Preview controls","preview"],
["Render & Export","Render queue","render"]
];

function el(tag, attrs, value) {
 const n = document.createElement(tag);
 Object.entries(attrs || {}).forEach(([k,v]) => n.setAttribute(k,v));
 if (value !== undefined) n.textContent = value;
 return n;
}
function close() {
 if (overlay) overlay.remove();
 overlay = null;
}
function back(body) {
 const b = el("button",{class:"ecpt-button"},"Back");
 b.addEventListener("click",renderMenu);
 body.appendChild(b);
}
function message(title,msg) {
 const body = overlay.querySelector(".ecpt-body");
 body.replaceChildren();
 body.appendChild(el("div",{class:"ecpt-title"},title));
 body.appendChild(el("p",{class:"ecpt-note"},msg));
 back(body);
}
function applyDpi(dpi) {
 currentDpi = Math.max(420,Math.min(1600,Math.round(Number(dpi)||1050)));
 if (window.effectcraft && window.effectcraft.setInterfaceDpi)
  window.effectcraft.setInterfaceDpi(currentDpi);
 const label = overlay.querySelector("#ecpt-dpi-value");
 if (label) label.textContent = currentDpi+" DPI · "+Math.round(420/currentDpi*100)+"%";
}
function renderDpi(body) {
 body.replaceChildren();
 body.appendChild(el("div",{class:"ecpt-title"},"Interface DPI"));
 body.appendChild(el("p",{class:"ecpt-note",id:"ecpt-dpi-value"},
  currentDpi+" DPI · "+Math.round(420/currentDpi*100)+"%"));
 [[420,"Phone","100%"],[700,"Compact Laptop","60%"],
  [800,"Standard Laptop","52%"],[900,"Wide Laptop","47%"],
  [1050,"Desktop Pro","40%"]].forEach(([dpi,name,pct])=>{
   const b=el("button",{class:"ecpt-button"},dpi+" DPI — "+name+" ("+pct+")");
   b.addEventListener("click",()=>applyDpi(dpi));
   body.appendChild(b);
 });
 body.appendChild(el("div",{class:"ecpt-title"},"Custom DPI"));
 const slider=el("input",{type:"range",min:"420",max:"1600",step:"10",
  value:String(currentDpi),class:"ecpt-slider"});
 slider.addEventListener("input",()=>applyDpi(slider.value));
 body.appendChild(slider);
 back(body);
}
function renderMenu() {
 if (!overlay) return;
 const body=overlay.querySelector(".ecpt-body");
 body.replaceChildren();
 items.forEach(([title,desc,action])=>{
  const b=el("button",{class:"ecpt-item",type:"button"});
  b.append(el("span",{class:"ecpt-item-title"},title),
   el("span",{class:"ecpt-item-description"},desc));
  b.addEventListener("click",()=>{
   if (action==="keyboard") {
    if (window.toggleVirtualKeyboard) window.toggleVirtualKeyboard();
    close();
   } else if (action==="dpi") renderDpi(body);
   else {
    const msgs={
     plugins:"WASM loader exists in source. Android file-picker loading still needs testing.",
     extensions:"ScriptUI extension runtime is not implemented in this Android build.",
     bridge:"Native After Effects .aex C SDK bridge is not implemented in this build.",
     import:"AEP/AEPX project parsing is not implemented yet.",
     preview:"This shortcut is not connected yet. Use existing preview controls.",
     render:"This shortcut is not connected yet. Use existing render/export controls."
    };
    message(title,msgs[action]||"Not available yet.");
   }
  });
  body.appendChild(b);
 });
}
function styles() {
 if (document.getElementById("ecpt-styles")) return;
 const style=el("style",{id:"ecpt-styles"});
 style.textContent=`
 #ecpt-overlay{position:fixed;inset:0;z-index:1000001;background:rgba(0,0,0,.35);display:flex;align-items:flex-end;justify-content:flex-end;padding:82px 18px 18px;box-sizing:border-box;font-family:system-ui,sans-serif}
 #ecpt-panel{width:min(340px,calc(100vw - 36px));max-height:calc(100vh - 110px);overflow:auto;box-sizing:border-box;color:#e5e7eb;background:#202127;border:1px solid #454752;border-radius:13px;padding:12px;box-shadow:0 12px 38px rgba(0,0,0,.55)}
 .ecpt-header{display:flex;align-items:center;justify-content:space-between;padding:2px 2px 10px;border-bottom:1px solid #3a3c45;margin-bottom:8px}
 .ecpt-heading{font-size:16px;font-weight:700}
 .ecpt-close{background:transparent;border:0;color:#ddd;font-size:24px}
 .ecpt-body{display:flex;flex-direction:column;gap:6px}
 .ecpt-item,.ecpt-button{width:100%;box-sizing:border-box;text-align:left;color:#e5e7eb;background:#2a2d35;border:1px solid #41444e;border-radius:8px;padding:10px 11px;font:inherit}
 .ecpt-item-title{display:block;font-size:13px;font-weight:650}
 .ecpt-item-description{display:block;color:#a8adba;font-size:10px;margin-top:3px}
 .ecpt-title{font-size:14px;font-weight:700;margin:7px 2px}
 .ecpt-note{font-size:12px;line-height:1.5;color:#b9bfcc}
 .ecpt-slider{width:100%;margin:8px 0 14px}`;
 document.head.appendChild(style);
}
window.openPocketTools=function(){
 if (overlay){close();return;}
 styles();
 currentDpi=window.effectcraft&&window.effectcraft.getInterfaceDpi
  ?window.effectcraft.getInterfaceDpi():1050;
 overlay=el("div",{id:"ecpt-overlay"});
 const panel=el("div",{id:"ecpt-panel",role:"dialog","aria-label":"Pocket Tools"});
 const header=el("div",{class:"ecpt-header"});
 header.append(el("span",{class:"ecpt-heading"},"Pocket Tools"),
  el("button",{class:"ecpt-close",type:"button"},"×"));
 header.querySelector("button").addEventListener("click",close);
 const body=el("div",{class:"ecpt-body"});
 panel.append(header,body);
 overlay.appendChild(panel);
 overlay.addEventListener("click",e=>{if(e.target===overlay)close();});
 document.body.appendChild(overlay);
 renderMenu();
};
})();
