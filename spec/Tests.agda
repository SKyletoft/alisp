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

infixr 5 _:h:_
_:h:_ : {n : Nat} → Value (suc n) → State n → State (suc n)
_:h:_ {n} v (state heap scopes) =
  let f = map λ { (str , (ref fin)) → str , ref (weaken-fin fin) }
  in state (v ∷ map (weaken-value {p = indb n}) heap)
           (map f scopes)

quot-any-state : {n : Nat} (s : State n) →
  small-step s (unevaluated (quot x))
    ≡ just ( suc n , indb n , atom "x" :h: s , evaluated (ref (from-nat n)))
  × ref-to-expr (atom "x" :h: s) (ref (from-nat n))
    ≡ just x
quot-any-state {n} s =
  refl ,
  trans
    (cong (value-to-expr (suc n) (atom "x" :h: s))
          (!!!-head (atom "x") (map (weaken-value {p = indb n}) (State.heap s))))
    refl
