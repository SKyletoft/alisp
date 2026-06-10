use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use smallvec::smallvec;

use crate::{
	eval::{self, RuntimeError},
	lisp_object::{
		Env, LispParseTree, ObjectReference,
		parse_tree::{atom, int, list, quasiquote, quote, unquote},
	},
	parse,
};

fn eval(code: &str) -> Result<LispParseTree, RuntimeError> {
	let mut env = Env::wait_for_new();
	let parsed = parse::parse_many(code).unwrap();
	let res = parsed.into_iter().try_fold((&mut env).nil(), |_, node| {
		let obj = ObjectReference::from_parse_object(node, &mut env);
		eval::eval_top(obj, &mut env)
	})?;
	let printable = crate::lisp_object::lisp_object_to_parse_tree(env.get(res), &env);
	Ok(printable)
}

#[quickcheck]
fn add(a: i16, b: i16) {
	let code = format!("(+ {a} {b})");
	let result = eval(&code);
	let expected = int((a as i32) + (b as i32));
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn sub(a: i16, b: i16) {
	let code = format!("(- {a} {b})");
	let result = eval(&code);
	let expected = int((a as i32) - (b as i32));
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn mul(a: i16, b: i16) {
	let code = format!("(* {a} {b})");
	let result = eval(&code);
	let expected = int((a as i32) * (b as i32));
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn div(a: i16, b: i16) -> TestResult {
	if b == 0 {
		return TestResult::discard();
	}
	let code = format!("(/ {a} {b})");
	let result = eval(&code);
	let expected = int((a as i32) / (b as i32));
	assert_eq!(result, Ok(expected));
	TestResult::passed()
}

#[quickcheck]
fn rem(a: i16, b: i16) -> TestResult {
	if b == 0 {
		return TestResult::discard();
	}
	let code = format!("(% {a} {b})");
	let result = eval(&code);
	let expected = int((a as i32) % (b as i32));
	assert_eq!(result, Ok(expected));
	TestResult::passed()
}

#[quickcheck]
fn eq(a: i16, b: i16) {
	let code = format!("(= {a} {b})");
	let result = eval(&code);
	let expected = ((a as i32) == (b as i32)).into();
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn lt(a: i16, b: i16) {
	let code = format!("(< {a} {b})");
	let result = eval(&code);
	let expected = ((a as i32) < (b as i32)).into();
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn gt(a: i16, b: i16) {
	let code = format!("(> {a} {b})");
	let result = eval(&code);
	let expected = ((a as i32) > (b as i32)).into();
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn lte(a: i16, b: i16) {
	let code = format!("(<= {a} {b})");
	let result = eval(&code);
	let expected = ((a as i32) <= (b as i32)).into();
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn gte(a: i16, b: i16) {
	let code = format!("(>= {a} {b})");
	let result = eval(&code);
	let expected = ((a as i32) >= (b as i32)).into();
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn add_mul(a: i16, b: i16, c: i16) {
	let code = format!("(* (+ {a} {b}) {c})");
	let result = eval(&code);
	let expected = int(((a as i32) + (b as i32)) * (c as i32));
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn add_sub_combined(a: i16, b: i16, c: i16) {
	let code = format!("(+ (- {a} {b}) {c})");
	let result = eval(&code);
	let expected = int(((a as i32) - (b as i32)) + (c as i32));
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn mul_sub_combined(a: i16, b: i16, c: i16) {
	let code = format!("(- (* {a} {b}) {c})");
	let result = eval(&code);
	let expected = int(((a as i32) * (b as i32)) - (c as i32));
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn div_mul_combined(a: i16, b: i16, c: i16, d: i16) -> TestResult {
	if c == d {
		return TestResult::discard();
	}
	let code = format!("(/ (* {a} {b}) (- {c} {d}))");
	let result = eval(&code);
	let expected = int(((a as i32) * (b as i32)) / ((c as i32) - (d as i32)));
	assert_eq!(result, Ok(expected));
	TestResult::passed()
}

#[quickcheck]
fn eq_combined(a: i16, b: i16, c: i16, d: i16) {
	let code = format!("(= (+ {a} {b}) (- {c} {d}))");
	let result = eval(&code);
	let expected = (((a as i32) + (b as i32)) == ((c as i32) - (d as i32))).into();
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn lt_combined(a: i16, b: i16, c: i16, d: i16) {
	let code = format!("(< (+ {a} {b}) (* {c} {d}))");
	let result = eval(&code);
	let expected = (((a as i32) + (b as i32)) < ((c as i32) * (d as i32))).into();
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn modulo_combined(a: i16, b: i16, c: i16, d: i16) -> TestResult {
	if c == d {
		return TestResult::discard();
	}
	let code = format!("(% (+ {a} {b}) (- {c} {d}))");
	let result = eval(&code);
	let expected = int(((a as i32) + (b as i32)) % ((c as i32) - (d as i32)));
	assert_eq!(result, Ok(expected));
	TestResult::passed()
}

#[quickcheck]
fn all_arithmetic(a: i16, b: i16, c: i16, d: i16, e: i16) -> TestResult {
	if d == 0 {
		return TestResult::discard();
	}
	let code = format!("(+ (- (* {a} {b}) (/ {c} {d})) {e})");
	let result = eval(&code);
	let expected = int((((a as i32) * (b as i32)) - ((c as i32) / (d as i32))) + (e as i32));
	assert_eq!(result, Ok(expected));
	TestResult::passed()
}

#[test]
fn print_returns_nil() {
	let code = "(print 42)";
	let expected = "nil".into();
	let result = eval(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn println_returns_nil() {
	let code = "(println 42)";
	let expected = "nil".into();
	let result = eval(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn print_combined() {
	let code = "(print (+ 1 2))";
	let expected = "nil".into();
	let result = eval(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn immediately_invoked_lambda() {
	let code = "((lambda [x] (* x x)) 2)";
	let expected = int(4);
	let result = eval(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn defun_expand() {
	// let code = "(defun square [x] (* x x))";
	// let expected = "(set 'square (lambda [x] (* x x)))";
	// let result = crate::eval::expand(code);
	// assert_eq!(result, Ok(expected));
}

#[test]
fn defun_eval() {
	let code = "(defun square [x] (* x x)) (square 2)";
	let expected = int(4);
	let result = eval(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn fib() {
	fn rust_fib(i: i32) -> i32 {
		match i {
			..=1 => 1,
			i => rust_fib(i - 1) + rust_fib(i - 2),
		}
	}
	let code = r#"
(defun fib [(i i32)]
       (cond ((<= i 1) 1)
	    (t (+ (fib (- i 1))
		  (fib (- i 2))))))
(fib 3)
"#;
	let expected = int(rust_fib(3));
	let result = eval(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn empty_vec_to_nil() {
	let list: LispParseTree = Vec::<LispParseTree>::new().into();
	assert!(matches!(list, LispParseTree::Atom(s) if s == "nil"));
}

#[test]
fn empty_vec_deque_to_nil() {
	use std::collections::VecDeque;
	let list: LispParseTree = VecDeque::<LispParseTree>::new().into();
	assert!(matches!(list, LispParseTree::Atom(s) if s == "nil"));
}

#[test]
fn display_atom() {
	assert_eq!(format!("{}", LispParseTree::Atom("hello".into())), "hello");
}

#[test]
fn display_integer() {
	assert_eq!(format!("{}", int(42)), "42");
}

#[test]
fn display_float() {
	assert_eq!(format!("{}", LispParseTree::Float(3.14)), "3.14");
}

#[test]
fn display_proper_list() {
	let list: LispParseTree = vec![int(1), int(2), int(3)].into();
	assert_eq!(format!("{list}"), "(1 2 3)");
}

#[test]
fn display_improper_list() {
	use crate::lisp_object::LispParseTree::*;
	let list = Pair(Box::new(Integer(1)), Box::new(Integer(2)));
	assert_eq!(format!("{list}"), "(1 . 2)");
}

#[test]
fn display_nested_list() {
	let list: LispParseTree = vec![LispParseTree::from(vec![int(1), int(2)]), int(3)].into();
	assert_eq!(format!("{list}"), "((1 2) 3)");
}

#[test]
fn display_lambda() {
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None)],
		ret_ty: None,
		body: vec![LispParseTree::Atom("body".into())],
	};
	assert_eq!(format!("{lambda}"), "(λ [x] body)");
}

#[test]
fn display_lambda_multi_arg() {
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None), ("y".into(), None)],
		ret_ty: None,
		body: vec![LispParseTree::Atom("body".into())],
	};
	assert_eq!(format!("{lambda}"), "(λ [x y] body)");
}

#[test]
fn display_partially_typed_lambda_1() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), Some(LispType::Integer))],
		ret_ty: None,
		body: vec![LispParseTree::Atom("body".into())],
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32)] body)");
}

#[test]
fn display_partially_typed_lambda_2() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None)],
		ret_ty: Some(LispType::Integer),
		body: vec![LispParseTree::Atom("body".into())],
	};
	assert_eq!(format!("{lambda}"), "(λ [x] -> i32 body)");
}

#[test]
fn display_partially_typed_lambda_3() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None), ("y".into(), Some(LispType::Integer)),],
		ret_ty: Some(LispType::Integer),
		body: vec![LispParseTree::Atom("body".into())],
	};
	assert_eq!(format!("{lambda}"), "(λ [x (y i32)] -> i32 body)");
}

#[test]
fn display_partially_typed_lambda_4() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None), ("y".into(), Some(LispType::Integer)),],
		ret_ty: None,
		body: vec![LispParseTree::Atom("body".into())],
	};
	assert_eq!(format!("{lambda}"), "(λ [x (y i32)] body)");
}

