use smallvec::smallvec;

use crate::{
	eval,
    lisp_object::{Env, LispParseTree, ObjectReference},
	parse,
};

pub fn eval_in_env<'a>(code: &str, env: &mut Env<'a>) -> ObjectReference<'a> {
	let parsed = parse::parse_many(code).unwrap();
	parsed.into_iter().fold(env.nil(), |_, node| {
		let obj = ObjectReference::from_parse_object(node, env);
		eval::eval(obj, env).unwrap()
	})
}

pub fn eval(code: &str) -> LispParseTree {
	let mut env = Env::wait_for_new();
	let res = eval_in_env(code, &mut env);
	crate::lisp_object::lisp_object_to_parse_tree(env.get(res), &env)
}

#[quickcheck]
fn add(a: i16, b: i16) -> TestResult {
	if a < 0 || b < 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(+ {a} {b})")) == (a + b).into())
}

#[quickcheck]
fn sub(a: i16, b: i16) -> TestResult {
	if a < 0 || b < 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(- {a} {b})")) == (a - b).into())
}

#[quickcheck]
fn mul(a: i16, b: i16) -> TestResult {
	if a < 0 || b < 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(* {a} {b})")) == (a * b).into())
}

#[quickcheck]
fn div(a: i16, b: i16) -> TestResult {
	if a < 0 || b <= 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(/ {a} {b})")) == (a / b).into())
}

#[quickcheck]
fn rem(a: i16, b: i16) -> TestResult {
	if a < 0 || b <= 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(% {a} {b})")) == (a % b).into())
}

#[quickcheck]
fn eq(a: i16, b: i16) -> TestResult {
	if a < 0 || b < 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(= {a} {b})")) == (a == b).into())
}

#[quickcheck]
fn lt(a: i16, b: i16) -> TestResult {
	if a < 0 || b < 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(< {a} {b})")) == (a < b).into())
}

#[quickcheck]
fn gt(a: i16, b: i16) -> TestResult {
	if a < 0 || b < 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(> {a} {b})")) == (a > b).into())
}

#[quickcheck]
fn lte(a: i16, b: i16) -> TestResult {
	if a < 0 || b < 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(<= {a} {b})")) == (a <= b).into())
}

#[quickcheck]
fn gte(a: i16, b: i16) -> TestResult {
	if a < 0 || b < 0 { return TestResult::discard(); }
	let (a, b) = (a as i32, b as i32);
	TestResult::from_bool(eval(&format!("(>= {a} {b})")) == (a >= b).into())
}

#[quickcheck]
fn add_mul(a: i16, b: i16, c: i16) -> TestResult {
	if a < 0 || b < 0 || c < 0 { return TestResult::discard(); }
	let (a, b, c) = (a as i32, b as i32, c as i32);
	TestResult::from_bool(eval(&format!("(* (+ {a} {b}) {c})")) == ((a + b) * c).into())
}

#[quickcheck]
fn add_sub_combined(a: i16, b: i16, c: i16) -> TestResult {
	if a < 0 || b < 0 || c < 0 { return TestResult::discard(); }
	let (a, b, c) = (a as i32, b as i32, c as i32);
	TestResult::from_bool(eval(&format!("(+ (- {a} {b}) {c})")) == ((a - b) + c).into())
}

#[quickcheck]
fn mul_sub_combined(a: i16, b: i16, c: i16) -> TestResult {
	if a < 0 || b < 0 || c < 0 { return TestResult::discard(); }
	let (a, b, c) = (a as i32, b as i32, c as i32);
	TestResult::from_bool(eval(&format!("(- (* {a} {b}) {c})")) == ((a * b) - c).into())
}

#[quickcheck]
fn div_mul_combined(a: i16, b: i16, c: i16, d: i16) -> TestResult {
	if a < 0 || b < 0 || c < 0 || d < 0 || c <= d { return TestResult::discard(); }
	let (a, b, c, d) = (a as i32, b as i32, c as i32, d as i32);
	TestResult::from_bool(eval(&format!("(/ (* {a} {b}) (- {c} {d}))")) == ((a * b) / (c - d)).into())
}

#[quickcheck]
fn eq_combined(a: i16, b: i16, c: i16, d: i16) -> TestResult {
	if a < 0 || b < 0 || c < 0 || d < 0 { return TestResult::discard(); }
	let (a, b, c, d) = (a as i32, b as i32, c as i32, d as i32);
	TestResult::from_bool(eval(&format!("(= (+ {a} {b}) (- {c} {d}))")) == ((a + b) == (c - d)).into())
}

#[quickcheck]
fn lt_combined(a: i16, b: i16, c: i16, d: i16) -> TestResult {
	if a < 0 || b < 0 || c < 0 || d < 0 { return TestResult::discard(); }
	let (a, b, c, d) = (a as i32, b as i32, c as i32, d as i32);
	TestResult::from_bool(eval(&format!("(< (+ {a} {b}) (* {c} {d}))")) == ((a + b) < (c * d)).into())
}

#[quickcheck]
fn modulo_combined(a: i16, b: i16, c: i16, d: i16) -> TestResult {
	if a < 0 || b < 0 || c < 0 || d < 0 || c <= d { return TestResult::discard(); }
	let (a, b, c, d) = (a as i32, b as i32, c as i32, d as i32);
	TestResult::from_bool(eval(&format!("(% (+ {a} {b}) (- {c} {d}))")) == ((a + b) % (c - d)).into())
}

