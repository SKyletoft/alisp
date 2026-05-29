# Alisp

Alisp is a lisp dialect because I want to experiment with optimising
JIT compilers in an untyped context.

This project has no interesting type system or syntax, hence s-expressions.

Based on a mix of elisp, scheme and clojure, because I'm mostly
familiar with elisp. Following Scheme whenever I have to look up any
ambiguity or oddity. And then taking arrays, maps and reasonable
datastructures from Clojure.

We also have optional type signatures for comparison's sake for the JIT.

The actual core language is kept as minimal as possible with as much
sugar as possible being passed to macros in the prelude.

Example:

```lisp
(defun square [(x i32)] -> i32
  (* x x)))

(defun fma [x y z]
  (+ (* x y) z))

(println (square 4))
```
