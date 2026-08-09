module Helpers where

open import Agda.Builtin.Bool
open import Agda.Builtin.List
open import Agda.Builtin.Maybe
open import Agda.Builtin.Nat
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

len : {a : Set} → List a → Nat
len [] = zero
len (_ ∷ xs) = suc (len xs)

to-vec : {a : Set} → (l : List a) → Vec a (len l)
to-vec [] = []
to-vec (x ∷ xs) = x ∷ to-vec xs

map : {a b : Set} → (a → b) → List a → List b
map f [] = []
map f (x ∷ xs) = f x ∷ map f xs

v-map : {n : Nat} → {a b : Set} → (a → b) → Vec a n → Vec b n
v-map f [] = []
v-map f (x ∷ xs) = f x ∷ v-map f xs

ne-map : {a b : Set} → (a → b) → NonEmptyList a → NonEmptyList b
ne-map f (x ∷ xs) = f x ∷ map f xs

fromNat : (n : Nat) → Fin (suc n)
fromNat zero = zero
fromNat (suc e) = suc (fromNat e)

toNat : {n : Nat} → (Fin n) → Nat
toNat zero = zero
toNat (suc x) = suc (toNat x)

weakenFin : {n : Nat} → Fin n → Fin (suc n)
weakenFin zero = zero
weakenFin (suc i) = suc (weakenFin i)

weakenFinMany : {n m : Nat} → {proof : n ≤ m} → Fin n → Fin m
weakenFinMany {n} {m} {base b} f = f
weakenFinMany {n} {suc m} {ind p} f = weakenFin (weakenFinMany {n} {m} {p} f)

toNat-weaken : {n : Nat} (f : Fin n) →
               toNat f ≡ toNat (weakenFin f)
toNat-weaken zero = refl
toNat-weaken (suc f) = cong suc (toNat-weaken f)

weaken-coerce : {n : Nat} (f : Fin n) → Fin (toNat f) → Fin n
weaken-coerce zero ()
weaken-coerce (suc f) zero = zero
weaken-coerce (suc f) (suc i) = weakenFin (weaken-coerce f i)

trans-less : {m n o : Nat} → m ≤ n → n ≤ o → m ≤ o
trans-less (base b) q = q
trans-less (ind p) (base b) = ind p
trans-less (ind p) (ind q) = ind (trans-less (ind p) q)

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
        → (a → Bool)
        → Vec a n
        → Maybe (Fin n)
indexOf eq [] = nothing
indexOf eq (y ∷ ys) with eq y
... | true = just zero
... | false with indexOf eq ys
...   | nothing = nothing
...   | just i = just (suc i)

_>>=_ : {a b : Set} → Maybe a → (a → Maybe b) → Maybe b
(just x) >>= f = f x
nothing >>= _ = nothing

_=<<_ : {a b : Set} → (a → Maybe b) → Maybe a → Maybe b
f =<< m = m >>= f

_<$>_ : {a b : Set} → (a → b) → Maybe a → Maybe b
f <$> (just x) = just (f x)
_ <$> nothing = nothing

find-where : {a b : Set} → (a → Bool) → List (a × b) → Maybe b
find-where f [] = nothing
find-where f ((x , y) ∷ xs) with f x
... | true = just y
... | false = find-where f xs
