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
  title: "Quanto da liquidez dos pagamentos instantâneos só dá voltas?",
  lede:
    "Simulação sintética de um sistema de pagamentos instantâneos com liquidação bruta em tempo real — o modelo do Pix, do FedNow, do TIPS e do NPP —, calibrada com dados públicos do Pix (Brasil, set/2026). Ajuste o desenho do sistema e veja quanta liquidez é circular, quanto disso dá para recuperar e quanto custa em atraso.",
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
    `LoopSettle 0.2 · Apache-2.0 · dados sintéticos calibrados com agregados públicos do Pix, não dados interbancários reais. Cada pagamento simulado representa cerca de ${k} pagamentos reais do Pix. Pix é marca do Banco Central do Brasil; o LoopSettle é independente, sem afiliação nem endosso do BCB.`,
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
  floorTitle: "O teto: quanto qualquer mecanismo com prazo d pode economizar",
  floorAria: "Limite apertado de economia e economia obtida pela compensação a cada d, em função do prazo d",
  floorDelay: "prazo d",
  floorTight: "teto (qualquer mecanismo)",
  floorRealised: "obtido compensando a cada d",
  floorCaption:
    "O teto vem do piso de liquidez de cada participante (o maior valor de \"tudo o que pagou até um instante, menos tudo o que recebeu até d depois\"). Vale para qualquer algoritmo com esse prazo. Em % da liquidez da liquidação bruta em tempo real.",
  fileTitle: "Analisar o seu arquivo de pagamentos",
  fileIntro: "CSV com as colunas time (segundos, HH:MM:SS ou t_ms em milissegundos), from, to e amount; uma coluna status opcional (só os liquidados contam). As linhas não precisam estar em ordem.",
  fileLabel: "Escolher arquivo",
  filePrivacy: "O arquivo é lido e analisado neste navegador. Nada é enviado para fora deste computador.",
  fileRunning: (n: string) => `Analisando ${n}…`,
  fileFailed: (e: string) => `Não foi possível analisar: ${e}`,
  fileTooBig: (mb: number) => `arquivo maior que ${mb} MB; use \`loopsettle bound\` na linha de comando`,
  fileDone: (p: string, n: number, h: string, skipped: number) => `${p} pagamentos entre ${n} participantes em ${h} h (${skipped} linhas ignoradas).`,
  fileUb: "Liquidez para liquidar tudo na hora (UB), nas unidades do arquivo",
  fileLb: "Com uma compensação no fim do período (LB)",
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
  constrainedCaption: "Referência: a meta do Pix é liquidar 50% dos pagamentos em até 2,8 s; o TIPS (zona do euro) liquida cada pagamento em menos de 10 s.",
  engines: {
    gross_real_time: "Bruto em tempo real (Pix, FedNow, TIPS)",
    deferred_net_1x: "Compensação 1× ao dia",
    deferred_net_96x: "Compensação a cada 15 min",
    micro_all: "Micro-ciclos atômicos (fila total)",
    micro_shortfall: "Micro-ciclos só na falta de saldo",
  },
  money: { bn: "bi", mn: "mi", k: "mil" },
  fitTitle: "Onde o seu sistema se encaixa",
  fitIntro: "Para o cliente, todos são instantâneos. O que muda a liquidez é como os bancos acertam as contas entre si:",
  fitRows: [
    { model: "Liquidação bruta em tempo real", engine: "motor “Bruto em tempo real”", systems: "Pix (Brasil), FedNow (EUA), TIPS (zona do euro), NPP (Austrália)" },
    { model: "Bruta, com conta pré-financiada", engine: "motor “Bruto” com saldo limitado", systems: "RTP (EUA), Real-Time Rail (Canadá, em lançamento)" },
    { model: "Compensação diferida em ciclos", engine: "motor “Compensação N× ao dia”", systems: "UPI (Índia, 10 ciclos por dia), Faster Payments (Reino Unido, 3 por dia), FAST (Singapura, 2 por dia), Interac e-Transfer (Canadá, 1 por dia)" },
    { model: "Compensação a cada poucos segundos", engine: "motor “Micro-ciclos”", systems: "SPEI (México), em média a cada 1,9 s" },
  ],
  fitCaption: "Detalhes e fontes de cada país no documento INSTANT_PAYMENTS do repositório. Os números desta tela vêm de um cenário calibrado com o Pix.",
};

