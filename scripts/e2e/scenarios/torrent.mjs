// Torrents na app real: abrir un .torrent en Descargar por ligazón (sen yt-dlp nin FFmpeg), ver o nome e os ficheiros, marcar cales e cancelar unha
// descarga que non avanza sen deixar restos. A descarga de verdade (con sementador local) probase no motor (tests/torrent.rs).
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

/** Bencode mínimo para facer un .torrent de proba. */
const enc = (v) =>
  typeof v === "number"
    ? Buffer.from(`i${v}e`)
    : typeof v === "string"
      ? enc(Buffer.from(v))
      : Buffer.isBuffer(v)
        ? Buffer.concat([Buffer.from(`${v.length}:`), v])
        : Array.isArray(v)
          ? Buffer.concat([Buffer.from("l"), ...v.map(enc), Buffer.from("e")])
          : Buffer.concat([Buffer.from("d"), ...Object.keys(v).sort().flatMap((k) => [enc(k), enc(v[k])]), Buffer.from("e")]);

function makeTorrent(file, name, parts) {
  const data = Buffer.concat(parts.map((p) => p.data));
  const pieceLength = 16384;
  const hashes = [];
  for (let i = 0; i < data.length; i += pieceLength) hashes.push(crypto.createHash("sha1").update(data.subarray(i, i + pieceLength)).digest());
  const info = { name, "piece length": pieceLength, pieces: Buffer.concat(hashes), files: parts.map((p) => ({ length: p.data.length, path: [p.name] })) };
  fs.writeFileSync(file, enc({ info }));
}

export default {
  about: "Torrents: abrir un .torrent, ver nome e ficheiros, marcar cales, sen precisar yt-dlp, e cancelar sen deixar restos",
  async run(app, c) {
    const file = path.join(app.media, "exemplo.torrent");
    makeTorrent(file, "exemplo", [
      { name: "a.bin", data: Buffer.alloc(40_000, 7) },
      { name: "b.txt", data: Buffer.from("ola torrent") },
    ]);

    await app.radial("Descargar por ligazón");
    c.ok(await app.ev(`[...document.querySelectorAll('[role=dialog] button')].some(b=>b.innerText.includes('ficheiro .torrent'))`), "co campo baleiro ofrece elixir un ficheiro .torrent");
    await app.ev(`(()=>{const i=document.querySelector('[role=dialog] input[type=text]'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i, ${JSON.stringify(file)}); i.dispatchEvent(new Event('input',{bubbles:true}));})()`);
    await app.sleep(600);
    c.ok(!/Falta instalar/.test(await app.ev(`document.querySelector('[role=dialog]').innerText`)), "un torrent non pide yt-dlp nin FFmpeg");
    c.ok((await app.ev(`document.querySelector('[data-testid=download-search]')?.disabled`)) === false, "o botón Buscar está activo");
    await app.ev(`document.querySelector('[data-testid=download-search]')?.click()`);
    await app.waitFor(`document.querySelector('[role=dialog] [data-testid=torrent-info]') || document.querySelector('[role=dialog] .warn')`, "ler o torrent", 30000);
    const card = await app.ev(`document.querySelector('[data-testid=torrent-info]')?.innerText.replace(/\\s+/g,' ').trim() ?? ''`);
    c.ok(/Torrent/.test(card) && /exemplo/.test(card) && /2 ficheiros/.test(card), "amosa o nome e canto ten", card);
    const header = () => app.ev(`document.querySelector('[data-testid=playlist] .all')?.innerText.replace(/\\s+/g,' ').trim() ?? ''`);
    const go = () => app.ev(`document.querySelector('[data-testid=download-go]')?.innerText.trim()`);
    c.ok((await header()) === "2 de 2 ficheiros", "lista os dous ficheiros, marcados", await header());
    c.ok((await go()) === "Descargar 2 ficheiros", "o botón di cantos descarga", await go());
    await app.ev(`document.querySelectorAll('[data-testid=playlist] li input')[0].click()`);
    await app.sleep(200);
    c.ok((await header()) === "1 de 2 ficheiros" && (await go()) === "Descargar 1 ficheiro", "desmarcar un cambia a conta", `${await header()} / ${await go()}`);
    await app.shot("torrent.png");
    await app.click("Pechar");

    // Unha ligazón que non é de torrent segue indo por yt-dlp (ou pide a extensión), e un magnet sen identificador rexéitase.
    const bad = await app.ev(`window.__TAURI_INTERNALS__.invoke('torrent_info', { source: 'magnet:?dn=sen-hash' }).then(()=>'ok', (e)=>String(e?.message ?? e))`);
    c.ok(bad !== "ok" && /identificador/.test(bad), "un magnet sen identificador dá erro", bad);

    // Cancelar unha descarga sen sementadores: para e non deixa nada no destino.
    const dest = path.join(app.media, "torrent-destino");
    await app.ev(`window.__TAURI_INTERNALS__.invoke('begin_job')`);
    const pending = app.ev(`window.__TAURI_INTERNALS__.invoke('torrent_download', { source: ${JSON.stringify(file)}, dir: ${JSON.stringify(dest)}, files: null }).then(()=>'ok', (e)=>String(e?.message ?? e))`);
    await app.sleep(3000);
    await app.ev(`window.__TAURI_INTERNALS__.invoke('cancel_job')`);
    const res = await pending;
    c.ok(res !== "ok" && /Cancelei/.test(res), "cancelar para a descarga", res);
    c.ok(!fs.existsSync(dest) || fs.readdirSync(dest).length === 0, "sen cartafoles nin ficheiros soltos no destino", fs.existsSync(dest) ? fs.readdirSync(dest).join(",") : "");
  },
};
