// Paquete de iconas na app real: pestana debaixo de Anotar, cartafol novo con todo o que se pide e orixinal intacto.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");

export default {
  about: "Iconas: pestana, cartafol novo con ICO, ICNS, web e PNG, sen pisar nada e coa imaxe de orixe intacta",
  async run(app, c) {
    const made = await app.ev(`(()=>{const k=document.createElement('canvas'); k.width=300; k.height=200; const g=k.getContext('2d'); g.fillStyle='#d03030'; g.fillRect(0,0,300,200); g.fillStyle='#fff'; g.fillRect(100,50,100,100); return k.toDataURL('image/png').split(',')[1];})()`);
    const src = path.join(app.media, "logo.png");
    fs.writeFileSync(src, Buffer.from(made, "base64"));
    const before = sha(src);
    await app.open(src);
    await app.ev(`document.querySelectorAll('li[role=option] input[type=checkbox]').forEach(x=>{ if(x.checked) x.click(); })`);
    const tabs = await app.tabs();
    c.ok(tabs.indexOf("Iconas") === tabs.indexOf("Anotar") + 1, "«Iconas» vai debaixo de «Anotar»", tabs.join(","));
    await app.click("Iconas");
    await app.waitFor(`[...document.querySelectorAll('.stack button')].some(b=>b.textContent.trim()==='Crear o paquete de iconas')`, "o botón", 20000);
    c.ok(await app.ev(`[...document.querySelectorAll('.stack .toggle')].length === 4`), "catro interruptores: Windows, macOS, Web e PNG soltos");
    c.ok(await app.ev(`!!document.querySelector('.stack .locked-tools .locked-tools:not(.on) [role=radiogroup]')`), "a imaxe non é cadrada: «Forma» está activa");
    await app.click("Crear o paquete de iconas");
    const dir = path.join(app.media, "logo-iconas");
    for (let i = 0; i < 60 && !fs.existsSync(path.join(dir, "icon.icns")); i++) await app.sleep(300);
    const all = fs.existsSync(dir) ? fs.readdirSync(dir) : [];
    c.ok(["icon.ico", "icon.icns", "favicon.ico", "apple-touch-icon.png", "site.webmanifest", "png"].every((f) => all.includes(f)), "o cartafol leva ICO, ICNS, favicons, manifesto e PNG", all.join(","));
    c.ok(fs.existsSync(path.join(dir, "png", "icon-1024.png")) && fs.existsSync(path.join(dir, "png", "icon-16.png")), "e os PNG de 16 a 1024");
    c.ok(sha(src) === before, "a imaxe de orixe queda igual, byte a byte");
    await app.click("Crear o paquete de iconas");
    for (let i = 0; i < 60 && !fs.existsSync(path.join(app.media, "logo-iconas (2)", "icon.ico")); i++) await app.sleep(300);
    c.ok(fs.existsSync(path.join(app.media, "logo-iconas (2)")) && fs.existsSync(path.join(dir, "icon.ico")), "a segunda vez non pisa: cartafol co « (2)» ao final");
  },
};
