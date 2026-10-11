# Machine-checked claims

`claims.lean` contains the machine-checked core of the formal claims made in
Chapters 2 and 7. It is cited in the paper as the accompanying development, and it exists so that
those claims are *evidence-backed* in the paper's own sense rather than asserted: a proof term is an
artefact, and it can be re-checked by anyone.

## What it proves, and what it deliberately does not

The file's header records its own history, because that history is part of the point:

* An earlier version asserted that "the second abstraction injective on the image of the first" is
  the condition missing from the paper's composition proposition. **That is false.** The refutations
  are themselves theorems in the file (`conditions_independent`), so the mistake is machine-checked
  rather than retracted in prose. What the paper uses instead is the weaker, correct condition
  (see `injOnImage_sufficient`, `injOnImage_not_necessary`).
* `relational_oblivious_not_equivalence` establishes the defect that does matter: for a relational
  inter-level edge the induced "identified" relation is reflexive and symmetric but not transitive,
  so the quotient is not definable.
* `monoidal_G_double`, `monoidal_G_iterate` and `threshold_not_monoidal` are the enrichment
  computation. Their mathematical content is **not** claimed as new: it is Lawvere's classification
  of the monoidal functors from the Lawvere quantale to the two-element poset. What the paper
  contributes is the application, not the mathematics.

## How to check it

Requires Lean 4 (tested with 4.34.1) and **no Mathlib** — only `Std`, which ships with the
toolchain. From this directory:

```
lean claims.lean
```

Success is a zero exit status with no output. There are no `sorry`, `axiom`, or `admit` terms in the
file; a grep for them is a valid check that nothing is assumed.

## A limitation, recorded rather than hidden

Kernel checking happens during compilation, which is what `lean claims.lean` exercises. An
independent re-check of the produced `.olean` with `leanchecker` was attempted and **not completed**
(the tool could not locate the object file in this toolchain layout). The claims below are therefore
kernel-checked but not independently replayed.
