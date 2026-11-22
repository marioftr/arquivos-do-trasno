// Funciona sen conexión: a páxina e os seus ficheiros gárdanse na primeira visita. Os ficheiros con hash (/assets) non cambian nunca.
const CACHE = "trasno-web-v1";
self.addEventListener("install", (e) => {
  self.skipWaiting();
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(["/", "/manifest.webmanifest"])));
});
self.addEventListener("activate", (e) => {
  e.waitUntil(caches.keys().then((ks) => Promise.all(ks.filter((k) => k !== CACHE).map((k) => caches.delete(k)))).then(() => self.clients.claim()));
});
self.addEventListener("fetch", (e) => {
  const r = e.request;
  if (r.method !== "GET" || new URL(r.url).origin !== location.origin) return;
  if (r.mode === "navigate") {
    // A páxina, da rede se hai; se non, a gardada.
    e.respondWith(
      fetch(r)
        .then((res) => {
          const copy = res.clone();
          caches.open(CACHE).then((c) => c.put("/", copy));
          return res;
        })
        .catch(() => caches.match("/")),
    );
    return;
  }
  e.respondWith(
    caches.match(r).then(
      (hit) =>
        hit ||
        fetch(r).then((res) => {
          if (res.ok) {
            const copy = res.clone();
            caches.open(CACHE).then((c) => c.put(r, copy));
          }
          return res;
        }),
    ),
  );
});
