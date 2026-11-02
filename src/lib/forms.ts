// Estado dun formulario de ferramenta con historial (desfacer/refacer) e
// valores por defecto (restaurar un campo ou todos). Os campos úsanse con
// `bind:value={$form.campo}`; Ctrl+Z / Ctrl+Y actúan sobre o formulario activo.

import { writable, get, type Readable, type Writable } from "svelte/store";

export interface Form<T extends Record<string, any>> extends Writable<T> {
  defaults: T;
  undo: () => void;
  redo: () => void;
  resetKey: (key: keyof T) => void;
  resetAll: () => void;
  canUndo: Readable<boolean>;
  canRedo: Readable<boolean>;
  dirty: Readable<boolean>;
}

export const activeForm = writable<Form<any> | null>(null);

const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

// Os axustes de cada ferramenta lémbranse (por tipo de ficheiro) ao cambiar de pestana e entre sesións.
const MEMORY_KEY = "arquivos-do-trasno.axustes";
let memory: Record<string, Record<string, unknown>> | null = null;
function loadMemory() {
  if (memory) return memory;
  try {
    memory = JSON.parse(localStorage.getItem(MEMORY_KEY) ?? "{}");
  } catch {
    memory = {};
  }
  return memory!;
}
let memTimer: ReturnType<typeof setTimeout> | undefined;
function remember(key: string, value: Record<string, unknown>) {
  loadMemory()[key] = value;
  clearTimeout(memTimer);
  memTimer = setTimeout(() => {
    try {
      localStorage.setItem(MEMORY_KEY, JSON.stringify(memory));
    } catch {
      // sen almacenamento: só vale mentres a app está aberta
    }
  }, 400);
}

// Axustes de cada ficheiro mentres a app está aberta: ao volver a un ficheiro, volven os que levaba.
const fileMemory = new Map<string, Record<string, unknown>>();

export function createForm<T extends Record<string, any>>(defaults: T, memoryKey?: string, fileKey?: string): Form<T> {
  // Con `memoryKey`, arranca cos últimos valores usados (só os campos que seguen existindo).
  // Con `fileKey`, un ficheiro que xa se tocou arranca cos seus propios valores, non cos dos demais.
  const saved = (fileKey ? fileMemory.get(fileKey) : undefined) ?? (memoryKey ? loadMemory()[memoryKey] : undefined);
  const start = { ...defaults } as T;
  if (saved) for (const k of Object.keys(defaults)) if (k in saved && typeof saved[k] === typeof defaults[k]) (start as any)[k] = saved[k];
  const store = writable<T>({ ...start });
  const canUndo = writable(false);
  const canRedo = writable(false);
  const dirty = writable(false);
  const past: T[] = [];
  let future: T[] = [];
  let last: T = { ...start };
  let burstStart: T | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let applying = false;

  const sync = () => {
    canUndo.set(past.length > 0 || burstStart !== null);
    canRedo.set(future.length > 0);
    dirty.set(!same(last, defaults));
  };

  /** Cada rolda de cambios seguidos (p. ex. arrastrar un slider) é un só paso. */
  const flush = () => {
    if (burstStart) {
      past.push(burstStart);
      if (past.length > 100) past.shift();
      burstStart = null;
      future = [];
    }
    clearTimeout(timer);
  };

  store.subscribe((v) => {
    if (memoryKey) remember(memoryKey, v);
    if (fileKey) fileMemory.set(fileKey, { ...v });
    if (applying || same(v, last)) return;
    if (!burstStart) burstStart = { ...last };
    last = { ...v };
    clearTimeout(timer);
    timer = setTimeout(() => {
      flush();
      sync();
    }, 450);
    sync();
  });

  const apply = (v: T) => {
    applying = true;
    last = { ...v };
    store.set({ ...v });
    applying = false;
  };

  const form: Form<T> = {
    subscribe: store.subscribe,
    set: store.set,
    update: store.update,
    defaults,
    undo() {
      flush();
      const prev = past.pop();
      if (!prev) return;
      future.push({ ...last });
      apply(prev);
      sync();
    },
    redo() {
      flush();
      const next = future.pop();
      if (!next) return;
      past.push({ ...last });
      apply(next);
      sync();
    },
    resetKey(key) {
      store.update((v) => ({ ...v, [key]: defaults[key] }));
    },
    resetAll() {
      store.set({ ...defaults });
    },
    canUndo,
    canRedo,
    dirty,
  };
  return form;
}

/** Rexistra o formulario como o activo mentres o compoñente viva. */
export function useForm(form: Form<any>): () => void {
  activeForm.set(form);
  return () => {
    if (get(activeForm) === form) activeForm.set(null);
  };
}
