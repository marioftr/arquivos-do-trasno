// Dar volta ao son: o visor prepara o son procesado e soa de verdade (non só ao exportar).
import { spawnSync } from "node:child_process";
import path from "node:path";
import { ffmpegBin } from "../harness.mjs";

export default {
  about: "Son → «Dar volta ao son»: a vista previa do visor soa ao revés en sincronía",
  async run(app, c) {
    // 8 s de vídeo cun tono só nos 2 primeiros segundos.
    const src = path.join(app.media, "tono.mp4");
    const r = spawnSync(ffmpegBin(), ["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc=s=160x120:r=10:d=8", "-f", "lavfi", "-i", "sine=f=440:r=44100:d=8,volume='if(lt(t,2),1,0)':eval=frame", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", src]);
    if (r.status !== 0) throw new Error("FFmpeg: " + r.stderr);
    await app.open(src);
    await app.click("Son");
    await app.slider("Graves", 1); // crea o grafo de son do visor (e o analizador de proba)

    const listen = async (at) => {
      await app.ev(`(()=>{const v=document.querySelector('video'); v.pause(); v.currentTime=${at}})()`);
      await app.sleep(300);
      await app.ev(`document.querySelector('video').play()`, true);
      await app.sleep(700);
      const l = await app.level(500);
      await app.ev(`document.querySelector('video').pause()`);
      return l;
    };
    c.ok((await listen(0.3)) > 0.02, "sen axustes, o tono dos 2 primeiros segundos soa");

    await app.toggle("Dar volta ao son");
    await app.waitFor(`!!window.__trasnoAux && window.__trasnoAux.readyState >= 3`, "o son procesado listo", 60000);
    c.ok(!(await app.ev(`/Preparando o son para o visor/.test(document.querySelector('.stack').innerText)`)), "o aviso de preparación desaparece ao rematar");
    c.ok((await listen(0.3)) < 0.005, "ao revés, o principio queda en silencio");
    c.ok((await listen(6.8)) > 0.02, "ao revés, o tono soa ao final do tramo");

    await app.toggle("Dar volta ao son");
    await app.sleep(500);
    c.ok((await listen(0.3)) > 0.02, "ao quitar o axuste volve o son orixinal");
  },
};
