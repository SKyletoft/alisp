module Language where

open import Agda.Builtin.Bool
open import Agda.Builtin.List
open import Agda.Builtin.Maybe
open import Agda.Builtin.Nat
open import Agda.Builtin.Sigma
open import Agda.Builtin.String
open import Helpers
open import Relation.Binary.PropositionalEquality using (subst)

data Ref (n : Nat) : Set where
  ref : Fin n → Ref n

data Expr : Set where
  atom      : String → Expr
  number    : Nat → Expr
  pair      : Expr → Expr → Expr
  quot      : Expr → Expr
  quasiquot : Expr → Expr
  unquot    : Expr → Expr
  lam       : List String → List Expr → Expr
  mac       : List String → List Expr → Expr

data Value (n : Nat) : Set where
  atom      : String → Value n
  number    : Nat → Value n
  pair      : Ref n → Ref n → Value n
  quot      : Ref n → Value n
  quasiquot : Ref n → Value n
  unquot    : Ref n → Value n
  lam       : List String → List Expr → Value n
  mac       : List String → List Expr → Value n

data PartialValue (n : Nat) : Set where
  evaluated   : Ref n → PartialValue n
  unevaluated : Expr → PartialValue n
  p-pair      : PartialValue n → PartialValue n → PartialValue n
  p-quasiquot : PartialValue n → PartialValue n

data State (n : Nat) : Set where
  state : {f : Fin n} →
          Vec Expr n →
          Vec String (toNat f) → State n

lookup : {n : Nat} (r : Ref n) → State n → Expr
lookup (ref r) (state vals _ ) = vals !! r

find : {n : Nat} → String → State n → Maybe (Ref n)
find {n} s (state {f} _ names) with indexOf primStringEquality names s
... | just i = just (ref (weaken-coerce f i))
... | nothing = nothing

insert : {n : Nat} → Expr → State n → State (suc n) × Ref (suc n)
insert {n} e (state {f} vals names) =
  state {suc n}
        (e ∷ vals)
        (subst (λ n → Vec String n) (toNat-weaken f) names)
  , ref (fromNat n)

replace : {n : Nat} → Expr → State n → Fin n → State n
replace e (state vals names) i = state (setAt e vals i) names

weaken-value-suc : {n : Nat} → Value n → Value (suc n)
weaken-value-suc (atom x)               = atom x
weaken-value-suc (number x)             = number x
weaken-value-suc (pair (ref i) (ref j)) = pair (ref (weakenFin i)) (ref (weakenFin j))
weaken-value-suc (quot (ref i))         = quot (ref (weakenFin i))
weaken-value-suc (quasiquot (ref i))    = quasiquot (ref (weakenFin i))
weaken-value-suc (unquot (ref i))       = unquot (ref (weakenFin i))
weaken-value-suc (lam x x₁)             = lam x x₁
weaken-value-suc (mac x x₁)             = mac x x₁

weaken-value : {n m : Nat} → {p : n ≤ m} → Value n → Value m
weaken-value {n} {m} {base b} v    = v
weaken-value {n} {suc m} {ind p} v = weaken-value-suc (weaken-value {n} {m} {p} v)

weaken-partial-suc : {n : Nat} → PartialValue n → PartialValue (suc n)
weaken-partial-suc (evaluated (ref i)) = evaluated (ref (weakenFin i))
weaken-partial-suc (unevaluated e)     = unevaluated e
weaken-partial-suc (p-pair v v₁)       = p-pair (weaken-partial-suc v) (weaken-partial-suc v₁)
weaken-partial-suc (p-quasiquot v)     = p-quasiquot (weaken-partial-suc v)

weaken-partial : {n m : Nat} → {p : n ≤ m} → PartialValue n → PartialValue m
weaken-partial {n} {m} {base b} v    = v
weaken-partial {n} {suc m} {ind p} v = weaken-partial-suc (weaken-partial {n} {m} {p} v)

small-step : {n : Nat} → (State n) → PartialValue n → Maybe (Σ Nat (λ m → (n ≤ m) × (State m × PartialValue m)))
small-step {n} s (evaluated x) = just (n , (base n , (s , evaluated x)))
small-step {n} s (unevaluated (atom x)) with find x s
... | just r = just (n , (base n , (s , evaluated r)))
... | nothing = nothing
small-step {n} s (unevaluated (number x)) with insert (number x) s
... | s' , r = just (suc n , (indb n , (s' , evaluated r)))
small-step {n} s (unevaluated (pair x x₁)) = just (n , (base n , (s , p-pair (unevaluated x) (unevaluated x₁))))
small-step {n} s (unevaluated (quot x)) with insert x s
... | s' , r = just (suc n , (indb n , (s' , evaluated r)))
small-step {n} s (unevaluated (quasiquot x)) = just (n , (base n , (s , p-quasiquot (unevaluated x))))
small-step {n} s (unevaluated (unquot x)) = nothing
small-step {n} s (unevaluated x@(lam _ _)) with insert x s
... | s' , r = just (suc n , (indb n , (s' , evaluated r)))
small-step {n} s (unevaluated x@(mac _ _)) with insert x s
... | s' , r = just (suc n , (indb n , (s' , evaluated r)))
small-step s (p-pair (evaluated r@(ref i)) e₁) with lookup r s | small-step s e₁
-- ... | lam _ _ | just (m , (lt-proof , (s' , e2))) =
        -- let r' = evaluated (ref (weakenFinMany {proof = lt-proof} i))
        -- in just (m , (lt-proof , (s' , p-pair r' e₂)))
... | lam _ _ | just (m , (lt-proof , (s' , evaluated x))) = {!!}
... | lam _ _ | just (m , (lt-proof , (s' , unevaluated x))) = {!!}
... | lam _ _ | just (m , (lt-proof , (s' , p-pair e2 e3))) = {!!}
... | lam _ _ | just (m , (lt-proof , (s' , p-quasiquot e2))) = {!!}
... | lam _ _ | nothing = nothing
... | mac _ _ | _ = {!!}
... | _ | _ = nothing
small-step s (p-pair e e₁) with small-step s e
... | just (n , proof , (s , e')) =
        let e₁' = (weaken-partial {p = proof} e₁)
         in just (n , (proof , (s , p-pair e' e₁')))
... | nothing = nothing
small-step {n} s (p-quasiquot e) = {!!}
