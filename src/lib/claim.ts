// Varias pestanas de ferramentas poden compartir un mesmo estado do visor (a relación de aspecto, o recorte...).
// Ao cambiar de ficheiro a pestana créase de novo: a nova instancia pode nacer antes de que a vella se destrúa,
// e a vella non pode borrar o que acaba de poñer a nova. Quen reclama a clave é o último en chegar.

const owners = new Map<string, symbol>();

export function claim(key: string) {
  const me = Symbol(key);
  owners.set(key, me);
  return {
    /** Executa `fn` só se ninguén máis reclamou a clave desde entón. */
    release(fn: () => void) {
      if (owners.get(key) !== me) return;
      owners.delete(key);
      fn();
    },
  };
}
