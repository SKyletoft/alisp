module Language where

open import Agda.Builtin.Bool
open import Agda.Builtin.List
open import Agda.Builtin.Maybe
open import Agda.Builtin.Nat
open import Agda.Builtin.Sigma
open import Agda.Builtin.String
open import Function using (case_of_; _$_)
open import Helpers

open Monad {{...}}

-- Greppable replacement for nothing for early proofs
NONE : {a : Set} → Maybe a
NONE = nothing

mutual
  record State (n : Nat) : Set where
    constructor state
    field
      heap   : Heap n
      scopes : Scopes n

  record Ref (n : Nat) : Set where
    constructor ref
    field
      ref : Fin n

  data Expr : Set where
    atom      : String → Expr
    number    : Nat → Expr
    pair      : Expr → Expr → Expr
    quot      : Expr → Expr
    quasiquot : Expr → Expr
    unquot    : Expr → Expr

  data Value (n : Nat) : Set where
    atom      : String → Value n
    number    : Nat → Value n
    pair      : Ref n → Ref n → Value n
    quot      : Ref n → Value n
    quasiquot : Ref n → Value n
    unquot    : Ref n → Value n
    lam       : List String → List Expr → Value n
    mac       : List String → List Expr → Value n
    builtin   : Builtin → Value n

  data Builtin : Set where
    lambda-builtin  : Builtin
    macro-builtin   : Builtin
    set-builtin     : Builtin
    declare-builtin : Builtin
    match-builtin   : Builtin

  data PartialValue (n : Nat) : Set where
    evaluated   : Ref n → PartialValue n
    unevaluated : Expr → PartialValue n
    p-fun       : PartialValues n → PartialValue n
    p-mac       : List String → PartialValues n → PartialValue n
    p-pair      : PartialValue n → PartialValue n → PartialValue n
    p-quasiquot : PartialValue n → PartialValue n

  PartialValues : Nat → Set
  PartialValues n = List (PartialValue n)

  Heap : Nat → Set
  Heap n = Vec (Value n) n

  Scopes : Nat → Set
  Scopes n = NonEmptyList (List (String × Ref n))

weaken-value-suc : {n : Nat} → Value n → Value (suc n)
weaken-value-suc (atom x)               = atom x
weaken-value-suc (number x)             = number x
weaken-value-suc (pair (ref i) (ref j)) = pair (ref (weaken-fin i)) (ref (weaken-fin j))
weaken-value-suc (quot (ref i))         = quot (ref (weaken-fin i))
weaken-value-suc (quasiquot (ref i))    = quasiquot (ref (weaken-fin i))
weaken-value-suc (unquot (ref i))       = unquot (ref (weaken-fin i))
weaken-value-suc (lam x x₁)             = lam x x₁
weaken-value-suc (mac x x₁)             = mac x x₁
weaken-value-suc (builtin x)            = builtin x

weaken-value : {n m : Nat} → {p : n ≤ m} → Value n → Value m
weaken-value {n} {m} {base b} v    = v
weaken-value {n} {suc m} {ind p} v = weaken-value-suc (weaken-value {n} {m} {p} v)

weaken-partial-suc : {n : Nat} → PartialValue n → PartialValue (suc n)
weaken-partial-suc (evaluated (ref i)) = evaluated (ref (weaken-fin i))
weaken-partial-suc (unevaluated e)     = unevaluated e
weaken-partial-suc (p-pair v v₁)       = p-pair (weaken-partial-suc v) (weaken-partial-suc v₁)
weaken-partial-suc (p-quasiquot v)     = p-quasiquot (weaken-partial-suc v)
weaken-partial-suc {n} (p-fun es)      = p-fun (map-weaken es)
  where
    map-weaken : List (PartialValue n) → List (PartialValue (suc n))
    map-weaken []       = []
    map-weaken (x ∷ es) = weaken-partial-suc x ∷ map-weaken es
weaken-partial-suc {n} (p-mac vs es) = p-mac vs (map-weaken es)
  where
    map-weaken : List (PartialValue n) → List (PartialValue (suc n))
    map-weaken []       = []
    map-weaken (x ∷ es) = weaken-partial-suc x ∷ map-weaken es

weaken-partial : {n m : Nat} → {p : n ≤ m} → PartialValue n → PartialValue m
weaken-partial {n} {m} {base b} v    = v
weaken-partial {n} {suc m} {ind p} v = weaken-partial-suc (weaken-partial {n} {m} {p} v)

weaken-ref : {n m : Nat} → {p : n ≤ m} → Ref n → Ref m
weaken-ref {p = p} (ref fin) = ref (weaken-fin-many {proof = p} fin)

