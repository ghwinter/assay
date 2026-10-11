/-
  Machine-checked correction of the revision-round formal claims.

  History, recorded deliberately: the first version of this file asserted that
  "g injective on the image of f" is the condition missing from
  `prop:soundness`. THAT IS FALSE. The refutations are proved below, so the
  mistake is machine-checked rather than merely retracted in prose.

  What survives:
    * kernel monotonicity and the equality criterion (true, but elementary
      first-isomorphism material — not novel, see the literature note);
    * the correct composition condition, and that injectivity on the image is
      sufficient but NOT necessary for it;
    * the defect that actually matters for the paper: for a relational edge the
      oblivious set is not transitive, so `B / Obl` is not definable;
    * the enrichment computation (content due to Lawvere, cited — not claimed as
      new): a monoidal map out of a divisible monoid cannot separate two
      positive values, and no per-stage threshold is monoidal.

  Build:  lean claims.lean        (needs only `import Std`; no Mathlib)
-/

import Std

namespace AssayPaper

/-! ## 1. Kernel monotonicity — true, elementary, not novel -/

theorem ker_comp_subset {α β γ : Type} (f : α → β) (g : β → γ) :
    ∀ a a' : α, f a = f a' → g (f a) = g (f a') :=
  fun _ _ h => congrArg g h

/-- `g` injective on the image of `f`. True characterisation of kernel equality
— but, as proved in §2, **not** the completeness condition and **not** what
composition needs. -/
def InjOnImage {α β γ : Type} (f : α → β) (g : β → γ) : Prop :=
  ∀ b b' : β, (∃ a, f a = b) → (∃ a', f a' = b') → g b = g b' → b = b'

