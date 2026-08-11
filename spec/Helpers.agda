module Helpers where

open import Agda.Builtin.Bool
open import Agda.Builtin.List
open import Agda.Builtin.Maybe
open import Agda.Builtin.Nat
open import Data.List.Base using (_++_)
open import Relation.Binary.PropositionalEquality using (_≡_; refl; cong; subst)

data _≤_ : Nat → Nat → Set where
  base : (b : Nat) → b ≤ b
  ind  : {a b : Nat} → a ≤ b → a ≤ (suc b)

indb : (n : Nat) → n ≤ (suc n)
indb n = ind (base n)

data _×_ (a b : Set) : Set where
  _,_ : a → b → a × b

infixr 2 _×_
infixr 4 _,_

data Vec (a : Set) : Nat → Set where
  []  : Vec a zero
  _∷_ : {n : Nat} (x : a) → Vec a n → Vec a (suc n)
{-# COMPILE GHC Vec = data [] (:) #-}

data NonEmptyList (a : Set) : Set where
  _∷_ : a → List a → NonEmptyList a
{-# COMPILE GHC NonEmptyList = data (:) #-}

data Fin : Nat → Set where
  zero : {n : Nat} → Fin (suc n)
  suc  : {n : Nat} → Fin n → Fin (suc n)

_:::_ : {a : Set} → a → NonEmptyList a → NonEmptyList a
x ::: (y ∷ ys) = x ∷ y ∷ ys

len : {a : Set} → List a → Nat
len [] = zero
len (_ ∷ xs) = suc (len xs)

to-vec : {a : Set} → (l : List a) → Vec a (len l)
to-vec [] = []
to-vec (x ∷ xs) = x ∷ to-vec xs

v-map : {n : Nat} → {a b : Set} → (a → b) → Vec a n → Vec b n
v-map f [] = []
v-map f (x ∷ xs) = f x ∷ v-map f xs

record Monad (F : Set → Set) : Set₁ where
  field
    return : {a : Set} → a → F a
    _>>=_  : {a b : Set} → F a → (a → F b) → F b
    _=<<_  : {a b : Set} → (a → F b) → F a → F b
    _<$>_  : {a b : Set} → (a → b) → F a → F b

open Monad {{...}}

map : {F : Set → Set} {{ r : Monad F }} {a b : Set} → (a → b) → F a → F b
map = _<$>_

data Identity (a : Set) : Set where
  identity : a → Identity a
{-# COMPILE GHC Identity = data Identity (Identity) #-}

replicate : {n : Nat} {a : Set} → a → Vec a n
replicate {zero} _ = []
replicate {suc n} x = x ∷ replicate {n} x

v-head : {n : Nat} {a : Set} → Vec a (suc n) → a
v-head (x ∷ _) = x

v-tail : {n : Nat} {a : Set} → Vec a (suc n) → Vec a n
v-tail (_ ∷ xs) = xs

bind-vec : {n : Nat} {a b : Set} → Vec a n → (a → Vec b n) → Vec b n
bind-vec [] f = []
bind-vec (x ∷ xs) f = v-head (f x) ∷ bind-vec xs (λ y → v-tail (f y))

instance
  Maybe-Monad : Monad Maybe
  Monad.return Maybe-Monad = just
  Monad._>>=_ Maybe-Monad = λ where
    (just x) f → f x
    nothing _ → nothing
  Monad._=<<_ Maybe-Monad = λ f m → Monad._>>=_ Maybe-Monad m f
  Monad._<$>_ Maybe-Monad = λ where
    f (just x) → just (f x)
    _ nothing → nothing

  List-Monad : Monad (λ a → List a)
  Monad.return List-Monad = λ x → x ∷ []
  Monad._>>=_ List-Monad = λ where
    [] f → []
    (x ∷ xs) f → f x ++ Monad._>>=_ List-Monad xs f
  Monad._=<<_ List-Monad = λ f m → Monad._>>=_ List-Monad m f
  Monad._<$>_ List-Monad = lmap
    where
      lmap : {a b : Set} → (a → b) → List a → List b
      lmap f [] = []
      lmap f (x ∷ xs) = f x ∷ lmap f xs


  NonEmptyList-Monad : Monad (λ a → NonEmptyList a)
  Monad.return NonEmptyList-Monad = λ x → x ∷ []
  Monad._>>=_ NonEmptyList-Monad = bind
    where
    ne-tail : {b : Set} → NonEmptyList b → List b
    ne-tail (g ∷ gs) = g ∷ gs

    bind : {a b : Set} → NonEmptyList a → (a → NonEmptyList b) → NonEmptyList b
    bind (x ∷ xs) f with f x
    ... | h ∷ t = h ∷ t ++ Monad._>>=_ List-Monad xs (λ y → ne-tail (f y))
  Monad._=<<_ NonEmptyList-Monad = λ f m → Monad._>>=_ NonEmptyList-Monad m f
  Monad._<$>_ NonEmptyList-Monad = ne-map
    where
      ne-map : {a b : Set} → (a → b) → NonEmptyList a → NonEmptyList b
      ne-map f (x ∷ xs) = f x ∷ f <$> xs

  Vec-Monad : {n : Nat} → Monad (λ a → Vec a n)
  Monad.return (Vec-Monad {n}) = replicate {n}
  Monad._>>=_ (Vec-Monad {n}) = bind-vec {n}
  Monad._=<<_ (Vec-Monad {n}) = λ f m → bind-vec {n} m f
  Monad._<$>_ (Vec-Monad {n}) = v-map

  Identity-Monad : Monad Identity
  Monad.return Identity-Monad = identity
  Monad._>>=_ Identity-Monad = λ where
    (identity x) f → f x
  Monad._=<<_ Identity-Monad = λ f m → Monad._>>=_ Identity-Monad m f
  Monad._<$>_ Identity-Monad = λ where
    f (identity x) → identity (f x)

from-nat : (n : Nat) → Fin (suc n)
from-nat zero = zero
from-nat (suc e) = suc (from-nat e)

to-nat : {n : Nat} → (Fin n) → Nat
to-nat zero = zero
to-nat (suc x) = suc (to-nat x)

weaken-fin : {n : Nat} → Fin n → Fin (suc n)
weaken-fin zero = zero
weaken-fin (suc i) = suc (weaken-fin i)

weaken-fin-many : {n m : Nat} → {proof : n ≤ m} → Fin n → Fin m
weaken-fin-many {n} {m} {base b} f = f
weaken-fin-many {n} {suc m} {ind p} f = weaken-fin (weaken-fin-many {n} {m} {p} f)

to-nat-weaken : {n : Nat} (f : Fin n) →
               to-nat f ≡ to-nat (weaken-fin f)
to-nat-weaken zero = refl
to-nat-weaken (suc f) = cong suc (to-nat-weaken f)

weaken-coerce : {n : Nat} (f : Fin n) → Fin (to-nat f) → Fin n
weaken-coerce zero ()
weaken-coerce (suc f) zero = zero
weaken-coerce (suc f) (suc i) = weaken-fin (weaken-coerce f i)

trans-less : {m n o : Nat} → m ≤ n → n ≤ o → m ≤ o
trans-less (base b) q = q
trans-less (ind p) (base b) = ind p
trans-less (ind p) (ind q) = ind (trans-less (ind p) q)

_!!_ : {A : Set} {n : Nat} → Vec A n → Fin n → A
(x ∷ xs) !! zero = x
(x ∷ xs) !! suc i = xs !! i

set-at : {a : Set} {n : Nat} → a → Vec a n → Fin n → Vec a n
set-at x (y ∷ ys) zero = x ∷ ys
set-at x (y ∷ ys) (suc i) = y ∷ set-at x ys i

if_then_else_ : {a : Set} → Bool → a → a → a
if true then x else _ = x
if false then _ else x = x

index-of : {n : Nat} {a : Set}
        → (a → Bool)
        → Vec a n
        → Maybe (Fin n)
index-of eq [] = nothing
index-of eq (y ∷ ys) with eq y
... | true = return zero
... | false = suc <$> index-of eq ys

find-where : {a b : Set} → (a → Bool) → List (a × b) → Maybe b
find-where f [] = nothing
find-where f ((x , y) ∷ xs) with f x
... | true = just y
... | false = find-where f xs

unwrap-or : {a : Set} → Maybe a → a → a
unwrap-or (just x) _ = x
unwrap-or nothing x = x
