module Language where

open import Agda.Builtin.Bool
open import Agda.Builtin.Nat
open import Agda.Builtin.String
open import Data.String.Properties using (_≟_)
open import Agda.Builtin.List
open import Agda.Builtin.Maybe
open import Relation.Binary.PropositionalEquality using (_≡_; refl; cong; subst)
open import Relation.Nullary using (yes; no)
open import Helpers

data Expr : Set where
  atom      : String → Expr
  number    : Nat → Expr
  pair      : Expr → Expr → Expr
  quot      : Expr → Expr
  quasiquot : Expr → Expr
  unquot    : Expr → Expr
  lam       : List String → List Expr → Expr
  mac       : List String → List Expr → Expr

data Ref (n : Nat) : Set where
  ref : Fin n → Ref n

data State (n : Nat) : Set where
  state : {f : Fin n} →
          Vec Expr n →
          Vec String (toNat f) → State n

lookup : {n : Nat} (r : Ref n) → State n → Expr
lookup (ref r) (state vals _ ) = vals !! r

find : {n : Nat} → String → State n → Maybe (Ref n)
find {n} s (state {f} _ names) with indexOf _≟_ names s
... | just i = just (ref (weaken-coerce f i))
... | nothing = nothing

insert : {n : Nat} → Expr → State n → State (suc n) × Ref (suc n)
insert {n} e (state {f} vals names) =
  state {f = weakenFin f}
        (e ∷ vals)
        (subst (λ n → Vec String n) (toNat-weaken f) names)
  , ref (fromNat n)

replace : {n : Nat} → Expr → State n → Fin n → State n
replace e (state vals names) i = state (setAt e vals i) names

small-step : {n : Nat} → (State n) × Expr → Maybe (State (suc n) × Ref (suc n))
small-step (s@(state vals names) , atom x) with find x s
... | nothing = nothing
... | just r@(ref i) with insert (lookup r s) s
...   | s' , _ = just (s' , ref (weakenFin i))
small-step (s , pair e e₁)   = {!!}
small-step (s , quasiquot e) = {!!}
small-step (s , unquot e)    = nothing
small-step (s , quot e)      = just (insert e s)
small-step (s , number x)    = just (insert (number x) s)
small-step (s , lam x x₁)    = just (insert (lam x x₁) s)
small-step (s , mac x x₁)    = just (insert (mac x x₁) s)
