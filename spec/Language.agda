module Language where

open import Agda.Builtin.Bool
open import Agda.Builtin.List
open import Agda.Builtin.Maybe
open import Agda.Builtin.Nat
open import Agda.Builtin.Sigma
open import Agda.Builtin.String
open import Data.List.Base using (_++_)
open import Function using (case_of_; _$_; _∘_)
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
    ptr       : Ref n → Value n
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
    p-mac       : PartialValues n → PartialValue n
    p-pair      : PartialValue n → PartialValue n → PartialValue n
    p-quasiquot : PartialValue n → PartialValue n

  PartialValues : Nat → Set
  PartialValues n = List (PartialValue n)

  Heap : Nat → Set
  Heap n = Vec (Value n) n

  Scope : Nat → Set
  Scope n = List (String × Ref n)

  Scopes : Nat → Set
  Scopes n = NonEmptyList (Scope n)

weaken-value-suc : {n : Nat} → Value n → Value (suc n)
weaken-value-suc (atom x)               = atom x
weaken-value-suc (number x)             = number x
weaken-value-suc (pair (ref i) (ref j)) = pair (ref (weaken-fin i)) (ref (weaken-fin j))
weaken-value-suc (quot (ref i))         = quot (ref (weaken-fin i))
weaken-value-suc (quasiquot (ref i))    = quasiquot (ref (weaken-fin i))
weaken-value-suc (unquot (ref i))       = unquot (ref (weaken-fin i))
weaken-value-suc (ptr (ref i))          = ptr (ref (weaken-fin i))
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
weaken-partial-suc {n} (p-mac es) = p-mac (map-weaken es)
  where
    map-weaken : List (PartialValue n) → List (PartialValue (suc n))
    map-weaken []       = []
    map-weaken (x ∷ es) = weaken-partial-suc x ∷ map-weaken es

weaken-partial : {n m : Nat} → {p : n ≤ m} → PartialValue n → PartialValue m
weaken-partial {n} {m} {base b} v    = v
weaken-partial {n} {suc m} {ind p} v = weaken-partial-suc (weaken-partial {n} {m} {p} v)

weaken-ref : {n m : Nat} → {p : n ≤ m} → Ref n → Ref m
weaken-ref {p = p} (ref fin) = ref (weaken-fin-many {proof = p} fin)

weaken-scope : {n m : Nat} → {p : n ≤ m} → Scope n → Scope m
weaken-scope [] = []
weaken-scope {p = p} ((s , r) ∷ ss) = (s , weaken-ref {p = p} r) ∷ weaken-scope {p = p} ss

weaken-scopes : {n m : Nat} → {p : n ≤ m} → Scopes n → Scopes m
weaken-scopes {p = p} = map (weaken-scope {p = p})

