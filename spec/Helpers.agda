module Helpers where

open import Agda.Builtin.Bool
open import Agda.Builtin.Maybe
open import Agda.Builtin.Nat
open import Relation.Binary.PropositionalEquality using (_≡_; refl; cong; subst)
open import Relation.Binary using (DecidableEquality)
open import Relation.Nullary using (yes; no)

data _×_ (a b : Set) : Set where
  _,_ : a → b → a × b

data Vec (a : Set) : Nat → Set where
  []  : Vec a zero
  _∷_ : {n : Nat} (x : a) → Vec a n → Vec a (suc n)

data Fin : Nat → Set where
  zero : {n : Nat} → Fin (suc n)
  suc  : {n : Nat} → Fin n → Fin (suc n)

fromNat : (n : Nat) → Fin (suc n)
fromNat zero = zero
fromNat (suc e) = suc (fromNat e)

toNat : {n : Nat} → (Fin n) → Nat
toNat zero = zero
toNat (suc x) = suc (toNat x)

weakenFin : {n : Nat} → Fin n → Fin (suc n)
weakenFin zero = zero
weakenFin (suc i) = suc (weakenFin i)

toNat-weaken : {n : Nat} (f : Fin n) →
               toNat f ≡ toNat (weakenFin f)
toNat-weaken zero = refl
toNat-weaken (suc f) = cong suc (toNat-weaken f)

weaken-coerce : {n : Nat} (f : Fin n) → Fin (toNat f) → Fin n
weaken-coerce zero ()
weaken-coerce (suc f) zero = zero
weaken-coerce (suc f) (suc i) = weakenFin (weaken-coerce f i)

_!!_ : {A : Set} {n : Nat} → Vec A n → Fin n → A
(x ∷ xs) !! zero = x
(x ∷ xs) !! suc i = xs !! i

setAt : {a : Set} {n : Nat} → a → Vec a n → Fin n → Vec a n
setAt x (y ∷ ys) zero = x ∷ ys
setAt x (y ∷ ys) (suc i) = y ∷ setAt x ys i

if_then_else_ : {a : Set} → Bool → a → a → a
if true then x else _ = x
if false then _ else x = x

indexOf : {n : Nat} {a : Set}
        → DecidableEquality a
        → Vec a n
        → a
        → Maybe (Fin n)
indexOf eq [] x = nothing
indexOf eq (y ∷ ys) x with eq x y
... | yes _ = just zero
... | no _ with indexOf eq ys x
...   | nothing = nothing
...   | just i = just (suc i)
