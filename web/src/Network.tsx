import { useEffect, useRef } from "react";
import { color, setup } from "./canvas";
import type { SimResult } from "./types";

/** Participants on a ring (largest at the top, clockwise by size). Each edge is drawn
 *  twice on top of each other: the directional part in grey, the CDFD circular part
 *  in the accent colour, so the share of money that "loops" is visible at a glance. */
export function Network({ data }: { data: SimResult }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const draw = () => {
      const c = ref.current;
      if (!c) return;
      const { ctx, w, h } = setup(c);
      const n = data.nodes.length;
      const cx = w / 2, cy = h / 2;
      const R = Math.min(w, h) / 2 - 26;
      const pos = data.nodes.map((_, i) => {
        const a = -Math.PI / 2 + (2 * Math.PI * i) / n;
        return [cx + R * Math.cos(a), cy + R * Math.sin(a)] as const;
      });
      const maxFlow = Math.max(...data.edges.map((e) => e[2]), 1);
      const circ = color("--circular");
      const dir = color("--directional");
      // Draw smaller edges first so large ones stay on top.
      const edges = [...data.edges].reverse();
      for (const [a, b, flow, cir] of edges) {
        const [x1, y1] = pos[a];
        const [x2, y2] = pos[b];
        // Curve to the left of the direction of travel: A->B and B->A do not overlap.
        const mx = (x1 + x2) / 2 + (y2 - y1) * 0.12;
        const my = (y1 + y2) / 2 - (x2 - x1) * 0.12;
        const width = 0.4 + 7 * Math.sqrt(flow / maxFlow);
        ctx.globalAlpha = 0.35;
        ctx.strokeStyle = dir;
        ctx.lineWidth = width;
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.quadraticCurveTo(mx, my, x2, y2);
        ctx.stroke();
        if (cir > 0) {
          ctx.globalAlpha = 0.75;
          ctx.strokeStyle = circ;
          ctx.lineWidth = width * (cir / flow);
          ctx.beginPath();
          ctx.moveTo(x1, y1);
          ctx.quadraticCurveTo(mx, my, x2, y2);
          ctx.stroke();
        }
      }
      ctx.globalAlpha = 1;
      const maxAct = Math.max(...data.nodes.map((d) => d.activity));
      const ink = color("--ink");
      const bg = color("--surface");
      data.nodes.forEach((d, i) => {
        const [x, y] = pos[i];
        const r = 3 + 13 * Math.sqrt(d.activity / maxAct);
        ctx.beginPath();
        ctx.arc(x, y, r, 0, 2 * Math.PI);
        ctx.fillStyle = bg;
        ctx.fill();
        ctx.lineWidth = 2;
        // Net receivers get a solid ring, net payers a dashed one.
        ctx.setLineDash(d.net_brl < 0 ? [3, 2] : []);
        ctx.strokeStyle = ink;
        ctx.stroke();
        ctx.setLineDash([]);
      });
    };
    draw();
    window.addEventListener("resize", draw);
    return () => window.removeEventListener("resize", draw);
  }, [data]);

  return (
    <canvas
      ref={ref}
      className="network"
      role="img"
      aria-label={`Rede de ${data.nodes.length} participantes; a cor de destaque mostra o componente circular máximo (CDFD) de cada fluxo.`}
    />
  );
}
