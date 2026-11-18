/** Cor (brillo, contraste, saturación) como matriz 5×4 de SVG para o visor: imita o filtro `eq` do motor, en luminancia e sen tocar a croma co contraste. */
export function lookMatrix(brightness: number, contrast: number, saturation: number): string {
  const L = [0.2126, 0.7152, 0.0722];
  // O brillo e o contraste de `eq` van no rango limitado do vídeo (16–235): 255/219 e pivote en 128.
  const offset = brightness * (255 / 219) - ((128 - 16) / 219) * (contrast - 1);
  const m1 = [0, 1, 2].map((i) => [0, 1, 2].map((j) => (i === j ? 1 : 0) + (contrast - 1) * L[j]));
  const m2 = [0, 1, 2].map((i) => [0, 1, 2].map((j) => (i === j ? saturation : 0) + (1 - saturation) * L[j]));
  const rows = [0, 1, 2].map((i) => [0, 1, 2].map((j) => m2[i].reduce((a, v, k) => a + v * m1[k][j], 0)));
  const f = (v: number) => v.toFixed(4);
  return [...rows.map((r) => `${r.map(f).join(" ")} 0 ${f(offset)}`), "0 0 0 1 0"].join(" ");
}
