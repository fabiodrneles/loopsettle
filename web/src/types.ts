export interface SimParams {
  seed: number;
  participants: number;
  payments: number;
  windowMs: number;
  maxDelayMs: number;
  receiverBias: number;
  liquidityFraction: number;
}

export interface EngineJson {
  name: string;
  liquidity_needed_brl: number;
  liquidity_saving_vs_ub_pct: number;
  delay_p50_ms: number;
  delay_p95_ms: number;
  delay_p99_ms: number;
  share_settled_within_2800ms: number;
  settled_value_brl: number;
  settled_cycle_value_brl: number;
  rejected_value_brl: number;
  rejected_count: number;
}

export interface Ladder {
  hodge_l1_cyclic: number;
  hodge_l2_curl: number;
  cdfd_max: number;
  cycle_netting: number;
}

export interface SimResult {
  payments: number;
  gross_value_brl: number;
  ub_brl: number;
  lb_brl: number;
  scale_factor: number;
  potential_offset: number;
  potential_liquidity_saving: number;
  topology: { gross: Ladder; net: Ladder; bilateral_share: number };
  nodes: { id: number; class: string; activity: number; net_brl: number; ub_brl: number }[];
  edges: [number, number, number, number][]; // from, to, flow, circular part
  engines: EngineJson[];
  sweep: { name: string; saving_pct: number; p95_ms: number; cycle_value_share: number }[];
  constrained: EngineJson[];
  intraday_value_brl: number[];
  intraday_net_debit_brl: number[];
}
