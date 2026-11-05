// Small Canvas 2D helpers: crisp drawing on high-DPI screens and theme colours from CSS.

export function setup(canvas: HTMLCanvasElement): { ctx: CanvasRenderingContext2D; w: number; h: number } {
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const w = canvas.clientWidth;
  const h = canvas.clientHeight;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  const ctx = canvas.getContext("2d")!;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, w, h);
  return { ctx, w, h };
}

export function color(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || "#888";
}

/** Compact BRL amount; `units` holds the language's suffixes (pt: bi/mi/mil, en: bn/M/k). */
export function fmtBRL(v: number, units = { bn: "bi", mn: "mi", k: "mil" }): string {
  const a = Math.abs(v);
  if (a >= 1e9) return `R$ ${(v / 1e9).toFixed(2)} ${units.bn}`;
  if (a >= 1e6) return `R$ ${(v / 1e6).toFixed(2)} ${units.mn}`;
  if (a >= 1e3) return `R$ ${(v / 1e3).toFixed(1)} ${units.k}`;
  return `R$ ${v.toFixed(0)}`;
}

export function fmtPct(x: number, digits = 1): string {
  return `${(100 * x).toFixed(digits)}%`;
}

export interface LineSeries {
  label: string;
  points: [number, number][];
  color: string;
  dashed?: boolean;
}

/** Line chart with light axes. Values are drawn as given; labels formatted by callbacks. */
export function lineChart(
  canvas: HTMLCanvasElement,
  series: LineSeries[],
  opts: { xFmt: (v: number) => string; yFmt: (v: number) => string; yMin?: number },
) {
  const { ctx, w, h } = setup(canvas);
  const mr = 12, mt = 10, mb = 26;
  const all = series.flatMap((s) => s.points);
  if (!all.length) return;
  const xs = all.map((p) => p[0]);
  const ys = all.map((p) => p[1]);
  const x0 = Math.min(...xs), x1 = Math.max(...xs);
  const y0 = opts.yMin ?? Math.min(0, ...ys);
  let y1 = Math.max(...ys);
  if (y1 === y0) y1 = y0 + 1;
  ctx.font = "11px 'IBM Plex Sans', system-ui, sans-serif";
  // Left margin fits the widest y label, so values like "R$ 7.98 mi" are never clipped.
  let ml = 40;
  for (let i = 0; i <= 4; i++) ml = Math.max(ml, ctx.measureText(opts.yFmt(y0 + ((y1 - y0) * i) / 4)).width + 12);
  const sx = (x: number) => ml + ((x - x0) / (x1 - x0 || 1)) * (w - ml - mr);
  const sy = (y: number) => h - mb - ((y - y0) / (y1 - y0)) * (h - mt - mb);
  ctx.fillStyle = color("--muted");
  ctx.strokeStyle = color("--grid");
  ctx.lineWidth = 1;
  for (let i = 0; i <= 4; i++) {
    const v = y0 + ((y1 - y0) * i) / 4;
    const y = sy(v);
    ctx.beginPath();
    ctx.moveTo(ml, y);
    ctx.lineTo(w - mr, y);
    ctx.stroke();
    ctx.textAlign = "right";
    ctx.fillText(opts.yFmt(v), ml - 6, y + 4);
  }
  for (let i = 0; i <= 4; i++) {
    const v = x0 + ((x1 - x0) * i) / 4;
    // The last label is right-aligned so it stays inside the canvas.
    ctx.textAlign = i === 4 ? "right" : "center";
    ctx.fillText(opts.xFmt(v), i === 4 ? w - mr : sx(v), h - 8);
  }
  for (const s of series) {
    ctx.strokeStyle = s.color;
    ctx.fillStyle = s.color;
    ctx.lineWidth = 2;
    ctx.setLineDash(s.dashed ? [5, 4] : []);
    ctx.beginPath();
    s.points.forEach(([x, y], i) => (i ? ctx.lineTo(sx(x), sy(y)) : ctx.moveTo(sx(x), sy(y))));
    ctx.stroke();
    ctx.setLineDash([]);
    if (s.points.length < 12) s.points.forEach(([x, y]) => ctx.fillRect(sx(x) - 2.5, sy(y) - 2.5, 5, 5));
  }
}