theorem ker_comp_eq_iff {α β γ : Type} (f : α → β) (g : β → γ) :
    (∀ a a' : α, g (f a) = g (f a') ↔ f a = f a') ↔ InjOnImage f g := by
  constructor
  · intro h b b' hb hb' hg
    obtain ⟨a, rfl⟩ := hb
    obtain ⟨a', rfl⟩ := hb'
    exact (h a a').mp hg
  · intro h a a'
    exact ⟨fun hg => h (f a) (f a') ⟨a, rfl⟩ ⟨a', rfl⟩ hg, fun hf => congrArg g hf⟩

/-! ## 2. Refutation of this file's own earlier claim

GRS completeness (Giacobazzi–Ranzato–Scozzari, MFCS'98 Def. 3.1, plain form) is
`α` being a *homomorphism for* `f`: `α`-indistinguishable inputs have
`α`-indistinguishable images. `InjOnImage f g` is instead a *separation*
condition on the image. They are independent, in both directions. -/

/-- GRS completeness, in the form used here. -/
def IsComplete {α β γ : Type} (f : α → β) (h : β → γ) (a : α → γ) : Prop :=
  ∀ x y : α, a x = a y → h (f x) = h (f y)

/-- **Refutation A.** Completeness does **not** imply injectivity on the image.
`α` groups `Fin 4` as `{0,1} {2,3}`; `f` maps each group into a group (so `α` is
a homomorphism for `f`), yet `f`'s image contains `0` and `1`, which `α`
identifies. -/
def aA : Fin 4 → Fin 2
  | 0 => 0
  | 1 => 0
  | 2 => 1
  | 3 => 1

def fA : Fin 4 → Fin 4
  | 0 => 0
  | 1 => 1
  | 2 => 0
  | 3 => 1

theorem complete_A : IsComplete fA aA aA := by
  unfold IsComplete
  decide
theorem not_injOnImage_A : ¬ InjOnImage fA aA := by
  unfold InjOnImage
  decide

/-- **Refutation B.** Injectivity on the image does **not** imply completeness
(the direction that matters for `prop:soundness`). `α` merges `{0,1} {2,3}
{4,5}` and is injective on `im f = {2,4}`, yet `0` and `1` are `α`-identical
while their images are not. -/
def aB : Fin 6 → Fin 3
  | 0 => 0
  | 1 => 0
  | 2 => 1
  | 3 => 1
  | 4 => 2
  | 5 => 2

def fB : Fin 6 → Fin 6
  | 0 => 2
  | 1 => 4
  | 2 => 2
  | 3 => 2
  | 4 => 4
  | 5 => 4

theorem injOnImage_B : InjOnImage fB aB := by
  unfold InjOnImage
  decide
theorem not_complete_B : ¬ IsComplete fB aB aB := by
  unfold IsComplete
  decide

/-- **The two conditions are independent.** The earlier claim in this file —
that injectivity on the image is the missing condition — is refuted in the
direction that matters. -/
theorem conditions_independent :
    (∃ (α β γ : Type) (f : α → β) (h : β → γ) (a : α → γ),
        IsComplete f h a ∧ ¬ InjOnImage f h) ∧
    (∃ (α β γ : Type) (f : α → β) (h : β → γ) (a : α → γ),
        InjOnImage f h ∧ ¬ IsComplete f h a) :=
  ⟨⟨Fin 4, Fin 4, Fin 2, fA, aA, aA, complete_A, not_injOnImage_A⟩,
   ⟨Fin 6, Fin 6, Fin 3, fB, aB, aB, injOnImage_B, not_complete_B⟩⟩

/-! ## 3. What composition actually needs

Not injectivity of the second stage on the image of the first, but: whatever the
two stages jointly identify must already be identified by the external
behaviour. Injectivity on the image is the special case `ext =` equality, hence
sufficient and strictly stronger. -/

def ComposeCondition {α β γ δ : Type} (f : α → β) (g : β → γ) (ext : α → δ) : Prop :=
  ∀ x y : α, g (f x) = g (f y) → ext x = ext y

/-- **Sufficient.** -/
theorem injOnImage_sufficient {α β γ δ : Type} (f : α → β) (g : β → γ) (ext : α → δ)
    (hinj : InjOnImage f g) (hext : ∀ x y : α, f x = f y → ext x = ext y) :
    ComposeCondition f g ext :=
  fun x y h => hext x y (hinj (f x) (f y) ⟨x, rfl⟩ ⟨y, rfl⟩ h)

/-- **Not necessary.** `ext` may identify more than `f` does, which injectivity
on the image forbids. -/
theorem injOnImage_not_necessary :
    ∃ (f : Fin 3 → Fin 3) (g : Fin 3 → Fin 2) (ext : Fin 3 → Fin 1),
      ComposeCondition f g ext ∧ ¬ InjOnImage f g := by
  refine ⟨id, fun _ => 0, fun _ => 0, fun _ _ _ => rfl, ?_⟩
  intro hinj
  have : (0 : Fin 3) = 1 := hinj 0 1 ⟨0, rfl⟩ ⟨1, rfl⟩ rfl
  exact absurd this (by decide)

/-! ## 4. The defect that does matter: the relational oblivious set is not an
equivalence relation, so `B / Obl` is not definable

`prop:oblivious` — "abstraction is quotienting by the kernel" — is stated for a
*function*. Where an inter-level edge is a general relation, the induced
"identified" relation is reflexive and symmetric but not transitive. -/

/-- `R ⊆ Fin 3 × Fin 2` given by `{(0,0), (1,0), (1,1), (2,1)}`. -/
def R : Fin 3 → Fin 2 → Bool
  | 0, 0 => true
  | 1, 0 => true
  | 1, 1 => true
  | 2, 1 => true
  | _, _ => false

/-- `a ~ a'` iff some `b` is `R`-related to both. -/
def tilde (a a' : Fin 3) : Bool :=
  (List.finRange 2).any (fun b => R a b && R a' b)

theorem tilde_refl : ∀ a : Fin 3, tilde a a = true := by decide
theorem tilde_symm : ∀ a a' : Fin 3, tilde a a' = tilde a' a := by decide

/-- **Not transitive.** `0 ~ 1` via `b = 0`, `1 ~ 2` via `b = 1`, but their
related sets are disjoint so `0 ≁ 2`. -/
theorem tilde_not_transitive :
    ¬ (∀ a a' a'' : Fin 3, tilde a a' = true → tilde a' a'' = true → tilde a a'' = true) := by
  intro h
  have h01 : tilde 0 1 = true := by decide
  have h12 : tilde 1 2 = true := by decide
  have h02 : tilde 0 2 = true := h 0 1 2 h01 h12
  exact absurd h02 (by decide)

/-- **The statement the paper needs and does not have.** For a relational edge
the oblivious set is reflexive and symmetric but not transitive, hence is not an
equivalence relation and cannot be quotiented by. -/
theorem relational_oblivious_not_equivalence :
    (∀ a : Fin 3, tilde a a = true) ∧
    (∀ a a' : Fin 3, tilde a a' = tilde a' a) ∧
    ¬ (∀ a a' a'' : Fin 3, tilde a a' = true → tilde a' a'' = true → tilde a a'' = true) :=
  ⟨tilde_refl, tilde_symm, tilde_not_transitive⟩

/-! ## 5. The enrichment computation

Content due to Lawvere (2002 Author Commentary on "Metric spaces, generalized
logic, and closed categories"): the monoidal functors from the Lawvere quantale
to the two-element poset are exactly three — constant `⊤`, `[ε < ∞]` (π₀, the
left adjoint), `[ε = 0]` (the right adjoint, Kelly's underlying poset). The last
two agree on all finite positive values, so no monoidal map separates two finite
positive unreliabilities. Proved here: the doubling computation, and that no
per-stage threshold is monoidal. -/

/-- **The doubling computation.** `G (e + e) = G e`. -/
theorem monoidal_G_double (G : Nat → Bool)
    (hmul : ∀ a b : Nat, G (a + b) = (G a && G b)) (e : Nat) :
    G (e + e) = G e := by
  rw [hmul, Bool.and_self]

/-- `n`-fold doubling, written as an explicit recursion to avoid depending on
`Function.iterate`. -/
def dblIter (e : Nat) : Nat → Nat
  | 0 => e
  | n + 1 => dblIter e n + dblIter e n

/-- Iterated: `G` is constant along the whole doubling orbit, so it cannot
separate `e` from `2ⁿ · e`. -/
theorem monoidal_G_iterate (G : Nat → Bool)
    (hmul : ∀ a b : Nat, G (a + b) = (G a && G b)) (e : Nat) :
    ∀ n : Nat, G (dblIter e n) = G e := by
  intro n
  induction n with
  | zero => rfl
  | succ k ih =>
    show G (dblIter e k + dblIter e k) = G e
    rw [hmul, ih, Bool.and_self]

/-- **No threshold is monoidal.** With `θ = 10`, each of two stages of
unreliability `6` passes; the composite (`12`) does not. So per-stage
reliability gates do not compose — reliability must be *budgeted*, not gated. -/
theorem threshold_not_monoidal :
    ¬ (∀ G : Nat → Bool, (∃ θ : Nat, ∀ e, G e = decide (e ≤ θ)) →
        ∀ a b, G (a + b) = (G a && G b)) := by
  intro h
  have key := h (fun e => decide (e ≤ 10)) ⟨10, fun _ => rfl⟩ 6 6
  exact absurd key (by decide)

end AssayPaper