#[quickcheck]
fn all_arithmetic(a: i16, b: i16, c: i16, d: i16, e: i16) -> TestResult {
	if a < 0 || b < 0 || c < 0 || d <= 0 || e < 0 { return TestResult::discard(); }
	let (a, b, c, d, e) = (a as i32, b as i32, c as i32, d as i32, e as i32);
	TestResult::from_bool(eval(&format!("(+ (- (* {a} {b}) (/ {c} {d})) {e})")) == ((a * b) - (c / d) + e).into())
}

#[test]
fn print_returns_nil() {
	let code = "(print 42)";
	let expected = "nil".into();
	let result = eval(code);
	assert_eq!(result, expected);
}

#[test]
fn println_returns_nil() {
	let code = "(println 42)";
	let expected = "nil".into();
	let result = eval(code);
	assert_eq!(result, expected);
}

#[test]
fn print_combined() {
	let code = "(print (+ 1 2))";
	let expected = "nil".into();
	let result = eval(code);
	assert_eq!(result, expected);
}

#[test]
fn immediately_invoked_lambda() {
	let code = "((lambda [x] (* x x)) 2)";
	let expected = 4.into();
	let result = eval(code);
	assert_eq!(result, expected);
}

#[test]
fn defun() {
	let code = "(defun square [x] (* x x)) (square 2)";
	let expected = 4.into();
	let result = eval(code);
	assert_eq!(result, expected);
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
(defn fib [(i i32)]
      (cond ((<= i 1) 1)
	    (t (+ (fib (- i 1))
		  (fib (- i 2))))))
(fib 3)
"#;
	let expected = rust_fib(3).into();
	let result = eval(code);
	assert_eq!(result, expected);
}

fn check_list(list: &LispParseTree, expected: &[i32]) {
	use crate::lisp_object::LispParseTree::*;
	match (list, expected) {
		(Atom(s), []) if s == "nil" => {}
		(Pair(Integer(n), cdr), [first, rest @ ..]) if n == first => {
			check_list(cdr, rest);
		}
		_ => panic!("expected Pair"),
	}
}

#[test]
fn vec_to_list() {
	let list: LispParseTree = vec![1, 2, 3].into();
	check_list(&list, &[1, 2, 3]);
}

#[test]
fn vec_deque_to_list() {
	use std::collections::VecDeque;
	let mut v = VecDeque::new();
	v.push_back(1);
	v.push_back(2);
	v.push_back(3);
	let list: LispParseTree = v.into();
	check_list(&list, &[1, 2, 3]);
}

#[test]
fn empty_vec_to_nil() {
	let list: LispParseTree = Vec::<i32>::new().into();
	assert!(matches!(list, LispParseTree::Atom(s) if s == "nil"));
}

#[test]
fn empty_vec_deque_to_nil() {
	use std::collections::VecDeque;
	let list: LispParseTree = VecDeque::<i32>::new().into();
	assert!(matches!(list, LispParseTree::Atom(s) if s == "nil"));
}

#[test]
fn display_atom() {
	assert_eq!(format!("{}", LispParseTree::Atom("hello".into())), "hello");
}

#[test]
fn display_integer() {
	assert_eq!(format!("{}", LispParseTree::Integer(42)), "42");
}

#[test]
fn display_float() {
	assert_eq!(format!("{}", LispParseTree::Float(3.14)), "3.14");
}

#[test]
fn display_proper_list() {
	let list: LispParseTree = vec![1, 2, 3].into();
	assert_eq!(format!("{list}"), "'(1 2 3)");
}

#[test]
fn display_improper_list() {
	use crate::lisp_object::LispParseTree::*;
	let list = Pair(Box::new(Integer(1)), Box::new(Integer(2)));
	assert_eq!(format!("{list}"), "'(1 . 2)");
}

#[test]
fn display_nested_list() {
	let list: LispParseTree = vec![LispParseTree::from(vec![1i32, 2]), 3i32.into()].into();
	assert_eq!(format!("{list}"), "'((1 2) 3)");
}

#[test]
fn display_lambda() {
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None)],
		ret_ty: None,
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x] (body))");
}

#[test]
fn display_lambda_multi_arg() {
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None), ("y".into(), None)],
		ret_ty: None,
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x y] (body))");
}

#[test]
fn display_partially_typed_lambda_1() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), Some(LispType::Integer))],
		ret_ty: None,
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32)] (body))");
}

#[test]
fn display_partially_typed_lambda_2() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None)],
		ret_ty: Some(LispType::Integer),
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x] -> i32 (body))");
}

#[test]
fn display_partially_typed_lambda_3() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None), ("y".into(), Some(LispType::Integer)),],
		ret_ty: Some(LispType::Integer),
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x (y i32)] -> i32 (body))");
}

#[test]
fn display_partially_typed_lambda_4() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), None), ("y".into(), Some(LispType::Integer)),],
		ret_ty: None,
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x (y i32)] (body))");
}

#[test]
fn display_typed_lambda() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), Some(LispType::Integer))],
		ret_ty: Some(LispType::Integer),
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32)] -> i32 (body))");
}

#[test]
fn display_partial_typed_lambda() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: smallvec![("x".into(), Some(LispType::Integer)), ("y".into(), None),],
		ret_ty: Some(LispType::Named("bool".into())),
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32) y] -> bool (body))");
}
