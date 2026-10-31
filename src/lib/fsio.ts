// Escritura de bytes xerados na interface (imaxes dun PDF, modelos 3D...).
// A gravación é atómica e nunca pisa o ficheiro de orixe (ver `write_bytes`).

import { call } from "./errors";

export async function writeBytes(path: string, source: string, bytes: Uint8Array, overwrite = false): Promise<void> {
  await call("write_bytes", bytes, {
    headers: {
      "x-path": encodeURIComponent(path),
      "x-source": encodeURIComponent(source),
      "x-overwrite": overwrite ? "1" : "0",
    },
  });
}
