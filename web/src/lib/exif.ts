/** Lector mínimo de EXIF en JPG: cámara, data, exposición e localización (o que importa para saber que leva unha foto). */

export interface Exif {
  make?: string;
  model?: string;
  lens?: string;
  software?: string;
  date?: string;
  exposure?: string;
  fnumber?: string;
  iso?: string;
  focal?: string;
  lat?: number;
  lon?: number;
}

const ascii = (b: DataView, off: number, n: number) => {
  let s = "";
  for (let i = 0; i < n && off + i < b.byteLength; i++) {
    const c = b.getUint8(off + i);
    if (!c) break;
    s += String.fromCharCode(c);
  }
  return s.trim();
};

export async function readExif(file: File): Promise<Exif> {
  const out: Exif = {};
  try {
    const b = new DataView(await file.slice(0, 262144).arrayBuffer());
    if (b.byteLength < 12 || b.getUint16(0) !== 0xffd8) return out;
    let at = 2;
    let base = -1;
    while (at + 4 < b.byteLength && b.getUint8(at) === 0xff) {
      const marker = b.getUint8(at + 1);
      if (marker === 0xda) break;
      const len = b.getUint16(at + 2);
      if (marker === 0xe1 && ascii(b, at + 4, 4) === "Exif") {
        base = at + 10;
        break;
      }
      at += 2 + len;
    }
    if (base < 0) return out;
    const le = b.getUint16(base) === 0x4949;
    const u16 = (o: number) => b.getUint16(base + o, le);
    const u32 = (o: number) => b.getUint32(base + o, le);
    const rat = (o: number, i = 0) => {
      const d = u32(o + i * 8 + 4);
      return d ? u32(o + i * 8) / d : 0;
    };
    const SIZE = [0, 1, 1, 2, 4, 8];
    const ifd = (ptr: number, each: (tag: number, type: number, count: number, val: number) => void) => {
      if (!ptr || base + ptr + 2 > b.byteLength) return;
      const n = Math.min(u16(ptr), 200);
      for (let i = 0; i < n; i++) {
        const e = ptr + 2 + i * 12;
        if (base + e + 12 > b.byteLength) return;
        const type = u16(e + 2);
        const count = u32(e + 4);
        const val = (SIZE[type] ?? 0) * count > 4 ? u32(e + 8) : e + 8;
        each(u16(e), type, count, val);
      }
    };
    let exifPtr = 0;
    let gpsPtr = 0;
    const text = (type: number, count: number, val: number) => (type === 2 ? ascii(b, base + val, count) : "");
    ifd(u32(4), (tag, type, count, val) => {
      if (tag === 0x010f) out.make = text(type, count, val);
      else if (tag === 0x0110) out.model = text(type, count, val);
      else if (tag === 0x0131) out.software = text(type, count, val);
      else if (tag === 0x0132) out.date ??= text(type, count, val);
      else if (tag === 0x8769) exifPtr = u32(val);
      else if (tag === 0x8825) gpsPtr = u32(val);
    });
    ifd(exifPtr, (tag, type, count, val) => {
      if (tag === 0x9003) out.date = text(type, count, val);
      else if (tag === 0xa434) out.lens = text(type, count, val);
      else if (tag === 0x829a) {
        const s = rat(val);
        out.exposure = s > 0 && s < 1 ? `1/${Math.round(1 / s)} s` : `${s} s`;
      } else if (tag === 0x829d) out.fnumber = `f/${Math.round(rat(val) * 10) / 10}`;
      else if (tag === 0x8827) out.iso = String(type === 3 ? u16(val) : u32(val));
      else if (tag === 0x920a) out.focal = `${Math.round(rat(val) * 10) / 10} mm`;
    });
    let latRef = "N";
    let lonRef = "E";
    let lat: number | undefined;
    let lon: number | undefined;
    const dms = (o: number) => rat(o, 0) + rat(o, 1) / 60 + rat(o, 2) / 3600;
    ifd(gpsPtr, (tag, type, count, val) => {
      if (tag === 1) latRef = text(type, count, val);
      else if (tag === 2) lat = dms(val);
      else if (tag === 3) lonRef = text(type, count, val);
      else if (tag === 4) lon = dms(val);
    });
    if (lat !== undefined && lon !== undefined) {
      out.lat = (latRef === "S" ? -1 : 1) * lat;
      out.lon = (lonRef === "W" ? -1 : 1) * lon;
    }
  } catch {
    // un EXIF roto non impide amosar o resto
  }
  return out;
}
