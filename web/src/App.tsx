import { useEffect, useRef, useState } from "react";
import { Network } from "./Network";
import { color, fmtBRL, fmtPct, lineChart } from "./canvas";
import { detectLang, LOCALE, saveLang, STRINGS, type Lang, type Strings } from "./i18n";
import type { EngineJson, FileResult, FloorPoint, SimParams, SimResult } from "./types";

// Reduced mode on phones and low-end devices: smaller default runs, same model.
const reduced =
  typeof window !== "undefined" &&
  (window.matchMedia("(max-width: 640px)").matches || (navigator.hardwareConcurrency || 4) <= 2);

const WINDOWS = [500, 1000, 2000, 5000, 10000];
const DELAYS = [2000, 5000, 10000];

const DEFAULTS: SimParams = {
  seed: 20260930,
  participants: reduced ? 20 : 40,
  payments: reduced ? 20_000 : 100_000,
  windowMs: 1000,
  maxDelayMs: 10000,
  receiverBias: 0.4,
  liquidityFraction: 0.5,
};

function label(s: Strings, name: string): string {
  const e = s.engines;
  if (name === "gross_real_time") return e.gross_real_time;
  if (name === "deferred_net_1x") return e.deferred_net_1x;
  if (name === "deferred_net_96x") return e.deferred_net_96x;
  if (name.startsWith("micro_cycle_all")) return e.micro_all;
  if (name.startsWith("micro_cycle_shortfall")) return e.micro_shortfall;
  return name;
}

function Slider(props: { id: string; label: string; value: number; min: number; max: number; step: number; show: string; onChange: (v: number) => void }) {
  return (
    <div className="field">
      <label htmlFor={props.id}>
        {props.label} <output htmlFor={props.id}>{props.show}</output>
      </label>
      <input id={props.id} type="range" min={props.min} max={props.max} step={props.step} value={props.value} onChange={(e) => props.onChange(Number(e.target.value))} />
    </div>
  );
}

// Files above this size are refused: the browser keeps the whole file in memory.
const MAX_FILE_BYTES = 200 * 1024 * 1024;

// The worker is created inside the effect (not in useMemo): React StrictMode in dev runs
// effect cleanup once and then re-runs the effect, so a memoized worker would stay terminated
// and the UI would hang on "Simulando…".
function useWorker() {
  const [worker, setWorker] = useState<Worker | null>(null);
  useEffect(() => {
    const w = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
    setWorker(w);
    return () => w.terminate();
  }, []);
  return worker;
}

