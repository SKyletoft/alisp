open import Data.Nat
open import Data.List
open import Data.String

data Id : Set where
  ident : String → Id

mutual
  data Stmt : Set where
    expr : Expr → Stmt
    set  : Id → Expr → Stmt
    decl : Id → Stmt

  -- Any valid expression in the language
  data Expr : Set where
    -- int actually represents both integers (u64) and floats (f64)
    int   : ℕ → Expr
    lam   : List Id → List Stmt → Expr → Expr
    mac   : List Id → List Stmt → Expr → Expr
    id    : Id → Expr
    qot   : Expr → Expr
    unqot : Expr → Expr
    qqot  : Expr → Expr
    pair  : Expr → Expr → Expr

-- Actually evaluated values, the subset of expressions that doesn't contain unevaluated middle steps.
-- Used to make sure order of operations is respected
data Value : Expr → Set where
  valᵢ : ∀ n → Value (int n)
  valₗ : ∀ ids → ∀ ss → ∀ e → Value (lam ids ss e)
  valₘ : ∀ ids → ∀ ss → ∀ e → Value (mac ids ss e)

-- The relation of going from one step to another
data _↦_ : Expr → Expr → Set where
  -- add₁ :
  --   ∀ {e₁ e₁' e₂} →
  --   e₁ ↦ e₁' →
  --   add e₁ e₂ ↦ add e₁' e₂
