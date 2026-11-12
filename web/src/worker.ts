/// <reference lib="webworker" />
// Runs the Rust simulator (compiled to wasm) off the main thread, so the UI never blocks.
// Two kinds of request: a simulation (`params`) or the analysis of the user's own payment file
// (`file`, the file's text). The file is read in the browser and never sent anywhere.
import init, { analyze_file, simulate } from "./wasm-pkg/loopsettle_wasm.js";
import type { SimParams } from "./types";

const ready = init();

self.onmessage = async (ev: MessageEvent<{ id: number; params?: SimParams; file?: string }>) => {
  const { id, params: p, file } = ev.data;
  try {
    await ready;
    const t0 = performance.now();
    const json = file !== undefined ? analyze_file(file) : simulate(p!.seed, p!.participants, p!.payments, p!.windowMs, p!.maxDelayMs, p!.receiverBias, p!.liquidityFraction);
    const result = JSON.parse(json);
    (self as unknown as Worker).postMessage({ id, kind: file !== undefined ? "file" : "sim", ok: true, result, ms: performance.now() - t0 });
  } catch (e) {
    (self as unknown as Worker).postMessage({ id, kind: file !== undefined ? "file" : "sim", ok: false, error: String(e) });
  }
};