export type Strings = typeof pt;

const en: Strings = {
  title: "How much instant-payment liquidity just goes around in loops?",
  lede:
    "Synthetic simulation of an instant payment system with real-time gross settlement — the model used by Pix, FedNow, TIPS and NPP — calibrated to public data from Brazil's Pix (Sep 2026). Change the system design and see how much liquidity is circular, how much of it can be recovered and what it costs in delay.",
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
    `LoopSettle 0.2 · Apache-2.0 · synthetic data calibrated to public Pix aggregates, not real interbank data. Each simulated payment stands for about ${k} real Pix payments. Pix is a trademark of Banco Central do Brasil; LoopSettle is independent and not affiliated with or endorsed by the BCB.`,
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
  floorTitle: "The ceiling: what any mechanism with deadline d can save",
  floorAria: "Tight bound on the saving and saving realised by netting every d, as a function of the deadline d",
  floorDelay: "deadline d",
  floorTight: "ceiling (any mechanism)",
  floorRealised: "realised by netting every d",
  floorCaption:
    "The ceiling comes from each participant's liquidity floor (the largest value of \"everything it has paid by some instant, minus everything it has received by d later\"). It holds for any algorithm with that deadline. In % of the liquidity of real-time gross settlement.",
  fileTitle: "Analyse your own payment file",
  fileIntro: "CSV with the columns time (seconds, HH:MM:SS, or t_ms in milliseconds), from, to and amount; an optional status column (only settled rows count). Rows need not be in order.",
  fileLabel: "Choose file",
  filePrivacy: "The file is read and analysed in this browser. Nothing leaves this computer.",
  fileRunning: (n: string) => `Analysing ${n}…`,
  fileFailed: (e: string) => `Could not analyse the file: ${e}`,
  fileTooBig: (mb: number) => `file larger than ${mb} MB; use \`loopsettle bound\` on the command line`,
  fileDone: (p: string, n: number, h: string, skipped: number) => `${p} payments among ${n} participants over ${h} h (${skipped} rows skipped).`,
  fileUb: "Liquidity to settle everything on arrival (UB), in the file's units",
  fileLb: "With one netting at the end of the period (LB)",
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
  constrainedCaption: "Reference: Pix targets 50% of payments settled within 2.8 s; TIPS (euro area) settles each payment in under 10 s.",
  engines: {
    gross_real_time: "Real-time gross (Pix, FedNow, TIPS)",
    deferred_net_1x: "Netting once a day",
    deferred_net_96x: "Netting every 15 min",
    micro_all: "Atomic micro-cycles (queue all)",
    micro_shortfall: "Micro-cycles on shortfall only",
  },
  money: { bn: "bn", mn: "M", k: "k" },
  fitTitle: "Where your system fits",
  fitIntro: "To customers they are all instant. What changes liquidity is how banks settle with each other:",
  fitRows: [
    { model: "Real-time gross settlement", engine: "“Real-time gross” engine", systems: "Pix (Brazil), FedNow (US), TIPS (euro area), NPP (Australia)" },
    { model: "Gross, against a prefunded account", engine: "“Gross” engine with limited balance", systems: "RTP (US), Real-Time Rail (Canada, launching)" },
    { model: "Deferred netting in cycles", engine: "“Netting N× a day” engine", systems: "UPI (India, 10 cycles a day), Faster Payments (UK, 3 a day), FAST (Singapore, 2 a day), Interac e-Transfer (Canada, 1 a day)" },
    { model: "Netting every few seconds", engine: "“Micro-cycles” engine", systems: "SPEI (Mexico), every 1.9 s on average" },
  ],
  fitCaption: "Details and sources for each country in the repository's INSTANT_PAYMENTS document. The numbers on this screen come from a scenario calibrated to Pix.",
};

export const STRINGS: Record<Lang, Strings> = { pt, en };