export function App() {
  const [lang, setLang] = useState<Lang>(detectLang);
  const [params, setParams] = useState<SimParams>(DEFAULTS);
  const [data, setData] = useState<SimResult | null>(null);
  const [status, setStatus] = useState<"running" | "idle" | "error">("running");
  const [elapsed, setElapsed] = useState(0);
  const [error, setError] = useState("");
  const worker = useWorker();
  const reqId = useRef(0);
  const fileReq = useRef(0);
  const [file, setFile] = useState<{ status: "idle" | "running" | "done" | "error"; name?: string; result?: FileResult; error?: string }>({ status: "idle" });
  const s = STRINGS[lang];
  const loc = LOCALE[lang];

  useEffect(() => {
    document.documentElement.lang = loc;
    document.title = lang === "pt" ? "LoopSettle — liquidez circular no Pix" : "LoopSettle — circular liquidity in Pix";
  }, [lang, loc]);

  useEffect(() => {
    if (!worker) return;
    worker.onmessage = (ev) => {
      const m = ev.data;
      if (m.kind === "file") {
        if (m.id !== fileReq.current) return;
        if (m.ok && !m.result.error) setFile((f) => ({ ...f, status: "done", result: m.result }));
        else setFile((f) => ({ ...f, status: "error", error: m.ok ? m.result.error : m.error }));
        return;
      }
      if (m.id !== reqId.current) return; // a newer request superseded this one
      if (m.ok) {
        setData(m.result);
        setElapsed(m.ms);
        setStatus("idle");
      } else {
        setError(m.error);
        setStatus("error");
      }
    };
  }, [worker]);

  // Debounce: re-run 250 ms after the last slider move.
  useEffect(() => {
    if (!worker) return;
    setStatus("running");
    const t = setTimeout(() => {
      reqId.current += 1;
      worker.postMessage({ id: reqId.current, params });
    }, 250);
    return () => clearTimeout(t);
  }, [params, worker]);

  const set = (k: keyof SimParams) => (v: number) => setParams((p) => ({ ...p, [k]: v }));
  const analyze = async (f: File | undefined) => {
    if (!f || !worker) return;
    if (f.size > MAX_FILE_BYTES) {
      setFile({ status: "error", name: f.name, error: s.fileTooBig(Math.round(MAX_FILE_BYTES / 1024 / 1024)) });
      return;
    }
    setFile({ status: "running", name: f.name });
    const text = await f.text();
    fileReq.current = reqId.current + 1_000_000 + fileReq.current + 1;
    worker.postMessage({ id: fileReq.current, file: text });
  };
  const toggleLang = () => {
    const next: Lang = lang === "pt" ? "en" : "pt";
    saveLang(next);
    setLang(next);
  };

  return (
    <div className="page">
      <header className="masthead">
        <div className="masthead-row">
          <h1>{s.title}</h1>
          <button type="button" className="lang" onClick={toggleLang} aria-label={s.langSwitchLabel}>
            {s.langSwitch}
          </button>
        </div>
        <p className="lede">{s.lede}</p>
      </header>

      <div className="layout">
        <aside className="controls" aria-label={s.controlsLabel}>
          <h2>{s.microTitle}</h2>
          <Slider id="win" label={s.window} value={WINDOWS.indexOf(params.windowMs)} min={0} max={WINDOWS.length - 1} step={1} show={`${params.windowMs / 1000} s`} onChange={(i) => set("windowMs")(WINDOWS[i])} />
          <Slider id="delay" label={s.delay} value={DELAYS.indexOf(params.maxDelayMs)} min={0} max={DELAYS.length - 1} step={1} show={`${params.maxDelayMs / 1000} s`} onChange={(i) => set("maxDelayMs")(DELAYS[i])} />
          <Slider id="liq" label={s.liqFraction} value={params.liquidityFraction} min={0.1} max={1} step={0.05} show={`${Math.round(params.liquidityFraction * 100)}% ${s.ofUB}`} onChange={set("liquidityFraction")} />
          <h2>{s.scenario}</h2>
          <Slider id="n" label={s.participants} value={params.participants} min={5} max={reduced ? 25 : 50} step={1} show={String(params.participants)} onChange={set("participants")} />
          <Slider id="p" label={s.payments} value={params.payments} min={5000} max={reduced ? 30000 : 100000} step={5000} show={params.payments.toLocaleString(loc)} onChange={set("payments")} />
          <Slider id="bias" label={s.bias} value={params.receiverBias} min={0} max={1} step={0.1} show={params.receiverBias.toFixed(1)} onChange={set("receiverBias")} />
          <div className="field">
            <label htmlFor="seed">{s.seed}</label>
            <input id="seed" type="number" value={params.seed} onChange={(e) => set("seed")(Number(e.target.value) || 0)} />
          </div>
          <p className={`status ${status}`} role="status" aria-live="polite">
            {status === "running" && s.running}
            {status === "idle" && data && s.done(data.payments.toLocaleString(loc), (elapsed / 1000).toFixed(1))}
            {status === "error" && s.failed(error)}
          </p>
          {reduced && <p className="note">{s.reduced}</p>}
        </aside>

        <main className="results">
          {data ? <Results data={data} params={params} s={s} /> : <p className="loading">{s.loading}</p>}
          <FilePanel s={s} file={file} onFile={analyze} loc={loc} />
        </main>
      </div>

      <footer className="foot">{s.footer(data ? Math.round(data.scale_factor).toLocaleString(loc) : "—")}</footer>
    </div>
  );
}

