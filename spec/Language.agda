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
  state : Vec (Value n) n →
          NonEmptyList (List (String × Fin n)) → State n

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

weaken-ref : {n m : Nat} → {p : n ≤ m} → Ref n → Ref m
weaken-ref {p = p} (ref fin) = ref (weakenFinMany {proof = p} fin)

mutual
  expr-to-value : {n : Nat} → State n → Expr → Σ Nat (λ m → (n ≤ m) × State m × Value m)
  expr-to-value {n} s (atom x)   = n , base n , s , atom x
  expr-to-value {n} s (number x) = n , base n , s , number x
  expr-to-value {n} s (pair e e₁) with insert e s
  ... | n , p , s , r with insert e₁ s
  ... | m , p' , s , r₁ = m , trans-less p p' , s , pair (weaken-ref {p = p'} r) r₁
  expr-to-value {n} s (quot e) with insert e s
  ... | m , p , s' , r =  m , p , s' , quot r
  expr-to-value {n} s (quasiquot e) with insert e s
  ... | m , p , s' , r =  m , p , s' , quasiquot r
  expr-to-value {n} s (unquot e) with insert e s
  ... | m , p , s' , r =  m , p , s' , unquot r
  expr-to-value {n} s (lam x x₁) = n , base n , s , lam x x₁
  expr-to-value {n} s (mac x x₁) = n , base n , s , mac x x₁

  insert : {n : Nat} → Expr → State n → Σ Nat (λ m → (n ≤ m) × State m × Ref m)
  insert {n} e s with expr-to-value s e
  ... | m , p , s'@(state vals names) , val =
    let f : List (String × Fin m) → List (String × Fin (suc m))
        f = map (λ where (str , fin) → str , weakenFin fin)
        vals' : Vec (Value (suc m)) (suc m)
        vals' = weaken-value {p = indb m} val
              ∷ v-map (weaken-value {p = indb m}) vals
        names' : NonEmptyList (List (String × Fin (suc m)))
        names' = ne-map f names
    in (suc m) , ind p , state vals' names' , ref (fromNat m)

lookup : {n : Nat} (r : Ref n) → State n → Value n
lookup (ref r) (state vals _ ) = vals !! r

find : {n : Nat} → String → State n → Maybe (Ref n)
find {n} s (state _ (names ∷ _)) with find-where (primStringEquality s) names
... | just i = just (ref i)
... | nothing = nothing

replace : {n : Nat} → Value n → State n → Fin n → State n
replace e (state vals names) i = state (setAt e vals i) names

extract-args : {n : Nat} → State n → Ref n → List String → Maybe (List (Ref n))
extract-args s r (_ ∷ xs) with lookup r s
... | pair e e₁ =
  let rest = extract-args s e₁ xs
    in (λ es → (e ∷ es)) <$> rest
... | _ = nothing
extract-args s r [] with lookup r s
... | pair _ _ = nothing
... | _ = just []

mutual
  small-step : {n : Nat} → State n → PartialValue n → Maybe (Σ Nat (λ m → (n ≤ m) × State m × PartialValue m))
  small-step {n} s (evaluated x) = just (n , base n , s , evaluated x)
  small-step {n} s (unevaluated (atom x)) with find x s
  ... | just r = just (n , base n , s , evaluated r)
  ... | nothing = nothing
  small-step {n} s (unevaluated (number x)) with insert (number x) s
  ... | m , p , s' , r = just (m , p , s' , evaluated r)
  small-step {n} s (unevaluated (pair x x₁)) = just (n , base n , s , p-pair (unevaluated x) (unevaluated x₁))
  small-step {n} s (unevaluated (quot x)) with insert x s
  ... | m , p , s' , r = just (m , p , s' , evaluated r)
  small-step {n} s (unevaluated (quasiquot x)) = just (n , base n , s , p-quasiquot (unevaluated x))
  small-step {n} s (unevaluated (unquot x)) = nothing
  small-step {n} s (unevaluated x@(lam _ _)) with insert x s
  ... | m , p , s' , r = just (m , p , s' , evaluated r)
  small-step {n} s (unevaluated x@(mac _ _)) with insert x s
  ... | m , p , s' , r = just (m , p , s' , evaluated r)
  small-step s (p-pair (evaluated r@(ref i)) e₁) with lookup r s | small-step s e₁
  -- ... | lam _ _ | just (m , (lt-proof , (s' , e2))) =
          -- let r' = evaluated (ref (weakenFinMany {proof = lt-proof} i))
          -- in just (m , (lt-proof , (s' , p-pair r' e₂)))
  ... | lam params body | just (n , lt-proof , s , evaluated x) =
      -- Do function call
        let args = extract-args s x params
        in {!!}
  ... | lam _ _ | just (m , lt-proof , s' , unevaluated x) = {!!}
  ... | lam _ _ | just (m , lt-proof , s' , p-pair e2 e3) = {!!}
      -- The final element should be evaluated, but is unused
  ... | lam _ _ | just (m , lt-proof , s' , p-quasiquot e2) = {!!}
  ... | lam _ _ | nothing = nothing
  ... | mac _ _ | _ = {!!}
  ... | _ | _ = nothing
  small-step s (p-pair e e₁) with small-step s e
  ... | just (n , proof , s , e') =
          let e₁' = (weaken-partial {p = proof} e₁)
          in just (n , proof , s , p-pair e' e₁')
  ... | nothing = nothing
  small-step {n} s (p-quasiquot e) = {!!}

  small-step-many : {n : Nat} → State n → List (PartialValue n) → Maybe (Σ Nat (λ m → (n ≤ m) × State m × List (PartialValue m)))
  small-step-many {n} s [] = just (n , (base n , (s , [])))
  small-step-many s (e@(evaluated _) ∷ es) with small-step-many s es
  ... | just (m , p , s' , es') = just (m , p , s' , weaken-partial {p = p} e ∷ es')
  ... | nothing = nothing
  small-step-many s (e ∷ es) with small-step s e
  ... | just (m , p , s' , e') = just (m , p , s' , e' ∷ map (weaken-partial {p = p}) es)
  ... | nothing = nothing
