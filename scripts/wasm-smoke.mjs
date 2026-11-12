// Smoke test of the browser wasm bundle under Node: loads the same .wasm the web app uses.
import { readFileSync } from "node:fs";
import { analyze_file, initSync, simulate } from "../web/src/wasm-pkg/loopsettle_wasm.js";
initSync({ module: readFileSync(new URL("../web/src/wasm-pkg/loopsettle_wasm_bg.wasm", import.meta.url)) });
for (const [n, p] of [[20, 20000], [50, 100000]]) {
  const t0 = performance.now();
  const r = JSON.parse(simulate(20260930, n, p, 1000, 10000, 0.4, 0.5));
  const ms = performance.now() - t0;
  if (!r.engines?.length || !r.edges?.length || r.ub_brl <= 0 || r.floor?.length !== 7) throw new Error("bad result");
  console.log(`participants=${n} payments=${r.payments} ub=${r.ub_brl.toFixed(0)} edges=${r.edges.length} time_ms=${ms.toFixed(0)}`);
}
// The user's-file path: the example payment file shipped in data/samples.
{
  const text = readFileSync(new URL("../data/samples/payments_example.csv", import.meta.url), "utf8");
  const t0 = performance.now();
  const r = JSON.parse(analyze_file(text));
  if (r.error || r.floor?.length !== 7 || !(r.ub > 0)) throw new Error("bad file result: " + JSON.stringify(r).slice(0, 200));
  for (const p of r.floor) if (p.netting_saving_pct > p.tight_bound_pct + 1e-9 || p.tight_bound_pct > p.window_bound_pct + 1e-9) throw new Error("sandwich violated");
  if (!JSON.parse(analyze_file("not,a,payment,file\n")).error) throw new Error("a bad file must report an error");
  console.log(`file payments=${r.payments} participants=${r.participants} ub=${r.ub.toFixed(0)} tight_10s=${r.floor[3].tight_bound_pct.toFixed(4)}% time_ms=${(performance.now() - t0).toFixed(0)}`);
}