mutual
  expr-to-value : {n : Nat} → State n → Expr → Σ Nat (λ m → (n ≤ m) × State m × Value m)
  expr-to-value {n} s (atom x)   = n , base n , s , atom x
  expr-to-value {n} s (number x) = n , base n , s , number x
  expr-to-value s (pair l r) =
    let _ , p , s , l' = insert l s
        m , q , s , r' = insert r s
        proof = trans-less p q
     in m , proof , s , pair (weaken-ref {p = q} l') r'
  expr-to-value s (quot e) =
    let m , p , s , r = insert e s
     in m , p , s , quot r
  expr-to-value s (quasiquot e) =
    let m , p , s , r = insert e s
     in m , p , s , quasiquot r
  expr-to-value s (unquot e) =
    let m , p , s , r = insert e s
     in m , p , s , unquot r

  insert : {n : Nat} → Expr → State n → Σ Nat (λ m → (n ≤ m) × State m × Ref m)
  insert {n} e s with expr-to-value s e
  ... | m , p , state vals names , val =
    let f = map λ { (str , (ref fin)) → str , ref (weaken-fin fin) }
        vals = weaken-value {p = indb m} val
              ∷ map (weaken-value {p = indb m}) vals
        names = map f names
     in suc m , ind p , state vals names , ref (from-nat m)

  insert-many-or-none : {n : Nat} → List Expr → State n → Σ Nat (λ m → (n ≤ m) × State m)
  insert-many-or-none [] s = _ , base _ , s
  insert-many-or-none (e ∷ es) s =
    let m , p , s' , r = insert e s
     in m , p , s'

  insert-many : {n : Nat} → NonEmptyList Expr → State n → Σ Nat (λ m → (n ≤ m) × State m × Ref m)
  insert-many (e ∷ es) s =
    let m , p , s     = insert-many-or-none es s
        m , q , s , r = insert {m} e s
        proof         = trans-less p q
     in m , proof , s , r

lookup : {n : Nat} (r : Ref n) → State n → Value n
lookup (ref r) (state vals _ ) = vals !! r

find : {n : Nat} → String → State n → Maybe (Ref n)
find {n} s (state _ (current-scope ∷ _)) = find-where (primStringEquality s) current-scope

replace : {n : Nat} → Value n → State n → Fin n → State n
replace e (state vals names) i = state (set-at e vals i) names

extract-args : {n : Nat} → State n → Ref n → List String → Maybe (List (String × Ref n))
extract-args s r (id ∷ xs) with lookup r s
... | pair e e₁ = do
      rest ← extract-args s e₁ xs
      just $ (id , e) ∷ rest
... | _ = nothing
extract-args s r [] with lookup r s
... | pair _ _ = nothing
... | _ = just []

mutual
  small-step : {n : Nat} → State n → PartialValue n → Maybe (Σ Nat (λ m → (n ≤ m) × State m × PartialValue m))
  small-step {n} s (evaluated x) = just (n , base n , s , evaluated x)
  small-step {n} s (unevaluated x) = small-step-expr s x
  small-step {n} s (p-fun es) = do
    m , p , heap , all-scopes , popped-scopes , es ← case small-step-many s es of λ
      { (just (m , p , s@(state heap all-scopes@(ss ∷ sss ∷ scopes)) , es)) →
        -- Agda cannot figure out which _,_ without an explicit signature, _∋_ doesn't work either
        let popped-scopes = sss ∷ scopes
            ret : Σ Nat (λ m →  n ≤ m × Heap m × Scopes m × Scopes m × List (PartialValue m))
            ret = m , p , heap , all-scopes , popped-scopes , es
         in just ret
      ; _ → nothing
      }
    case return-value es of λ
      { (just r) → just (m , p , state heap popped-scopes , evaluated r)
      ; nothing  → just (m , p , state heap all-scopes    , p-fun es)
      }
    where
      return-value : {n : Nat} → PartialValues n → Maybe (Ref n)
      return-value (evaluated x ∷ []) = just x
      return-value (evaluated _ ∷ es) = return-value es
      return-value _ = nothing
  small-step s (p-pair (evaluated r@(ref i)) e₁) with lookup r s | small-step s e₁
  ... | lam params body | just (o , lt-proof , s@(state heap scopes) , evaluated x) = do
          args ← extract-args s x params
          let s = state heap (args ::: scopes)
          just $ o , lt-proof , s , p-fun (map unevaluated body)
  ... | lam _ _ | ret = ret
  ... | mac _ _ | _   = NONE
  ... | _       | _   = nothing
  small-step s (p-pair l r) = do
    (n , proof , s , l') ← small-step s l
    let r' = weaken-partial {p = proof} r
    just $ n , proof , s , p-pair l' r'
  small-step s (p-mac vs es) =
    -- Continue macro call
    NONE
  small-step {n} s (p-quasiquot e) = NONE

  small-step-many : {n : Nat} → State n → PartialValues n → Maybe (Σ Nat (λ m → (n ≤ m) × State m × PartialValues m))
  small-step-many {n} s []                 = just (n , (base n , (s , [])))
  small-step-many s (e@(evaluated _) ∷ es) = do
    m , p , s' , es' ← small-step-many s es
    just $ m , p , s' , weaken-partial {p = p} e ∷ es'
  small-step-many s (e ∷ es) = do
    m , p , s' , e' ← small-step s e
    just $ m , p , s' , e' ∷ map (weaken-partial {p = p}) es

  small-step-expr : {n : Nat} → State n → Expr → Maybe (Σ Nat (λ m → (n ≤ m) × State m × PartialValue m))
  small-step-expr {n} s (atom x) = do
    r ← find x s
    just $ n , base n , s , evaluated r
  small-step-expr {n} s (number x) =
    let m , p , s' , r = insert (number x) s
    in just $ m , p , s' , evaluated r
  small-step-expr {n} s (pair x x₁) = just (n , base n , s , p-pair (unevaluated x) (unevaluated x₁))
  small-step-expr {n} s (quot x)    =
    let m , p , s' , r = insert x s
    in just $ m , p , s' , evaluated r
  small-step-expr {n} s (quasiquot x) = just (n , base n , s , p-quasiquot (unevaluated x))
  small-step-expr {n} s (unquot x)    = nothing

new-state : State 5
new-state = state
  ( (builtin lambda-builtin)
  ∷ (builtin macro-builtin)
  ∷ (builtin set-builtin)
  ∷ (builtin declare-builtin)
  ∷ (builtin match-builtin)
  ∷ []
  )
  (( ("lambda"  , ref zero)
   ∷ ("macro"   , ref (suc zero))
   ∷ ("set"     , ref (suc (suc zero)))
   ∷ ("declare" , ref (suc (suc (suc zero))))
   ∷ ("match"   , ref (suc (suc (suc (suc zero)))))
   ∷ []
  ) ∷ [])

eval : Nat → Expr → Maybe Expr
eval fuel expr = do
  n , _ , s , pv ← small-step-expr new-state expr
  _ , _ , s , e  ← do-steps {n} fuel s pv
  r              ← is-evaluated e
  value-to-expr fuel s (lookup r s)
  where
    value-to-expr : {n : Nat} → Nat → State n → Value n → Maybe Expr
    value-to-expr zero _ _              = NONE
    value-to-expr _ s (atom x)          = just (atom x)
    value-to-expr _ s (number x)        = just (number x)
    value-to-expr (suc f) s (pair r r₁) = do
      e  ← value-to-expr f s (lookup r s)
      e₁ ← value-to-expr f s (lookup r₁ s)
      just $ pair e e₁
    value-to-expr (suc f) s (quot r)      = map quot      (value-to-expr f s (lookup r s))
    value-to-expr (suc f) s (quasiquot r) = map quasiquot (value-to-expr f s (lookup r s))
    value-to-expr (suc f) s (unquot r)    = map unquot    (value-to-expr f s (lookup r s))
    value-to-expr _       s (lam _ _)     = just (atom "lambda")
    value-to-expr _       s (mac _ _)     = just (atom "macro")
    value-to-expr _       s (builtin _)   = just (atom "builtin")

    do-steps : {n : Nat} → Nat → State n → PartialValue n → Maybe (Σ Nat (λ m → (n ≤ m) × State m × PartialValue m))
    do-steps {n} _ s (evaluated x) = just (n , base n , s , evaluated x)
    do-steps zero _ _ = nothing
    do-steps {n} (suc fuel) s values = do
      n , p , s , values ← small-step s values
      n , q , s , values ← do-steps fuel s values
      just (n , trans-less p q , s , values)

    is-evaluated : {n : Nat} → PartialValue n → Maybe (Ref n)
    is-evaluated (evaluated x) = just x
    is-evaluated _ = nothing


BuiltinSig : Nat → Set
BuiltinSig n = State n → List Expr → Maybe $ Σ Nat (λ m → (n ≤ m) × State m × Value m)

lambda-builtin-impl : {n : Nat} → BuiltinSig n
lambda-builtin-impl s e = NONE

macro-builtin-impl : {n : Nat} → BuiltinSig n
macro-builtin-impl s e = NONE

set-builtin-impl : {n : Nat} → BuiltinSig n
set-builtin-impl s e = NONE

declare-builtin-impl : {n : Nat} → BuiltinSig n
declare-builtin-impl s e = NONE

match-builtin-impl : {n : Nat} → BuiltinSig n
match-builtin-impl s e = NONE
