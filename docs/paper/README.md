# Compression and Verification

**Layered Emergence, Structural Convergence, and Verifiable Execution in Complex Software Systems**

*The Compression–Assurance Working Notes* — an independent technical report.
<https://github.com/ghwinter/assay>

---

## Layout

```
paper/
├── en/                     AUTHORITATIVE TEXT
│   ├── main.tex            book class, title page, abstract, \input chain
│   ├── preamble.tex        packages, theorem environments, macros
│   ├── ch01 … ch12         twelve chapters
│   ├── appA … appC         proofs, templates, glossary
│   ├── references.tex      manual thebibliography (no BibTeX needed)
│   └── build/main.pdf      106 pp.
├── zh/                     CONTROLLED RENDERING of en/
│   └── (same 18 files)     build/main.pdf  96 pp.
├── TERMINOLOGY.md          the binding term mapping between the two builds
├── build.ps1               builds both variants
└── README.md
```

**The English build is the standard.** The Chinese build is a rendering of it, and the
one-to-one term mapping is fixed in `TERMINOLOGY.md`. Where a Chinese word would otherwise
carry two meanings, the Chinese is split and the split is recorded there — for example
`确认` is reserved for *validation*, while *attestation* and *ascertain* are rendered as
`审定` and `核实`.

## Build

Requires **XeLaTeX** (a full TeX Live or MiKTeX installation) with `ctex` for the Chinese
variant.

```powershell
pwsh -File build.ps1            # both variants
pwsh -File build.ps1 -Lang en   # English only
pwsh -File build.ps1 -Lang zh   # Chinese only
```

Manually, from either language directory:

```bash
xelatex -interaction=nonstopmode -output-directory=build main.tex   # x3
```

Three passes are required to settle the table of contents, cross-references and the
bibliography. No BibTeX or Biber run is needed: the bibliography is a manual
`thebibliography` environment.

`build.ps1` locates `xelatex` in the usual MiKTeX and TeX Live locations if it is not on
`PATH`.

Both variants currently build clean: **0 errors, 0 undefined references, 0 missing
characters.**

## Contents

| | Chapter | Subject |
|---|---|---|
| 1 | Introduction | four questions; the four theses; epistemic discipline; contributions |
| 2 | Formal framework | levels, Galois connections, **oblivious sets**, contracts, traces, TCB, **contract operads** |
| 3 | Structural convergence | three graded propositions; over- and under-constraint; mechanisms; strength law |
| 4 | Internal infrastructuralisation | integration-cost theorem; **conditional necessity of the six functions**; transport vs. re-derivation; coarse-graining criterion |
| 5 | Comprehension | inverse compression; cognitive maps; descent protocol; **observability-rank criterion** |
| 6 | Design | forward compression; **invariant centralisation**; leak pairs; technical debt; **governable complexity** |
| 7 | Intent to verified execution | five dimensions of correctness; refinement chain; intent gap; **consensus bound and incentives** |
| 8 | Levels as obligation units | L0…L∞; assurance allocation matrix |
| 9 | Research programme | fourteen falsifiable hypotheses; three directions; six-phase roadmap |
| 10 | Literature map | six groups; five layers; the vacant intersection |
| 11 | Objections and limits | list of non-claims; arguments removed from the load-bearing structure |
| 12 | Conclusion | answers; compression as the single thread; core proposition |

## Formal results

1. **Oblivious set** `Obl(Λ) = ker α` — "what this level need not care about" becomes
   computable; abstraction is quotienting by the kernel. With Dennett's real-pattern
   criterion, the existence of a level reduces to a compression ratio.
2. **Contract operad** — composability of local guarantees reduces to *well-typed
   colouring* and *acyclicity of the interface graph*; this explains why retry and
   feedback invalidate existing composition arguments.
3. **Level family and transport** — separates cross-domain isomorphism of the six
   functions from transport along problem shape, with a decidable test.
4. **Integration cost** — `Θ(nm) → O(nk+m)`; the threshold is set by *entities × shared
   resources*, not by lines of code.
5. **Six functions** — conditional necessity from well-definedness of the operational
   semantics; they may be built or adopted, but not omitted.
6. **Coarse-graining criterion** — makes the convergence thesis falsifiable inside a
   single project, without cross-project statistics.
7. **Invariant centralisation** — `|ρ(c)| ≥ |S(ι)|`; minimising the expected read-set
   requires single-site enforcement.
8. **Near-decomposability as interface sufficiency**, yielding the **leak-pair**
   diagnostic: whether an abstraction leaks is measurable, not merely felt.
9. **Observability rank** — `rank_T J = dim T` is necessary and sufficient for
   comprehension to converge; separates *insufficient observation* (a design action) from
   *insufficient sampling* (a probing action).
10. **Governable complexity** `C = E[ρ]·λ` against capacity `K = hb`: stability `C ≤ K`,
    and architecture fixes the scaling exponent `h ∝ n^{1+α}`.
11. **Technical debt as a description-length gap**, with a rate–distortion reading of
    refactoring.
12. **Consensus reliability bound** `≤ exp(−2n(½−p)²)`, dominated by a correlated error
    mode `q`, which is in turn reduced to an incentive-design problem.
13. **Consolidated assurance condition** — two-layer trust plus artefact binding; and the
    observation that "verification is cheap" must be split into **checking time** and
    **certificate size**.

## Explicit non-claims

See Chapter 11. In brief: the report declines to use Gödel incompleteness or Rice's theorem
to establish the intent gap (epistemic grounds are given instead, and the gap is identified
as the **frame problem**); declines to use the open status of P versus NP to support
"verification is cheaper than generation"; declines to treat Simon's near-decomposability as
a universal law; declines "a minimal verifier is therefore absolutely reliable"; declines
"all large systems converge on one architecture"; and declines analogies to the **Langlands
programme** or **Landauer's principle** as load-bearing structure.

## Licence

The report text — the LaTeX sources in `en/` and `zh/`, the compiled PDFs, and the
prose in `README.md` and `TERMINOLOGY.md` — is licensed under the
**Creative Commons Attribution 4.0 International Licence (CC-BY-4.0)**; the full
legal code is in [LICENSE](LICENSE). You may share and adapt it for any purpose,
including commercially, provided you give appropriate credit, link to the licence,
and indicate if changes were made.

Copyright 2026 ghwinter.

`build.ps1` and the repository tooling outside this directory are dual-licensed
**MIT OR Apache-2.0**; see the repository root.
