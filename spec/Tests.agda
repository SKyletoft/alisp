module Tests where

open import Agda.Builtin.Maybe
open import Agda.Builtin.List
open import Agda.Builtin.Nat
open import Agda.Builtin.Sigma
open import Function using (case_of_; _$_)
open import Language
open import Helpers
import Relation.Binary.PropositionalEquality as Eq
open Eq
open Eq.≡-Reasoning

x : Expr
x = atom "x"

nil : Expr
nil = atom "nil"

insert-value : {n : Nat} → State n → Value (suc n) → State (suc n)
insert-value {n} (state heap scopes) v =
  let p = indb n
  in state (atom "x" ∷ map (weaken-value {p = p}) heap)
           (weaken-scopes {p = p} scopes)

quot-inner :
 let final-state = insert-value new-state (atom "x")
     final-expr = ref (from-nat 5)
 in (small-step new-state (unevaluated (quot x)) ≡ just ( 6 , indb 5 , final-state , evaluated final-expr))
    × ref-to-expr final-state final-expr ≡ just x
quot-inner = refl , refl
