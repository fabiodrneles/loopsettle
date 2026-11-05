// UI strings in Portuguese and English. The language comes from ?lang=pt|en, then from the
// viewer's last choice (localStorage), then from the browser language (pt* -> pt, else en).

export type Lang = "pt" | "en";

export function detectLang(): Lang {
  try {
    const q = new URLSearchParams(window.location.search).get("lang");
    if (q === "pt" || q === "en") return q;
  } catch {
    /* ignore */
  }
  try {
    const saved = window.localStorage.getItem("loopsettle-lang");
    if (saved === "pt" || saved === "en") return saved;
  } catch {
    /* storage may be blocked */
  }
  return (navigator.language || "en").toLowerCase().startsWith("pt") ? "pt" : "en";
}

export function saveLang(lang: Lang) {
  try {
    window.localStorage.setItem("loopsettle-lang", lang);
  } catch {
    /* storage may be blocked */
  }
}

export const LOCALE: Record<Lang, string> = { pt: "pt-BR", en: "en-US" };

const pt = {
  title: "Quanto do dinheiro do Pix só dá voltas?",
  lede:
    "Simulação sintética, calibrada com dados públicos do Banco Central (set/2026). Ajuste o desenho do sistema e veja quanta liquidez é circular, quanto disso dá para recuperar e quanto custa em atraso.",
  langSwitch: "English",
  langSwitchLabel: "Mudar o idioma para inglês",
  controlsLabel: "Parâmetros da simulação",
  microTitle: "Micro-ciclos atômicos",
  window: "Janela de busca de ciclos",
  delay: "Atraso máximo tolerado",
  liqFraction: "Saldo inicial no teste com liquidez restrita",
  ofUB: "do UB",
  scenario: "Cenário",
  participants: "Participantes",
  payments: "Pagamentos simulados no dia",
  bias: "Desequilíbrio entre recebedores (hipótese)",
  seed: "Semente",
  running: "Simulando…",
  done: (n: string, s: string) => `${n} pagamentos em ${s} s`,
  failed: (e: string) => `A simulação falhou: ${e}. Recarregue a página para tentar de novo.`,
  reduced: "Modo reduzido: cenários menores para caber no aparelho.",
  loading: "Carregando o simulador…",
  footer: (k: string) =>
    `LoopSettle 0.1 · Apache-2.0 · dados sintéticos calibrados, não dados interbancários reais. Cada pagamento simulado representa cerca de ${k} pagamentos reais.`,
  netTitle: "A rede de um dia",
  keyCirc: "parte circular máxima (CDFD)",
  keyDir: "parte direcional",
  netCaption: (p: string) =>
    `${p} do volume bruto poderia ser compensado em ciclos se a ordem no tempo não importasse. Círculos tracejados são pagadores líquidos.`,
  netAria: (n: number) => `Rede de ${n} participantes; a cor de destaque mostra o componente circular máximo (CDFD) de cada fluxo.`,
  pvTitle: "Potencial vs realizável",
  pvPotential: "Entradas que cobririam saídas no mesmo dia (medida potencial)",
  pvCdfd: "Teto estático de compensação em ciclos (CDFD)",
  pvCycles: "Compensação gulosa por ciclos",
  pvEod: "Liquidez poupável com uma compensação no fim do dia",
  pvMicro: (s: number) => `Liquidez poupada de fato pelos micro-ciclos (${s} s)`,
  pvCaption: (p: string) =>
    `Os micro-ciclos liquidam ${p} do valor dentro de ciclos, mas o pico de liquidez quase não muda: ele vem de desequilíbrios que duram horas, não segundos.`,
  liqTitle: "Liquidez necessária por desenho",
  lb: "Mínimo teórico (LB)",
  sweepTitle: "Economia × janela",
  sweepCaption: (s: number) => `Economia de liquidez dos micro-ciclos em relação ao bruto, para cada janela, com atraso máximo de ${s} s.`,
  sweepAria: "Economia de liquidez por janela de busca de ciclos",
  dayTitle: "O dia em 15 minutos",
  dayCaption: "Valor pago por intervalo e soma das posições devedoras acumuladas (o que exige liquidez no bruto).",
  dayAria: "Valor pago e posições devedoras ao longo do dia",
  constrainedTitle: (p: number) => `Com liquidez restrita (${p}% do UB)`,
  colDesign: "Desenho",
  colRejected: "Valor rejeitado",
  colP95: "Atraso p95",
  colWithin: "Até 2,8 s",
  constrainedCaption: "A meta do Manual de Tempos do Pix é liquidar 50% em até 2,8 s.",
  engines: {
    gross_real_time: "Bruto em tempo real (Pix)",
    deferred_net_1x: "Compensação 1× ao dia",
    deferred_net_96x: "Compensação a cada 15 min",
    micro_all: "Micro-ciclos atômicos (fila total)",
    micro_shortfall: "Micro-ciclos só na falta de saldo",
  },
  money: { bn: "bi", mn: "mi", k: "mil" },
};

