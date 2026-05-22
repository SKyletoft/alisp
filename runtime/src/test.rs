use crate::{eval, lisp_object::{Env, LispParseTree}, parse};

pub fn eval(code: &str) -> LispParseTree {
	let parsed = parse::parse(code).unwrap();
	let mut env = Env::new().unwrap();
	eval::eval(&parsed, &mut env).unwrap()
}

#[test]
fn add() {
	let code = "(+ 1 2)";
	let expected = 3.into();
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
		params: vec![("x".into(), None)],
		ret_ty: None,
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x] (body))");
}

#[test]
fn display_lambda_multi_arg() {
	let lambda = LispParseTree::Lambda {
		params: vec![("x".into(), None), ("y".into(), None)],
		ret_ty: None,
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x y] (body))");
}

#[test]
fn display_partially_typed_lambda_1() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: vec![("x".into(), Some(LispType::Named("i32".into())))],
		ret_ty: None,
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32)] (body))");
}

#[test]
fn display_partially_typed_lambda_2() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: vec![("x".into(), None)],
		ret_ty: Some(LispType::Named("i32".into())),
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x] -> i32 (body))");
}

#[test]
fn display_partially_typed_lambda_3() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: vec![
			("x".into(), None),
			("y".into(), Some(LispType::Named("i32".into()))),
		],
		ret_ty: Some(LispType::Named("i32".into())),
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x (y i32)] -> i32 (body))");
}

#[test]
fn display_partially_typed_lambda_4() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: vec![
			("x".into(), None),
			("y".into(), Some(LispType::Named("i32".into()))),
		],
		ret_ty: None,
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [x (y i32)] (body))");
}

#[test]
fn display_typed_lambda() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: vec![("x".into(), Some(LispType::Named("i32".into())))],
		ret_ty: Some(LispType::Named("i32".into())),
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32)] -> i32 (body))");
}

#[test]
fn display_partial_typed_lambda() {
	use crate::lisp_object::LispType;
	let lambda = LispParseTree::Lambda {
		params: vec![
			("x".into(), Some(LispType::Named("i32".into()))),
			("y".into(), None),
		],
		ret_ty: Some(LispType::Named("bool".into())),
		body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
	};
	assert_eq!(format!("{lambda}"), "(λ [(x i32) y] -> bool (body))");
}
