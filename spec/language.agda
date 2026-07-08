open import Data.Nat using (ℕ ; zero ; suc)
open import Data.List using (List ; [] ; _∷_)
open import Data.String using (String)
open import Data.Product using (_×_ ; _,_)
open import Data.Maybe using (Maybe ; just ; nothing)
open import Relation.Binary.PropositionalEquality using (_≡_)

data Id : Set where
  ident : String → Id

data Ref : Set where
  ref : ℕ → Ref

data BuiltinMacro : Set where
  λ-builtin : BuiltinMacro
  μ-builtin : BuiltinMacro
  set-builtin : BuiltinMacro
  declare-builtin : BuiltinMacro

data Obj : Set where
  atom : Id → Obj
  int : ℕ → Obj
  nil : Obj
  pair : Ref → Ref → Obj
  quote-obj : Ref → Obj
  quasiquote-obj : Ref → Obj
  unquote-obj : Ref → Obj
  lambda-obj : List Id → List Ref → Ref → Obj
  macro-obj : List Id → List Ref → Ref → Obj
  builtin-macro : BuiltinMacro → Obj

Heap = List (Ref × Obj)
Scope = List (Id × Ref)

data RuntimeError : Set where
  undefined-id : Id → RuntimeError
  malformed-call : RuntimeError
  arity-mismatch : RuntimeError
  non-callable : RuntimeError
  invalid-unquote : RuntimeError
  invalid-set-target : RuntimeError
  invalid-declare-target : RuntimeError

data Kont : Set where
  halt : Kont
  call-head : List Ref → Kont → Kont
  call-arg : Ref → List Ref → List Ref → Kont → Kont
  qq : ℕ → Kont → Kont

data Ctrl : Set where
  eval : Ref → Ctrl
  value : Ref → Ctrl
  error : RuntimeError → Ctrl

record State : Set where
  constructor ⟨_,_,_,_⟩
  field
    heap : Heap
    scope : Scope
    ctrl : Ctrl
    kont : Kont

open State

postulate
  lookup-heap : Heap → Ref → Maybe Obj
  lookup-scope : Scope → Id → Maybe Ref

  proper-args : Heap → Ref → Maybe (List Ref)
  reverse : ∀ {A : Set} → List A → List A

  bind-params : Scope → List Id → List Ref → Maybe Scope
  bind-raw-params : Scope → List Id → List Ref → Maybe Scope
  remove-ids : Scope → List Id → Scope

  run-block : Heap → Scope → List Ref → Ref → Maybe (Heap × Ref)
  eval-to-value : Heap → Scope → Ref → Maybe (Heap × Ref)

  build-lambda-object : Heap → Scope → List Ref → Maybe (Heap × Ref)
  build-macro-object : Heap → Scope → List Ref → Maybe (Heap × Ref)
  run-set : Heap → Scope → List Ref → Maybe (Heap × Scope × Ref)
  run-declare : Heap → Scope → List Ref → Maybe (Heap × Scope × Ref)

  not-callable : Obj → Set

