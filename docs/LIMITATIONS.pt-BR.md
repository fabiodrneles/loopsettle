# Limitações conhecidas

> [English version](LIMITATIONS.md)

1. **Dados sintéticos.** Nenhum dado interbancário real é usado. O gerador reproduz agregados
   públicos (volume, valor, ticket médio, participação do B2B, concentração por chaves), mas
   não a estrutura bilateral nem a dinâmica intradiária reais.
2. **Estrutura bilateral é hipótese.** O modelo gravitacional com viés recebedor (D9) decide
   quanto do fluxo é recíproco e quanto é direcional. A razão LB/UB, a fração circular e todas
   as economias dependem dele (sensibilidade em `results/baseline/sensitivity.csv`).
3. **Tamanho ∝ chaves Pix.** A participação por instituição usa chaves por ISPB como proxy de
   transações (D11). Bancos digitais podem ter muitas chaves por cliente.
4. **Escala.** 200 participantes e 1 milhão de pagamentos/dia (fator 239). O número de
   pagamentos por janela de micro-ciclo é ~239× menor que no Pix real; a chance de achar
   ciclos numa janela curta deve crescer com a escala real. A liquidez absoluta (R$) não
   escala linearmente.
5. **Perfil horário e rajadas.** O perfil intradiário é uma hipótese (sem série horária
   pública usada) e o processo de chegada não tem rajadas (Hawkes, dias de salário).
6. **Um dia.** O período de análise é um dia; não há efeitos entre dias nem saldos de abertura
   reais.
7. **Micro-ciclos heurísticos.** Uma cabeça de fila por aresta, ciclos de até 6, bloqueio
   simples em caso de falta de saldo (D12). Um otimizador completo de gridlock pode liquidar
   mais.
8. **Atraso medido é só o atraso adicionado pelo motor.** O tempo de processamento real do
   SPI/PSPs (que entra na meta de 2,8 s do Manual de Tempos) não é modelado.
9. **CDFD proporcional não implementada** (D6); a escada usa só a CDFD máxima.
10. **Validação externa incompleta.** O comando `topology <csv>` está pronto, mas a amostra
    on-chain de stablecoins não foi baixada (Fase 8).
11. **Hodge em grafos densos.** A fração rotacional de Hodge no grafo líquido (≈0,98) é
    dominada pela dimensão do espaço de ciclos (m − n + 1 ≫ n − 1): ruído aleatório projeta-se
    quase todo em ciclos. É um diagnóstico de forma, não um montante compensável.
12. **Interface web verificada só em navegador emulado.** Em 05/11/2026 foi aberta no Chromium
    desktop (1440 px) e num iPhone 13 emulado, em português e inglês (capturas em `docs/img/`).
    Ainda não foi aberta no notebook de referência (i3, 4 GB).
13. **Calibrado com um só sistema.** Só agregados do Pix são usados. Há motores para os modelos de
    liquidação de outros sistemas (FedNow, TIPS, NPP, UPI, Faster Payments — ver
    [`INSTANT_PAYMENTS.pt-BR.md`](INSTANT_PAYMENTS.pt-BR.md)), mas nenhum outro sistema foi
    calibrado; valores, perfil horário e concentração mudam de país para país.
14. **Chegadas estacionárias.** Um perfil horário único para todos os participantes e pesos
    bilaterais constantes (D8, D9). Com essas hipóteses, a economia da compensação de fim de dia
    encolhe com 1/√(volume) (`results/robustness/scale.csv`); desequilíbrios intradiários
    sistemáticos e próprios de cada participante (folha de pagamento, credenciadoras, funding de
    fintechs) não encolheriam. Ver [`PAPER_READINESS.pt-BR.md`](PAPER_READINESS.pt-BR.md), W2.
    *Atualização (W2):* os perfis por classe agora existem (`profile_strength`, D18); com eles o
    potencial deixa de desabar (2,8% com 30 milhões e o σ calibrado), mas os formatos são hipóteses
    até que se usem dados intradiários por participante.
