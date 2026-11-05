/// <reference lib="webworker" />
// Runs the Rust simulator (compiled to wasm) off the main thread, so the UI never blocks.
import init, { simulate } from "./wasm-pkg/loopsettle_wasm.js";
import type { SimParams } from "./types";

const ready = init();

self.onmessage = async (ev: MessageEvent<{ id: number; params: SimParams }>) => {
  const { id, params: p } = ev.data;
  try {
    await ready;
    const t0 = performance.now();
    const json = simulate(p.seed, p.participants, p.payments, p.windowMs, p.maxDelayMs, p.receiverBias, p.liquidityFraction);
    const result = JSON.parse(json);
    (self as unknown as Worker).postMessage({ id, ok: true, result, ms: performance.now() - t0 });
  } catch (e) {
    (self as unknown as Worker).postMessage({ id, ok: false, error: String(e) });
  }
};
