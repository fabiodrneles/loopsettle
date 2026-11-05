import { useEffect, useRef, useState } from "react";
import { Network } from "./Network";
import { color, fmtBRL, fmtPct, lineChart } from "./canvas";
import type { EngineJson, SimParams, SimResult } from "./types";

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

const ENGINE_LABEL: Record<string, string> = {
  gross_real_time: "Bruto em tempo real (Pix)",
  deferred_net_1x: "Compensação 1× ao dia",
  deferred_net_96x: "Compensação a cada 15 min",
};
function label(name: string): string {
  if (ENGINE_LABEL[name]) return ENGINE_LABEL[name];
  if (name.startsWith("micro_cycle_all")) return "Micro-ciclos atômicos (fila total)";
  if (name.startsWith("micro_cycle_shortfall")) return "Micro-ciclos só na falta de saldo";
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
  const [params, setParams] = useState<SimParams>(DEFAULTS);
  const [data, setData] = useState<SimResult | null>(null);
  const [status, setStatus] = useState<"running" | "idle" | "error">("running");
  const [elapsed, setElapsed] = useState(0);
  const [error, setError] = useState("");
  const worker = useWorker();
  const reqId = useRef(0);

  useEffect(() => {
    if (!worker) return;
    worker.onmessage = (ev) => {
      const m = ev.data;
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

  return (
    <div className="page">
      <header className="masthead">
        <h1>Quanto do dinheiro do Pix só dá voltas?</h1>
        <p className="lede">
          Simulação sintética, calibrada com dados públicos do Banco Central (set/2026). Ajuste o desenho do sistema e veja
          quanta liquidez é circular, quanto disso dá para recuperar e quanto custa em atraso.
        </p>
      </header>

      <div className="layout">
        <aside className="controls" aria-label="Parâmetros da simulação">
          <h2>Micro-ciclos atômicos</h2>
          <Slider id="win" label="Janela de busca de ciclos" value={WINDOWS.indexOf(params.windowMs)} min={0} max={WINDOWS.length - 1} step={1} show={`${params.windowMs / 1000} s`} onChange={(i) => set("windowMs")(WINDOWS[i])} />
          <Slider id="delay" label="Atraso máximo tolerado" value={DELAYS.indexOf(params.maxDelayMs)} min={0} max={DELAYS.length - 1} step={1} show={`${params.maxDelayMs / 1000} s`} onChange={(i) => set("maxDelayMs")(DELAYS[i])} />
          <Slider id="liq" label="Saldo inicial no teste com liquidez restrita" value={params.liquidityFraction} min={0.1} max={1} step={0.05} show={`${Math.round(params.liquidityFraction * 100)}% do UB`} onChange={set("liquidityFraction")} />
          <h2>Cenário</h2>
          <Slider id="n" label="Participantes" value={params.participants} min={5} max={reduced ? 25 : 50} step={1} show={String(params.participants)} onChange={set("participants")} />
          <Slider id="p" label="Pagamentos simulados no dia" value={params.payments} min={5000} max={reduced ? 30000 : 100000} step={5000} show={params.payments.toLocaleString("pt-BR")} onChange={set("payments")} />
          <Slider id="bias" label="Desequilíbrio entre recebedores (hipótese)" value={params.receiverBias} min={0} max={1} step={0.1} show={params.receiverBias.toFixed(1)} onChange={set("receiverBias")} />
          <div className="field">
            <label htmlFor="seed">Semente</label>
            <input id="seed" type="number" value={params.seed} onChange={(e) => set("seed")(Number(e.target.value) || 0)} />
          </div>
          <p className={`status ${status}`} role="status" aria-live="polite">
            {status === "running" && "Simulando…"}
            {status === "idle" && data && `${data.payments.toLocaleString("pt-BR")} pagamentos em ${(elapsed / 1000).toFixed(1)} s`}
            {status === "error" && `A simulação falhou: ${error}. Recarregue a página para tentar de novo.`}
          </p>
          {reduced && <p className="note">Modo reduzido: cenários menores para caber no aparelho.</p>}
        </aside>

        <main className="results">{data ? <Results data={data} params={params} /> : <p className="loading">Carregando o simulador…</p>}</main>
      </div>

      <footer className="foot">
        LoopSettle 0.1 · Apache-2.0 · dados sintéticos calibrados, não dados interbancários reais. Cada pagamento simulado
        representa cerca de {data ? Math.round(data.scale_factor).toLocaleString("pt-BR") : "—"} pagamentos reais.
      </footer>
    </div>
  );
}

function Results({ data, params }: { data: SimResult; params: SimParams }) {
  const g = data.topology.gross;
  const micro = data.engines.find((e) => e.name.startsWith("micro_cycle_all"))!;
  return (
    <>
      <section className="panel hero" aria-labelledby="net-h">
        <div className="panel-head">
          <h2 id="net-h">A rede de um dia</h2>
          <p className="legend">
            <span className="key circ" /> parte circular máxima (CDFD) <span className="key dir" /> parte direcional
          </p>
        </div>
        <Network data={data} />
        <p className="caption">
          {fmtPct(g.cdfd_max)} do volume bruto poderia ser compensado em ciclos se a ordem no tempo não importasse. Círculos tracejados
          são pagadores líquidos.
        </p>
      </section>

      <section className="panel" aria-labelledby="pv-h">
        <h2 id="pv-h">Potencial vs realizável</h2>
        <dl className="ladder">
          <div><dt>Entradas que cobririam saídas no mesmo dia (medida potencial)</dt><dd>{fmtPct(data.potential_offset)}</dd></div>
          <div><dt>Teto estático de compensação em ciclos (CDFD)</dt><dd>{fmtPct(g.cdfd_max)}</dd></div>
          <div><dt>Compensação gulosa por ciclos</dt><dd>{fmtPct(g.cycle_netting)}</dd></div>
          <div><dt>Liquidez poupável com uma compensação no fim do dia</dt><dd>{fmtPct(data.potential_liquidity_saving)}</dd></div>
          <div className="realized"><dt>Liquidez poupada de fato pelos micro-ciclos ({params.maxDelayMs / 1000} s)</dt><dd>{fmtPct(micro.liquidity_saving_vs_ub_pct / 100, 3)}</dd></div>
        </dl>
        <p className="caption">
          Os micro-ciclos liquidam {fmtPct(micro.settled_cycle_value_brl / Math.max(micro.settled_value_brl, 1))} do valor dentro de ciclos, mas o pico de liquidez quase não muda: ele vem de desequilíbrios que duram horas, não segundos.
        </p>
      </section>

      <section className="panel" aria-labelledby="liq-h">
        <h2 id="liq-h">Liquidez necessária por desenho</h2>
        <Bars engines={data.engines} ub={data.ub_brl} lb={data.lb_brl} />
      </section>

      <section className="panel" aria-labelledby="sw-h">
        <h2 id="sw-h">Economia × janela</h2>
        <SweepChart data={data} />
        <p className="caption">Economia de liquidez dos micro-ciclos em relação ao bruto, para cada janela, com atraso máximo de {params.maxDelayMs / 1000} s.</p>
      </section>

      <section className="panel" aria-labelledby="day-h">
        <h2 id="day-h">O dia em 15 minutos</h2>
        <DayChart data={data} />
        <p className="caption">Valor pago por intervalo e soma das posições devedoras acumuladas (o que exige liquidez no bruto).</p>
      </section>

      <section className="panel" aria-labelledby="c-h">
        <h2 id="c-h">Com liquidez restrita ({Math.round(params.liquidityFraction * 100)}% do UB)</h2>
        <table className="tbl">
          <thead>
            <tr><th scope="col">Desenho</th><th scope="col">Valor rejeitado</th><th scope="col">Atraso p95</th><th scope="col">Até 2,8 s</th></tr>
          </thead>
          <tbody>
            {data.constrained.map((e) => (
              <tr key={e.name}>
                <th scope="row">{label(e.name)}</th>
                <td>{fmtPct(e.rejected_value_brl / Math.max(e.rejected_value_brl + e.settled_value_brl, 1))}</td>
                <td>{(e.delay_p95_ms / 1000).toFixed(1)} s</td>
                <td>{fmtPct(e.share_settled_within_2800ms)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="caption">A meta do Manual de Tempos do Pix é liquidar 50% em até 2,8 s.</p>
      </section>
    </>
  );
}

function Bars({ engines, ub, lb }: { engines: EngineJson[]; ub: number; lb: number }) {
  const rows = [...engines, { name: "lb", liquidity_needed_brl: lb } as EngineJson];
  return (
    <ul className="bars">
      {rows.map((e) => (
        <li key={e.name}>
          <span className="bar-label">{e.name === "lb" ? "Mínimo teórico (LB)" : label(e.name)}</span>
          <span className="bar-track"><span className={`bar ${e.name === "lb" ? "lbbar" : ""}`} style={{ width: `${(100 * e.liquidity_needed_brl) / Math.max(ub, 1)}%` }} /></span>
          <span className="bar-value">{fmtBRL(e.liquidity_needed_brl)}</span>
        </li>
      ))}
    </ul>
  );
}

function SweepChart({ data }: { data: SimResult }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const draw = () =>
      ref.current &&
      lineChart(ref.current, [{ label: "economia", color: color("--circular"), points: data.sweep.map((s, i) => [[0.5, 1, 2, 5, 10][i], s.saving_pct]) }], {
        xFmt: (v) => `${v.toFixed(1)} s`,
        yFmt: (v) => `${v.toFixed(3)}%`,
      });
    draw();
    window.addEventListener("resize", draw);
    return () => window.removeEventListener("resize", draw);
  }, [data]);
  return <canvas ref={ref} className="chart" role="img" aria-label="Economia de liquidez por janela de busca de ciclos" />;
}

function DayChart({ data }: { data: SimResult }) {
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
          { label: "valor", color: color("--directional"), points: v.map((x, i) => [i / 4, x * scale]) },
          { label: "posições devedoras", color: color("--ink"), points: d.map((x, i) => [i / 4, x]) },
        ],
        { xFmt: (h) => `${Math.round(h)}h`, yFmt: (y) => fmtBRL(y) },
      );
    };
    draw();
    window.addEventListener("resize", draw);
    return () => window.removeEventListener("resize", draw);
  }, [data]);
  return <canvas ref={ref} className="chart" role="img" aria-label="Valor pago e posições devedoras ao longo do dia" />;
}