function Results({ data, params, s }: { data: SimResult; params: SimParams; s: Strings }) {
  const g = data.topology.gross;
  const micro = data.engines.find((e) => e.name.startsWith("micro_cycle_all"))!;
  const brl = (v: number) => fmtBRL(v, s.money);
  return (
    <>
      <section className="panel hero" aria-labelledby="net-h">
        <div className="panel-head">
          <h2 id="net-h">{s.netTitle}</h2>
          <p className="legend">
            <span className="key circ" /> {s.keyCirc} <span className="key dir" /> {s.keyDir}
          </p>
        </div>
        <Network data={data} ariaLabel={s.netAria(data.nodes.length)} />
        <p className="caption">{s.netCaption(fmtPct(g.cdfd_max))}</p>
      </section>

      <section className="panel" aria-labelledby="pv-h">
        <h2 id="pv-h">{s.pvTitle}</h2>
        <dl className="ladder">
          <div><dt>{s.pvPotential}</dt><dd>{fmtPct(data.potential_offset)}</dd></div>
          <div><dt>{s.pvCdfd}</dt><dd>{fmtPct(g.cdfd_max)}</dd></div>
          <div><dt>{s.pvCycles}</dt><dd>{fmtPct(g.cycle_netting)}</dd></div>
          <div><dt>{s.pvEod}</dt><dd>{fmtPct(data.potential_liquidity_saving)}</dd></div>
          <div className="realized"><dt>{s.pvMicro(params.maxDelayMs / 1000)}</dt><dd>{fmtPct(micro.liquidity_saving_vs_ub_pct / 100, 3)}</dd></div>
        </dl>
        <p className="caption">{s.pvCaption(fmtPct(micro.settled_cycle_value_brl / Math.max(micro.settled_value_brl, 1)))}</p>
      </section>

      <section className="panel" aria-labelledby="floor-h">
        <h2 id="floor-h">{s.floorTitle}</h2>
        <FloorLegend s={s} />
        <FloorChart points={data.floor} aria={s.floorAria} />
        <FloorTable points={data.floor} s={s} />
        <p className="caption">{s.floorCaption}</p>
      </section>

      <section className="panel" aria-labelledby="liq-h">
        <h2 id="liq-h">{s.liqTitle}</h2>
        <Bars engines={data.engines} ub={data.ub_brl} lb={data.lb_brl} s={s} brl={brl} />
      </section>

      <section className="panel" aria-labelledby="sw-h">
        <h2 id="sw-h">{s.sweepTitle}</h2>
        <SweepChart data={data} aria={s.sweepAria} />
        <p className="caption">{s.sweepCaption(params.maxDelayMs / 1000)}</p>
      </section>

      <section className="panel" aria-labelledby="day-h">
        <h2 id="day-h">{s.dayTitle}</h2>
        <DayChart data={data} aria={s.dayAria} money={s.money} />
        <p className="caption">{s.dayCaption}</p>
      </section>

      <section className="panel" aria-labelledby="c-h">
        <h2 id="c-h">{s.constrainedTitle(Math.round(params.liquidityFraction * 100))}</h2>
        <table className="tbl">
          <thead>
            <tr><th scope="col">{s.colDesign}</th><th scope="col">{s.colRejected}</th><th scope="col">{s.colP95}</th><th scope="col">{s.colWithin}</th></tr>
          </thead>
          <tbody>
            {data.constrained.map((e) => (
              <tr key={e.name}>
                <th scope="row">{label(s, e.name)}</th>
                <td>{fmtPct(e.rejected_value_brl / Math.max(e.rejected_value_brl + e.settled_value_brl, 1))}</td>
                <td>{(e.delay_p95_ms / 1000).toFixed(1)} s</td>
                <td>{fmtPct(e.share_settled_within_2800ms)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="caption">{s.constrainedCaption}</p>
      </section>

      <section className="panel" aria-labelledby="fit-h">
        <h2 id="fit-h">{s.fitTitle}</h2>
        <p className="caption">{s.fitIntro}</p>
        <ul className="fit">
          {s.fitRows.map((r) => (
            <li key={r.model}>
              <span className="fit-model">{r.model}</span>
              <span className="fit-engine">{r.engine}</span>
              <span className="fit-systems">{r.systems}</span>
            </li>
          ))}
        </ul>
        <p className="caption">{s.fitCaption}</p>
      </section>
    </>
  );
}

function Bars({ engines, ub, lb, s, brl }: { engines: EngineJson[]; ub: number; lb: number; s: Strings; brl: (v: number) => string }) {
  const rows = [...engines, { name: "lb", liquidity_needed_brl: lb } as EngineJson];
  return (
    <ul className="bars">
      {rows.map((e) => (
        <li key={e.name}>
          <span className="bar-label">{e.name === "lb" ? s.lb : label(s, e.name)}</span>
          <span className="bar-track"><span className={`bar ${e.name === "lb" ? "lbbar" : ""}`} style={{ width: `${(100 * e.liquidity_needed_brl) / Math.max(ub, 1)}%` }} /></span>
          <span className="bar-value">{brl(e.liquidity_needed_brl)}</span>
        </li>
      ))}
    </ul>
  );
}

function SweepChart({ data, aria }: { data: SimResult; aria: string }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const draw = () =>
      ref.current &&
      lineChart(ref.current, [{ label: "saving", color: color("--circular"), points: data.sweep.map((s, i) => [[0.5, 1, 2, 5, 10][i], s.saving_pct]) }], {
        xFmt: (v) => `${v.toFixed(1)} s`,
        yFmt: (v) => `${v.toFixed(3)}%`,
      });
    draw();
    window.addEventListener("resize", draw);
    return () => window.removeEventListener("resize", draw);
  }, [data]);
  return <canvas ref={ref} className="chart" role="img" aria-label={aria} />;
}

function DayChart({ data, aria, money }: { data: SimResult; aria: string; money: Strings["money"] }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const draw = () => {
      if (!ref.current) return;
      const v = data.intraday_value_brl;
      const d = data.intraday_net_debit_brl;
      const scale = Math.max(...d, 1) / Math.max(...v, 1);
      lineChart(
        ref.current,
        [
          { label: "value", color: color("--directional"), points: v.map((x, i) => [i / 4, x * scale]) },
          { label: "debit positions", color: color("--ink"), points: d.map((x, i) => [i / 4, x]) },
        ],
        { xFmt: (h) => `${Math.round(h)}h`, yFmt: (y) => fmtBRL(y, money) },
      );
    };
    draw();
    window.addEventListener("resize", draw);
    return () => window.removeEventListener("resize", draw);
    // money (the language's "mi"/"M" suffixes) is a stable object per language
  }, [data, money]);
  return <canvas ref={ref} className="chart" role="img" aria-label={aria} />;
}