#[test]
fn display_typed_lambda() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), Some(LispType::Integer))],
		ret_ty: Some(LispType::Integer),
		body: vec![LispParseTree::Atom("body".into())],
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32)] -> i32 body)");
}

#[test]
fn display_partial_typed_lambda() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), Some(LispType::Integer)), ("y".into(), None),],
		ret_ty: Some(LispType::Named("bool".into())),
		body: vec![LispParseTree::Atom("body".into())],
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32) y] -> bool body)");
}

#[test]
fn set_and_get() {
	let code = "(set 'x 5) x";
	let expected = int(5);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn set_quote() {
	let code = "(set 'id 'x) (set id 5) x";
	let expected = int(5);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn inline_macro() {
	let code = "((macro [x] ,x) 1)";
	let expected = int(1);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_from_var() {
	let code = "(set 'm (macro [x] ,x)) (m 1)";
	let expected = int(1);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_plus_one() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) (m 1)";
	let expected = int(2);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_expands_in_quasiquote() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) `(m 1)";
	let expected = parse::parse("(+ 1 1)").unwrap();
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_expands_in_quasiquoted_list() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) `(1 (m 1) 3)";
	let expected = parse::parse("(1 (+ 1 1) 3)").unwrap();
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_does_not_expand_in_quoted_list() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) '(1 (m 1) 3)";
	let expected = parse::parse("'(1 (m 1) 3)").unwrap();
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_does_not_expand_in_quote() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) '(m 1)";
	let expected = parse::parse("'(m 1)").unwrap();
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn quote_outside_macro_keeps_nested_quasiquote_literal() {
	let code = "'(a `(b ,c) d)";
	let expected = list([
		atom("a"),
		quasiquote(list([atom("b"), unquote(atom("c"))])),
		atom("d"),
	]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn quote_inside_macro_keeps_nested_quasiquote_literal() {
	let code = "(set 'm (macro [x] '(a `(b ,x) d))) (m 7)";
	let expected = list([
		atom("a"),
		quasiquote(list([atom("b"), unquote(atom("x"))])),
		atom("d"),
	]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn quote_inside_macro_keeps_plain_data_literal() {
	let code = "(set 'm (macro [x] '(a b c))) (m 7)";
	let expected = list([atom("a"), atom("b"), atom("c")]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn quasiquote_outside_macro_unquotes_expression() {
	let code = "`(a ,(+ 1 2) d)";
	let expected = list([atom("a"), int(3), atom("d")]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn quasiquote_outside_macro_unquotes_head_and_tail() {
	let code = "`(,(+ 1 2) b ,(+ 3 4))";
	let expected = list([int(3), atom("b"), int(7)]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn nested_quasiquote_preserves_inner_unquote() {
	let code = "``(a ,x)";
	let expected = quasiquote(list([atom("a"), unquote(atom("x"))]));
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn nested_quasiquote_preserves_inner_unquote_in_macro() {
	let code = "(set 'm (macro [x] ``(a ,x))) (m 7)";
	let expected = quasiquote(list([atom("a"), unquote(atom("x"))]));
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_call_inside_quote_stays_literal() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) '(m 1)";
	let expected = list([atom("m"), int(1)]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_call_inside_quasiquote_expands() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) `(m 1)";
	let expected = list([atom("+"), int(1), int(1)]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_call_inside_quasiquoted_list_expands() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) `(1 (m 1) 3)";
	let expected = list([int(1), list([atom("+"), int(1), int(1)]), int(3)]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_argument_quoted_list_stays_quoted() {
	let code = "(set 'm (macro [x] x)) (m '(1 2 3))";
	let expected = quote(list([int(1), int(2), int(3)]));
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_argument_quasiquoted_list_splices() {
	let code = "(set 'm (macro [x] x)) (m `(1 ,(+ 1 2) 3))";
	let expected = list([int(1), int(3), int(3)]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_returns_quoted_code_without_expanding_it() {
	let code = "(set 'm (macro [x] '(m ,x))) (m 9)";
	let expected = list([atom("m"), unquote(atom("x"))]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macro_returns_quasiquoted_code_with_nested_quote() {
	let code = r#"
		(set 'm (macro [x] `(m '(inner ,x))))
		(m 9)
	"#;
	let expected = list([atom("m"), quote(list([atom("inner"), unquote(atom("x"))]))]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[test]
fn var_goes_out_of_scope() {
	let code = r#"
		(set 'f (lambda [x] (set 'y 1) x))
		x
	"#;
	let expected = Err(RuntimeError::UndefinedVariable("x".into()));
	let res = eval(code);
	assert_eq!(res, expected);
}

#[test]
fn shadowing_args_dont_alias_in_call() {
	let code = r#"
		(set 'x 5)
		(set 'f (lambda [x y] (+ x y)))
		(f 6 x)
	"#;
	let expected = int(11);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}
