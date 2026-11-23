/** A última versión da app de escritorio: pregúntase a GitHub só cando se abre a páxina de descarga. */

export const REPO_URL = "https://github.com/marioftr/arquivos-do-trasno";
export const RELEASES_URL = `${REPO_URL}/releases`;
export const LATEST_URL = `${RELEASES_URL}/latest`;
const API = "https://api.github.com/repos/marioftr/arquivos-do-trasno/releases/latest";

export interface Asset {
  /** «exe» (instalador) ou «msi» (paquete para equipos xestionados). */
  kind: "exe" | "msi";
  name: string;
  url: string;
  size: number;
}
export interface Release {
  tag: string;
  version: string;
  date: string;
  page: string;
  assets: Asset[];
}

let cached: Promise<Release> | null = null;

/** Devolve a última versión publicada; se GitHub non responde, lanza (a páxina cae á ligazón xeral). */
export function latestRelease(): Promise<Release> {
  cached ??= (async () => {
    const res = await fetch(API, { headers: { Accept: "application/vnd.github+json" } });
    if (!res.ok) throw new Error(String(res.status));
    const j = (await res.json()) as { tag_name: string; published_at: string; html_url: string; assets: { name: string; browser_download_url: string; size: number }[] };
    const assets: Asset[] = [];
    for (const a of j.assets ?? []) {
      const kind = /-setup\.exe$/i.test(a.name) ? "exe" : /\.msi$/i.test(a.name) ? "msi" : null;
      if (kind) assets.push({ kind, name: a.name, url: a.browser_download_url, size: a.size });
    }
    if (!assets.length) throw new Error("sen ficheiros");
    return { tag: j.tag_name, version: j.tag_name.replace(/^v/, ""), date: j.published_at, page: j.html_url, assets };
  })().catch((e) => {
    cached = null;
    throw e;
  });
  return cached;
}

/** O sistema de quen visita: a app só existe para Windows de momento. */
export function platform(): "windows" | "mobile" | "other" {
  const ua = navigator.userAgent;
  if (/android|iphone|ipad|ipod/i.test(ua)) return "mobile";
  return /windows/i.test(ua) ? "windows" : "other";
}
