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
  let f = map λ { (str , (ref fin)) → str , ref (weaken-fin fin) }
  in state (atom "x" ∷ map (weaken-value {p = indb n}) heap)
           (map f scopes)

quot-any-state : {n : Nat} (s : State n) →
  small-step s (unevaluated (quot x))
    ≡ just ( suc n , indb n , insert-value s (atom "x") , evaluated (ref (from-nat n)))
  × ref-to-expr (insert-value s (atom "x")) (ref (from-nat n))
    ≡ just x
quot-any-state {n} s =
  refl ,
  trans
    (cong (value-to-expr (suc n) (insert-value s (atom "x")))
          (!!!-head (atom "x") (map (weaken-value {p = indb n}) (State.heap s))))
    refl