export type Strings = typeof pt;

const en: Strings = {
  title: "How much of Pix money just goes around in loops?",
  lede:
    "Synthetic simulation calibrated to public Banco Central do Brasil data (Sep 2026). Change the system design and see how much liquidity is circular, how much of it can be recovered and what it costs in delay.",
  langSwitch: "Português",
  langSwitchLabel: "Switch language to Portuguese",
  controlsLabel: "Simulation parameters",
  microTitle: "Atomic micro-cycles",
  window: "Cycle search window",
  delay: "Maximum tolerated delay",
  liqFraction: "Opening balance in the constrained-liquidity test",
  ofUB: "of UB",
  scenario: "Scenario",
  participants: "Participants",
  payments: "Simulated payments per day",
  bias: "Receiver imbalance (assumption)",
  seed: "Seed",
  running: "Simulating…",
  done: (n: string, s: string) => `${n} payments in ${s} s`,
  failed: (e: string) => `The simulation failed: ${e}. Reload the page to try again.`,
  reduced: "Reduced mode: smaller scenarios to fit this device.",
  loading: "Loading the simulator…",
  footer: (k: string) =>
    `LoopSettle 0.1 · Apache-2.0 · calibrated synthetic data, not real interbank data. Each simulated payment stands for about ${k} real payments.`,
  netTitle: "The network of one day",
  keyCirc: "maximum circular part (CDFD)",
  keyDir: "directional part",
  netCaption: (p: string) =>
    `${p} of gross volume could be netted in cycles if time order did not matter. Dashed circles are net payers.`,
  netAria: (n: number) => `Network of ${n} participants; the accent colour shows the maximum circular component (CDFD) of each flow.`,
  pvTitle: "Potential vs realizable",
  pvPotential: "Outflows covered by same-day inflows (potential measure)",
  pvCdfd: "Static ceiling for cycle netting (CDFD)",
  pvCycles: "Greedy cycle netting",
  pvEod: "Liquidity saved by one end-of-day netting",
  pvMicro: (s: number) => `Liquidity actually saved by micro-cycles (${s} s)`,
  pvCaption: (p: string) =>
    `Micro-cycles settle ${p} of value inside cycles, yet the liquidity peak barely moves: it comes from imbalances lasting hours, not seconds.`,
  liqTitle: "Liquidity needed by design",
  lb: "Theoretical minimum (LB)",
  sweepTitle: "Saving × window",
  sweepCaption: (s: number) => `Micro-cycle liquidity saving relative to gross settlement, for each window, with a ${s} s maximum delay.`,
  sweepAria: "Liquidity saving by cycle search window",
  dayTitle: "The day in 15-minute steps",
  dayCaption: "Value paid per interval and the sum of accumulated debit positions (what gross settlement must fund).",
  dayAria: "Value paid and debit positions over the day",
  constrainedTitle: (p: number) => `With constrained liquidity (${p}% of UB)`,
  colDesign: "Design",
  colRejected: "Value rejected",
  colP95: "p95 delay",
  colWithin: "Within 2.8 s",
  constrainedCaption: "The Pix timing target (Manual de Tempos) is to settle 50% within 2.8 s.",
  engines: {
    gross_real_time: "Real-time gross (Pix)",
    deferred_net_1x: "Netting once a day",
    deferred_net_96x: "Netting every 15 min",
    micro_all: "Atomic micro-cycles (queue all)",
    micro_shortfall: "Micro-cycles on shortfall only",
  },
  money: { bn: "bn", mn: "M", k: "k" },
};

export const STRINGS: Record<Lang, Strings> = { pt, en };