data _↦_ : State → State → Set where
  eval-int :
    ∀ {H S r n k} →
    lookup-heap H r ≡ just (int n) →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , value r , k ⟩

  eval-nil :
    ∀ {H S r k} →
    lookup-heap H r ≡ just nil →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , value r , k ⟩

  eval-lambda-value :
    ∀ {H S r ps ss ret k} →
    lookup-heap H r ≡ just (lambda-obj ps ss ret) →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , value r , k ⟩

  eval-macro-value :
    ∀ {H S r ps ss ret k} →
    lookup-heap H r ≡ just (macro-obj ps ss ret) →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , value r , k ⟩

  eval-id :
    ∀ {H S r x v k} →
    lookup-heap H r ≡ just (atom x) →
    lookup-scope S x ≡ just v →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , value v , k ⟩

  eval-id-miss :
    ∀ {H S r x k} →
    lookup-heap H r ≡ just (atom x) →
    lookup-scope S x ≡ nothing →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , error (undefined-id x) , k ⟩

  eval-quote :
    ∀ {H S r x k} →
    lookup-heap H r ≡ just (quote-obj x) →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , value x , k ⟩

  eval-quasiquote :
    ∀ {H S r x k} →
    lookup-heap H r ≡ just (quasiquote-obj x) →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , eval x , qq 0 k ⟩

  eval-unquote-outside-qq :
    ∀ {H S r x k} →
    lookup-heap H r ≡ just (unquote-obj x) →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , error invalid-unquote , k ⟩

  qq-finish :
    ∀ {H S r k} →
    ⟨ H , S , value r , qq 0 k ⟩ ↦ ⟨ H , S , value r , k ⟩

  qq-descend :
    ∀ {H S r x d k} →
    lookup-heap H r ≡ just (quasiquote-obj x) →
    ⟨ H , S , eval r , qq d k ⟩ ↦ ⟨ H , S , eval x , qq (suc d) k ⟩

  qq-unquote :
    ∀ {H S r x d k} →
    lookup-heap H r ≡ just (unquote-obj x) →
    ⟨ H , S , eval r , qq (suc d) k ⟩ ↦ ⟨ H , S , eval x , qq d k ⟩

  call-start :
    ∀ {H S r f xs args k} →
    lookup-heap H r ≡ just (pair f xs) →
    proper-args H xs ≡ just args →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , eval f , call-head args k ⟩

  call-bad-list :
    ∀ {H S r f xs k} →
    lookup-heap H r ≡ just (pair f xs) →
    proper-args H xs ≡ nothing →
    ⟨ H , S , eval r , k ⟩ ↦ ⟨ H , S , error malformed-call , k ⟩

  call-dispatch-lambda :
    ∀ {H S fv args ps ss ret k a as} →
    lookup-heap H fv ≡ just (lambda-obj ps ss ret) →
    args ≡ a ∷ as →
    ⟨ H , S , value fv , call-head args k ⟩ ↦
    ⟨ H , S , eval a , call-arg fv as [] k ⟩

  call-dispatch-lambda-no-args :
    ∀ {H S fv ps ss ret k S′ H′ v} →
    lookup-heap H fv ≡ just (lambda-obj ps ss ret) →
    bind-params [] ps [] ≡ just S′ →
    run-block H S′ ss ret ≡ just (H′ , v) →
    ⟨ H , S , value fv , call-head [] k ⟩ ↦ ⟨ H′ , S , value v , k ⟩

  call-arg-next :
    ∀ {H S fv a as done k v} →
    ⟨ H , S , value v , call-arg fv (a ∷ as) done k ⟩ ↦
    ⟨ H , S , eval a , call-arg fv as (v ∷ done) k ⟩

  call-lambda-enter :
    ∀ {H S fv done ps ss ret k vals S′ H′ v} →
    lookup-heap H fv ≡ just (lambda-obj ps ss ret) →
    vals ≡ reverse done →
    bind-params [] ps vals ≡ just S′ →
    run-block H S′ ss ret ≡ just (H′ , v) →
    ⟨ H , S , value v , call-arg fv [] done k ⟩ ↦ ⟨ H′ , S , value v , k ⟩

  call-dispatch-macro :
    ∀ {H S fv args ps ss ret k S′ H₁ code H₂ code′ S″} →
    lookup-heap H fv ≡ just (macro-obj ps ss ret) →
    bind-raw-params S ps args ≡ just S′ →
    run-block H S′ ss ret ≡ just (H₁ , code) →
    eval-to-value H₁ S′ code ≡ just (H₂ , code′) →
    S″ ≡ remove-ids S′ ps →
    ⟨ H , S , value fv , call-head args k ⟩ ↦ ⟨ H₂ , S″ , eval code′ , k ⟩

  call-dispatch-builtin-lambda :
    ∀ {H S fv args k H′ r} →
    lookup-heap H fv ≡ just (builtin-macro λ-builtin) →
    build-lambda-object H S args ≡ just (H′ , r) →
    ⟨ H , S , value fv , call-head args k ⟩ ↦ ⟨ H′ , S , value r , k ⟩

  call-dispatch-builtin-macro :
    ∀ {H S fv args k H′ r} →
    lookup-heap H fv ≡ just (builtin-macro μ-builtin) →
    build-macro-object H S args ≡ just (H′ , r) →
    ⟨ H , S , value fv , call-head args k ⟩ ↦ ⟨ H′ , S , value r , k ⟩

  call-dispatch-builtin-set :
    ∀ {H S fv args k H′ S′ r} →
    lookup-heap H fv ≡ just (builtin-macro set-builtin) →
    run-set H S args ≡ just (H′ , S′ , r) →
    ⟨ H , S , value fv , call-head args k ⟩ ↦ ⟨ H′ , S′ , value r , k ⟩

  call-dispatch-builtin-declare :
    ∀ {H S fv args k H′ S′ r} →
    lookup-heap H fv ≡ just (builtin-macro declare-builtin) →
    run-declare H S args ≡ just (H′ , S′ , r) →
    ⟨ H , S , value fv , call-head args k ⟩ ↦ ⟨ H′ , S′ , value r , k ⟩

  call-dispatch-non-callable :
    ∀ {H S fv args k o} →
    lookup-heap H fv ≡ just o →
    not-callable o →
    ⟨ H , S , value fv , call-head args k ⟩ ↦ ⟨ H , S , error non-callable , k ⟩
