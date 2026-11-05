// Smoke test of the browser wasm bundle under Node: loads the same .wasm the web app uses.
import { readFileSync } from "node:fs";
import { initSync, simulate } from "../web/src/wasm-pkg/loopsettle_wasm.js";
initSync({ module: readFileSync(new URL("../web/src/wasm-pkg/loopsettle_wasm_bg.wasm", import.meta.url)) });
for (const [n, p] of [[20, 20000], [50, 100000]]) {
  const t0 = performance.now();
  const r = JSON.parse(simulate(20260930, n, p, 1000, 10000, 0.4, 0.5));
  const ms = performance.now() - t0;
  if (!r.engines?.length || !r.edges?.length || r.ub_brl <= 0) throw new Error("bad result");
  console.log(`participants=${n} payments=${r.payments} ub=${r.ub_brl.toFixed(0)} edges=${r.edges.length} time_ms=${ms.toFixed(0)}`);
}