// The liquidity floor curve: the most any mechanism with deadline d can save (tight bound) and
// what netting every d realises, in % of the liquidity of real-time gross settlement.
function FloorChart({ points, aria }: { points: FloorPoint[]; aria: string }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const draw = () =>
      ref.current &&
      lineChart(
        ref.current,
        [
          { label: "tight bound", color: color("--circular"), points: points.map((p) => [Math.log10(p.delay_s), p.tight_bound_pct]) },
          { label: "netting every d", color: color("--ink"), dashed: true, points: points.map((p) => [Math.log10(p.delay_s), p.netting_saving_pct]) },
        ],
        { xFmt: (v) => fmtDelay(10 ** v), yFmt: (v) => `${v.toFixed(2)}%`, yMin: 0 },
      );
    draw();
    window.addEventListener("resize", draw);
    return () => window.removeEventListener("resize", draw);
  }, [points]);
  return <canvas ref={ref} className="chart" role="img" aria-label={aria} />;
}

function FloorLegend({ s }: { s: Strings }) {
  return (
    <p className="legend">
      <span className="key circ" /> {s.floorTight} &nbsp; <span className="key dashed" /> {s.floorRealised}
    </p>
  );
}

function fmtDelay(sec: number): string {
  if (sec < 60) return `${Math.round(sec)} s`;
  if (sec < 3600) return `${Math.round(sec / 60)} min`;
  return `${Math.round(sec / 3600)} h`;
}

function FloorTable({ points, s }: { points: FloorPoint[]; s: Strings }) {
  return (
    <table className="tbl">
      <thead>
        <tr><th scope="col">{s.floorDelay}</th><th scope="col">{s.floorTight}</th><th scope="col">{s.floorRealised}</th></tr>
      </thead>
      <tbody>
        {points.map((p) => (
          <tr key={p.delay_s}>
            <th scope="row">{fmtDelay(p.delay_s)}</th>
            <td>{p.tight_bound_pct.toFixed(3)}%</td>
            <td>{p.netting_saving_pct.toFixed(3)}%</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// Analyse the user's own payment file in the browser. The file is read here and handed to the
// worker; nothing is uploaded.
function FilePanel({ s, file, onFile, loc }: { s: Strings; file: { status: string; name?: string; result?: FileResult; error?: string }; onFile: (f: File | undefined) => void; loc: string }) {
  const r = file.result;
  return (
    <section className="panel" aria-labelledby="file-h">
      <h2 id="file-h">{s.fileTitle}</h2>
      <p className="caption">{s.fileIntro}</p>
      <div className="field">
        <label htmlFor="payfile">{s.fileLabel}</label>
        <input id="payfile" type="file" accept=".csv,text/csv" onChange={(e) => onFile(e.target.files?.[0])} />
      </div>
      <p className="note">{s.filePrivacy}</p>
      <p className={`status ${file.status === "error" ? "error" : ""}`} role="status" aria-live="polite">
        {file.status === "running" && s.fileRunning(file.name ?? "")}
        {file.status === "error" && s.fileFailed(file.error ?? "")}
        {file.status === "done" && r && s.fileDone(r.payments.toLocaleString(loc), r.participants, r.span_hours.toFixed(1), r.rows_skipped + r.rows_skipped_status)}
      </p>
      {file.status === "done" && r && (
        <>
          <dl className="ladder">
            <div><dt>{s.fileUb}</dt><dd>{r.ub.toLocaleString(loc, { maximumFractionDigits: 2 })}</dd></div>
            <div><dt>{s.fileLb}</dt><dd>{r.lb.toLocaleString(loc, { maximumFractionDigits: 2 })}</dd></div>
          </dl>
          <FloorLegend s={s} />
          <FloorChart points={r.floor} aria={s.floorAria} />
          <FloorTable points={r.floor} s={s} />
        </>
      )}
    </section>
  );
}

