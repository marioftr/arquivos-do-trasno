// Caché pequena en memoria doutros visores: volver a un ficheiro xa visto non o carga de novo.
// Dura mentres a aplicación siga aberta.

function lru<V>(max: number) {
  const m = new Map<string, V>();
  return {
    get(k: string): V | undefined {
      const v = m.get(k);
      if (v !== undefined) {
        m.delete(k);
        m.set(k, v);
      }
      return v;
    },
    set(k: string, v: V) {
      m.delete(k);
      m.set(k, v);
      while (m.size > max) m.delete(m.keys().next().value as string);
    },
    delete(k: string) {
      m.delete(k);
    },
  };
}

/** Último instante visto de cada vídeo ou audio, para retomalo ao volver a el. */
export const playPos = new Map<string, number>();

/** Ondas de son (picos) por ficheiro. */
export const peaksCache = lru<number[]>(40);
/** Texto dos ficheiros de texto. */
export const textCache = lru<string>(30);
/** Vista previa xa preparada de vídeo/audio que o navegador non le: ruta e se está recodificada. */
export const previewCache = lru<{ path: string; converted: boolean }>(40);