mutual
  expr-to-value : {n : Nat} → State n → Expr → Σ Nat (λ m → (n ≤ m) × State m × Value m)
  expr-to-value {n} s (atom x)   = n , base n , s , atom x
  expr-to-value {n} s (number x) = n , base n , s , number x
  expr-to-value s (pair l r) =
    let _ , p , s , l' = insert-expr l s
        m , q , s , r' = insert-expr r s
        proof = trans-less p q
     in m , proof , s , pair (weaken-ref {p = q} l') r'
  expr-to-value s (quot e) =
    let m , p , s , r = insert-expr e s
     in m , p , s , quot r
  expr-to-value s (quasiquot e) =
    let m , p , s , r = insert-expr e s
     in m , p , s , quasiquot r
  expr-to-value s (unquot e) =
    let m , p , s , r = insert-expr e s
     in m , p , s , unquot r

  insert-value : {n : Nat} → State n → Value n → Σ Nat (λ m → (n ≤ m) × State m × Ref m)
  insert-value {n} (state heap scopes) v =
    let heap = weaken-value {p = indb n} v ∷ map (weaken-value {p = indb n}) heap
        scopes = weaken-scopes {p = indb n} scopes
    in suc n , indb n , state heap scopes , ref (from-nat n)

  insert-expr : {n : Nat} → Expr → State n → Σ Nat (λ m → (n ≤ m) × State m × Ref m)
  insert-expr {n} e s =
    let m , p , s , val = expr-to-value s e
        o , _ , s , r = insert-value s val
     in o , ind p , s , r

  insert-many-or-none : {n : Nat} → List Expr → State n → Σ Nat (λ m → (n ≤ m) × State m)
  insert-many-or-none [] s = _ , base _ , s
  insert-many-or-none (e ∷ es) s =
    let m , p , s' , r = insert-expr e s
     in m , p , s'

  insert-many : {n : Nat} → NonEmptyList Expr → State n → Σ Nat (λ m → (n ≤ m) × State m × Ref m)
  insert-many (e ∷ es) s =
    let m , p , s     = insert-many-or-none es s
        m , q , s , r = insert-expr {m} e s
        proof         = trans-less p q
     in m , proof , s , r

lookup : {n : Nat} (r : Ref n) → State n → Value n
lookup (ref r) (state vals _ ) = vals !!! r

value-to-expr : {n : Nat} → Nat → State n → Value n → Maybe Expr
value-to-expr zero _ _              = NONE
value-to-expr _ s (atom x)          = just (atom x)
value-to-expr _ s (number x)        = just (number x)
value-to-expr (suc f) s (pair r r₁) = do
  e  ← value-to-expr f s (lookup r s)
  e₁ ← value-to-expr f s (lookup r₁ s)
  return $ pair e e₁
value-to-expr (suc f) s (quot r)      = map quot      (value-to-expr f s (lookup r s))
value-to-expr (suc f) s (quasiquot r) = map quasiquot (value-to-expr f s (lookup r s))
value-to-expr (suc f) s (unquot r)    = map unquot    (value-to-expr f s (lookup r s))
value-to-expr (suc f) s (ptr r)       = value-to-expr f s (lookup r s)
value-to-expr _       s (lam _ _)     = just (atom "lambda")
value-to-expr _       s (mac _ _)     = just (atom "macro")
value-to-expr _       s (builtin _)   = just (atom "builtin")

ref-to-expr : {n : Nat} → State n → Ref n → Maybe Expr
ref-to-expr {n} s r = value-to-expr n s (lookup r s)

find : {n : Nat} → String → State n → Maybe (Ref n)
find {n} s (state _ (current-scope ∷ _)) = find-where (primStringEquality s) current-scope

replace : {n : Nat} → Value n → State n → Fin n → State n
replace e (state vals names) i = state (set-at e vals i) names

extract-builtin-args : {n : Nat} → Nat → State n → Ref n → Maybe (List (Ref n))
extract-builtin-args 0 _ _ = nothing
extract-builtin-args (suc n) s r with lookup r s
... | pair l r = do
  xs ← extract-builtin-args n s r
  return $ l ∷ xs
... | atom "nil" = just []
... | _ = nothing

extract-args : {n : Nat} → State n → Ref n → List String → Maybe (List (String × Ref n))
extract-args s r (id ∷ xs) with lookup r s
... | pair e e₁ = do
      rest ← extract-args s e₁ xs
      return $ (id , e) ∷ rest
... | _ = nothing
extract-args s r [] with lookup r s
... | atom "nil" = just []
... | _ = nothing

BuiltinSig : Nat → Set
BuiltinSig n = State n → List (Ref n) → Maybe $ Σ Nat (λ m → (n ≤ m) × State m × Ref m)

lambda-builtin-impl : {n : Nat} → BuiltinSig n
lambda-builtin-impl {n} s (args ∷ body) = do
  args ← (transpose ∘ map (λ {r → case lookup r s of λ
        { (atom x) → just x
        ; _ → nothing
        }
    })) =<< extract-builtin-args n s args
  let body = {!!}
  just $ insert-value s $ lam args body
lambda-builtin-impl _ _ = nothing

macro-builtin-impl : {n : Nat} → BuiltinSig n
macro-builtin-impl s e = NONE

set-builtin-impl : {n : Nat} → BuiltinSig n
set-builtin-impl s e = NONE

declare-builtin-impl : {n : Nat} → BuiltinSig n
declare-builtin-impl s e = NONE

match-builtin-impl : {n : Nat} → BuiltinSig n
match-builtin-impl s e = NONE

return-value : {n : Nat} → PartialValues n → Maybe (Ref n)
return-value (evaluated x ∷ []) = just x
return-value (evaluated _ ∷ es) = return-value es
return-value _ = nothing

mutual
  small-step : {n : Nat} → State n → PartialValue n → Maybe (Σ Nat (λ m → (n ≤ m) × State m × PartialValue m))
  small-step {n} s (evaluated x) = just (n , base n , s , evaluated x)
  small-step {n} s (unevaluated x) = small-step-expr s x
  small-step {n} s (p-fun es) = do
    m , p , heap , all-scopes , popped-scopes , es ← case small-step-many s es of λ
      { (just (m , p , s@(state heap all-scopes@(ss ∷ sss ∷ scopes)) , es)) →
        let popped-scopes = sss ∷ scopes
            -- Agda cannot figure out which _,_ without an explicit signature, _∋_ doesn't work either
            ret : Σ Nat (λ m →  n ≤ m × Heap m × Scopes m × Scopes m × List (PartialValue m))
            ret = m , p , heap , all-scopes , popped-scopes , es
         in just ret
      ; _ → nothing
      }
    case return-value es of λ
      { (just r) → just (m , p , state heap popped-scopes , evaluated r)
      ; nothing  → just (m , p , state heap all-scopes    , p-fun es)
      }
  small-step {n} s (p-mac es) = do
    m , p , heap , all-scopes , popped-scopes , es ← case small-step-many s es of λ
      { (just (m , p , s@(state heap all-scopes@(ss ∷ sss ∷ scopes)) , es)) →
        let popped-scopes = sss ∷ scopes
            -- Agda cannot figure out which _,_ without an explicit signature, _∋_ doesn't work either
            ret : Σ Nat (λ m →  n ≤ m × Heap m × Scopes m × Scopes m × List (PartialValue m))
            ret = m , p , heap , all-scopes , popped-scopes , es
         in just ret
      ; _ → nothing
      }
    case return-value es of λ
      { (just r) → do
          let s = state heap popped-scopes
          let m , _ , s , r = insert-value s (ptr r)
          just (m , ind p , s , evaluated r)
      ; nothing  → just (m , p , state heap all-scopes , p-fun es)
      }
  small-step {n} s (p-pair (evaluated r@(ref i)) tail) with lookup r s | small-step s tail
  ... | lam params body | just (o , lt-proof , s@(state heap scopes) , evaluated x) = do
          args ← extract-args s x params
          let s = state heap (args ::: scopes)
          return $ o , lt-proof , s , p-fun (map unevaluated body)
  ... | lam _ _ | just (o , lt-proof , s , tail) =
          return $ o , lt-proof , s , p-pair (weaken-partial {p = lt-proof} (evaluated r)) tail
  ... | lam _ _ | nothing = nothing
  ... | mac params body | _ = do
          m , lt-proof , s@(state heap (scope ∷ scopes)) , tail ← case tail of λ
            { (unevaluated xs) → return $ insert-expr xs s
            ; _ → nothing
            }
          args ← extract-args s tail params
          let s = state heap ((args ++ scope) ∷ scope ∷ scopes)
          return $ m , lt-proof , s , p-mac (map unevaluated body)
  ... | builtin b | _ = do
          m , lt-proof , s , tail ← case tail of λ
            { (unevaluated xs) → return $ insert-expr xs s
            ; _ → nothing
            }
          args ← extract-builtin-args n s tail
          let builtin-fn = case b of λ
                { lambda-builtin → lambda-builtin-impl
                ; macro-builtin → macro-builtin-impl
                ; set-builtin → set-builtin-impl
                ; declare-builtin → declare-builtin-impl
                ; match-builtin → match-builtin-impl
                }
          m , p , s , v ← builtin-fn s args
          return $ m , trans-less lt-proof p , s , evaluated v
  ... | _       | _   = nothing
  small-step s (p-pair l r) = do
    (n , proof , s , l') ← small-step s l
    let r' = weaken-partial {p = proof} r
    return $ n , proof , s , p-pair l' r'
  small-step {n} s (p-quasiquot e) = NONE

  small-step-many : {n : Nat} → State n → PartialValues n → Maybe (Σ Nat (λ m → (n ≤ m) × State m × PartialValues m))
  small-step-many {n} s []                 = just (n , (base n , (s , [])))
  small-step-many s (e@(evaluated _) ∷ es) = do
    m , p , s' , es' ← small-step-many s es
    return $ m , p , s' , weaken-partial {p = p} e ∷ es'
  small-step-many s (e ∷ es) = do
    m , p , s' , e' ← small-step s e
    return $ m , p , s' , e' ∷ map (weaken-partial {p = p}) es

  small-step-expr : {n : Nat} → State n → Expr → Maybe (Σ Nat (λ m → (n ≤ m) × State m × PartialValue m))
  small-step-expr {n} s (atom x) = do
    r ← find x s
    return $ n , base n , s , evaluated r
  small-step-expr {n} s (number x) =
    let m , p , s' , r = insert-expr (number x) s
     in return $ m , p , s' , evaluated r
  small-step-expr {n} s (pair x x₁) = just (n , base n , s , p-pair (unevaluated x) (unevaluated x₁))
  small-step-expr {n} s (quot x)    =
    let m , p , s' , r = insert-expr x s
     in return $ m , p , s' , evaluated r
  small-step-expr {n} s (quasiquot x) = just (n , base n , s , p-quasiquot (unevaluated x))
  small-step-expr {n} s (unquot x)    = nothing

new-heap : Heap 5
new-heap =
  (builtin lambda-builtin)
  ∷ (builtin macro-builtin)
  ∷ (builtin set-builtin)
  ∷ (builtin declare-builtin)
  ∷ (builtin match-builtin)
  ∷ []

new-scope : Scope 5
new-scope =
  ( ("lambda"  , ref zero)
   ∷ ("macro"   , ref (suc zero))
   ∷ ("set"     , ref (suc (suc zero)))
   ∷ ("declare" , ref (suc (suc (suc zero))))
   ∷ ("match"   , ref (suc (suc (suc (suc zero)))))
   ∷ []
  )

new-state : State 5
new-state = state new-heap (new-scope ∷ [])

eval : Nat → Expr → Maybe Expr
eval fuel expr = do
  n , _ , s , pv ← small-step-expr new-state expr
  _ , _ , s , e  ← do-steps {n} fuel s pv
  r              ← is-evaluated e
  value-to-expr fuel s (lookup r s)
  where
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
