/** Subtítulos: ler e escribir SRT e VTT. */

export interface Cue {
  start: number;
  end: number;
  text: string;
}

const pad = (n: number, w = 2) => String(n).padStart(w, "0");

export function clock(sec: number, sep: "," | "."): string {
  const ms = Math.max(0, Math.round(sec * 1000));
  return `${pad(Math.floor(ms / 3_600_000))}:${pad(Math.floor(ms / 60_000) % 60)}:${pad(Math.floor(ms / 1000) % 60)}${sep}${pad(ms % 1000, 3)}`;
}

export const toSrt = (cues: Cue[]) => cues.map((c, i) => `${i + 1}\n${clock(c.start, ",")} --> ${clock(c.end, ",")}\n${c.text}\n`).join("\n");
export const toVtt = (cues: Cue[]) => `WEBVTT\n\n${cues.map((c) => `${clock(c.start, ".")} --> ${clock(c.end, ".")}\n${c.text}\n`).join("\n")}`;
export const toTxt = (cues: Cue[]) => cues.map((c) => c.text).join("\n");

const TIME = /(\d+):(\d{2}):(\d{2})[,.](\d{1,3})|(\d{1,2}):(\d{2})[,.](\d{1,3})/;
function seconds(t: string): number | null {
  const m = TIME.exec(t);
  if (!m) return null;
  return m[1] !== undefined ? Number(m[1]) * 3600 + Number(m[2]) * 60 + Number(m[3]) + Number(m[4].padEnd(3, "0")) / 1000 : Number(m[5]) * 60 + Number(m[6]) + Number(m[7].padEnd(3, "0")) / 1000;
}

/** Le un SRT ou un VTT (as notas e a cabeceira do VTT sáltanse). */
export function parseSubs(text: string): Cue[] {
  const cues: Cue[] = [];
  for (const block of text.replace(/^﻿/, "").replace(/\r/g, "").split(/\n{2,}/)) {
    const lines = block.split("\n").map((l) => l.trimEnd());
    const at = lines.findIndex((l) => l.includes("-->"));
    if (at < 0) continue;
    const [a, b] = lines[at].split("-->");
    const start = seconds(a);
    const end = seconds(b ?? "");
    const body = lines.slice(at + 1).join("\n").trim();
    if (start !== null && end !== null && body) cues.push({ start, end, text: body });
  }
  return cues;
}

/** Move os tempos `shift` segundos e multiplícaos por `ratio` (cambio de cadencia); o que queda antes do cero descártase. */
export function retime(cues: Cue[], shift: number, ratio: number): Cue[] {
  return cues
    .map((c) => ({ ...c, start: c.start * ratio + shift, end: c.end * ratio + shift }))
    .filter((c) => c.end > 0)
    .map((c) => ({ ...c, start: Math.max(0, c.start) }));
}
