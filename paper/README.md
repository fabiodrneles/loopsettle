# JOSS paper — working folder

> [Versão em português](README.pt-BR.md)

| File | What it is |
|---|---|
| [`paper.pt-BR.md`](paper.pt-BR.md) | **Portuguese draft for the author to read** (not sent to JOSS) |
| `paper.md` | English version submitted to JOSS — to be written from the Portuguese draft after the author's review |
| [`paper.bib`](paper.bib) | the paper's bibliography (a subset of [`../docs/references/references.bib`](../docs/references/references.bib) plus the software cited) |

The format follows the official JOSS documentation (read on 5 Nov 2026): YAML metadata; required
sections **Summary, Statement of need, State of the field, Software design, Research impact
statement, AI usage disclosure, Acknowledgements, References**; 750 to 1,750 words (the draft has
about 1,350); no API documentation in the text. JOSS compiles the PDF with Pandoc (Docker
`openjournals/inara` or the `openjournals/openjournals-draft-action` GitHub Action), to be set up
together with `paper.md`.

## When we can submit: from May 2027

JOSS desk-rejects, before review, any submission that fails one of these four criteria. Status of
LoopSettle today:

| JOSS criterion (required) | Status on 5 Nov 2026 | What is missing |
|---|---|---|
| 1. Public repository for **more than six months**, with active development over that period | created on 5 Nov 2026; 55 commits, all on the same day | wait until **at least 6 May 2027**, developing in public until then |
| 2. **Demonstrated research impact** (papers or preprints using the software; use by others) | author's own use only, no preprint yet | publish the **preprint** first (it becomes the evidence); seek at least one external user (a central bank, a researcher) |
| 3. Good open-source practice (for a single-author project: history over time, releases or a changelog, tests and CI, documentation, CONTRIBUTING, support expectations) | tests, CI and documentation in place; releases and CONTRIBUTING missing | add `CONTRIBUTING`, `CHANGELOG`, the support statement and the **v0.1.0 tag** now, and new releases at each relevant change |
| 4. **Iterative development** over time (not a single burst of commits) | a single burst | use issues and PRs for each improvement over the coming months (e.g. proportional CDFD, strict FIFO, on-chain validation, answers to preprint readers) |

Consequence for the order of the papers: **the preprint first**, then JOSS. A preprint is not a
prior publication for JOSS, and it is exactly the impact evidence criterion 2 asks for.

## What only the author can fill in

- **ORCID** and **affiliation** (marked `[PREENCHER]` today).
- **AI usage disclosure:** JOSS requires the tools and versions, where they were used, the nature
  of the assistance and a confirmation that a human reviewed and validated everything and made the
  core design decisions. An incomplete disclosure is treated as an ethical breach. Review the
  draft's text word by word and add the model and version.
- **Conflicts of interest and funding:** JOSS asks authors to disclose any relevant professional
  relationship (including employment in payments) and any financial support.
- **Someone who tested the installation:** for new software, JOSS asks that a colleague has
  installed and tried it.

## Points to watch in the text

- The research impact statement is the weakest section today, because it depends on the preprint.
- The state of the field cites BoF-PSS2 (free for research, closed source, under licence) and
  PSSimPy (Python, large-value; no licence shown on PyPI). Check both before submitting.
- PSSimPy's authors in `paper.bib` come from PyPI (Kenneth See and Hanzholah Shobri); check them
  against the ICIS 2024 paper.
