use std::assert_matches;

use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use smallstr::SmallString;

use crate::{
	eval::{self, RuntimeError},
	lisp_object::{
		Env, LambdaArgs, LispObject, LispParseTree, LispType, MacroArgs, ObjectReference,
		parse_tree::{array, atom, float, int, list, quasiquote, quote, unquote},
	},
	parse,
};

fn eval(code: &str) -> Result<LispParseTree, RuntimeError> {
	let mut env = Env::wait_for_new();
	let parsed = parse::parse_many(code).unwrap();
	let res = parsed.into_iter().try_fold(env.nil(), |_, node| {
		let obj = ObjectReference::from_parse_object(node, &mut env);
		eval::eval_top(&mut env, obj)
	})?;
	let printable = crate::lisp_object::lisp_object_to_parse_tree(&env, env.get(res));
	Ok(printable)
}

fn eval_to_obj(code: &str) -> (LispObject<'static>, Env<'static, 0>) {
	let mut env = Env::wait_for_new();
	let parsed = parse::parse(code).unwrap();
	let obj = ObjectReference::from_parse_object(parsed, &mut env);
	let res = eval::eval_top(&mut env, obj).unwrap();
	let result = env.get(res).clone();
	(result, env)
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
fn defun_eval() {
	let code = "(defun square [x] (* x x)) (square 2)";
	let expected = int(4);
	let result = eval(code);
	assert_eq!(result, Ok(expected));
}

#[ignore]
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
	assert_matches!(list, LispParseTree::Atom(s) if s == "nil");
}

#[test]
fn empty_vec_deque_to_nil() {
	use std::collections::VecDeque;
	let list: LispParseTree = VecDeque::<LispParseTree>::new().into();
	assert_matches!(list, LispParseTree::Atom(s) if s == "nil");
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

#[ignore]
#[test]
fn macro_expands_in_quasiquote() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) `(m 1)";
	let expected = parse::parse("(+ 1 1)").unwrap();
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[ignore]
#[test]
fn macro_expands_in_quasiquoted_list() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) `(1 (m 1) 3)";
	let expected = parse::parse("(1 (+ 1 1) 3)").unwrap();
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[ignore]
#[test]
fn macro_does_not_expand_in_quoted_list() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) '(1 (m 1) 3)";
	let expected = parse::parse("'(1 (m 1) 3)").unwrap();
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[ignore]
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

#[ignore]
#[test]
fn macro_call_inside_quasiquote_expands() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) `(m 1)";
	let expected = list([atom("+"), int(1), int(1)]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[ignore]
#[test]
fn macro_call_inside_quasiquoted_list_expands() {
	let code = "(set 'm (macro [x] (+ 1 ,x))) `(1 (m 1) 3)";
	let expected = list([int(1), list([atom("+"), int(1), int(1)]), int(3)]);
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[ignore]
#[test]
fn macro_argument_quoted_list_stays_quoted() {
	let code = "(set 'm (macro [x] x)) (m '(1 2 3))";
	let expected = quote(list([int(1), int(2), int(3)]));
	let res = eval(code);
	assert_eq!(res, Ok(expected));
}

#[ignore]
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

#[test]
fn neg_numbers() {
	let code = "(-5 -2.44)";
	let expected = list([int(-5), float(-2.44)]);
	let result = parse::parse(code);
	assert_eq!(result, Ok(expected));
}

#[quickcheck]
fn parse_number(n: f64) {
	if !n.is_normal() {
		return;
	}
	let code = if n.fract() == 0.0 {
		format!("{n:.1}")
	} else {
		format!("{n}")
	};
	let result = parse::parse(&code);
	match result {
		Ok(LispParseTree::Float(m)) if (n - m).abs() <= f64::EPSILON => {}
		Ok(LispParseTree::Float(m)) => panic!("{m} is too far from {n}"),
		_ => panic!("Didn't parse {code:?} as float"),
	}
}

#[test]
fn int_list() {
	let code = "(1 2 3)";
	let expected = list([int(1), int(2), int(3)]);
	let result = parse::parse(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn quoted_int_list() {
	let code1 = "'(1 2 3)";
	let code2 = "' (1 2 3)";

	let expected = quote(list([int(1), int(2), int(3)]));

	let result1 = parse::parse(code1);
	let result2 = parse::parse(code2);

	assert_eq!(result1, Ok(expected.clone()));
	assert_eq!(result2, Ok(expected));
}

#[test]
fn quasiquoted_int_list() {
	let code1 = "`(1 2 3)";
	let code2 = "` (1 2 3)";

	let expected = quasiquote(list([int(1), int(2), int(3)]));

	let result1 = parse::parse(code1);
	let result2 = parse::parse(code2);

	assert_eq!(result1, Ok(expected.clone()));
	assert_eq!(result2, Ok(expected));
}

#[test]
fn int_array() {
	let code = "[1 2 3]";
	let expected = array([int(1), int(2), int(3)]);
	let result = parse::parse(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn quoted_int_array() {
	let code = "'[1 2 3]";
	let expected = quote(array([int(1), int(2), int(3)]));
	let result = parse::parse(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn quasiquoted_int_array() {
	let code = "`[1 2 3]";
	let expected = quasiquote(array([int(1), int(2), int(3)]));
	let result = parse::parse(code);
	assert_eq!(result, Ok(expected));
}

#[test]
fn whitespace_after_number() {
	let code = "123a";
	let result = parse::parse(code);
	assert!(result.is_err());
}

#[test]
fn floats() {
	let code = "123.456";
	let result = parse::parse(code);
	let expected = float(code.parse().unwrap());
	assert_eq!(result, Ok(expected));
}

#[test]
fn lambda_no_args() {
	let result = parse::parse("(lambda [] body)").unwrap();
	assert_eq!(result, list([atom("lambda"), array([]), atom("body")]));
	let (obj, env) = eval_to_obj("(lambda [] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [], rest: None, post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn define_lambda_no_args() {
	let result = parse::parse("(set 'f (lambda [] body))").unwrap();
	assert_eq!(
		result,
		list([
			atom("set"),
			quote(atom("f")),
			list([atom("lambda"), array([]), atom("body")])
		])
	);
	let (obj, env) = eval_to_obj("(lambda [] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [], rest: None, post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_in_lambda() {
	let result = parse::parse("(set 'f (lambda [] (set 'g (lambda [x] x))))").unwrap();
	assert_eq!(
		result,
		list([
			atom("set"),
			quote(atom("f")),
			list([
				atom("lambda"),
				array([]),
				list([
					atom("set"),
					quote(atom("g")),
					list([atom("lambda"), array([atom("x")]), atom("x"),])
				])
			])
		])
	);
	let (obj, env) = eval_to_obj("(lambda [x] x)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", None)], rest: None, post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("x")));
}

#[test]
fn lambda_one_arg_no_types() {
	let result = parse::parse("(lambda [x] body)").unwrap();
	assert_eq!(
		result,
		list([atom("lambda"), array([atom("x")]), atom("body")])
	);
	let (obj, env) = eval_to_obj("(lambda [x] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", None)], rest: None, post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_typed_untyped_args_with_return() {
	let result = parse::parse("(lambda [(x i32) y] -> bool body)").unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([list([atom("x"), atom("i32")]), atom("y")]),
			atom("->"),
			atom("bool"),
			atom("body")
		]
		.into()
	);
	let (obj, env) = eval_to_obj("(lambda [(x i32) y] -> bool body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", Some(LispType::Integer)), ("y", None)], rest: None, post: [] },
		ret_ty: Some(LispType::Named("bool")),
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_one_typed_arg_with_return() {
	let result = parse::parse("(lambda [(x i32)] -> i32 body)").unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([list([atom("x"), atom("i32")])]),
			atom("->"),
			atom("i32"),
			atom("body")
		]
		.into()
	);
	let (obj, env) = eval_to_obj("(lambda [(x i32)] -> i32 body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", Some(LispType::Integer))], rest: None, post: [] },
		ret_ty: Some(LispType::Integer),
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_untyped_then_typed_args_no_return() {
	let result = parse::parse("(lambda [x (y i32)] body)").unwrap();
	assert_eq!(
		result,
		list([
			atom("lambda"),
			array([atom("x"), list([atom("y"), atom("i32")])]),
			atom("body")
		])
	);
	let (obj, env) = eval_to_obj("(lambda [x (y i32)] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", None), ("y", Some(LispType::Integer))], rest: None, post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_untyped_then_typed_args_with_return() {
	let result = parse::parse("(lambda [x (y i32)] -> i32 body)").unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([atom("x"), list([atom("y"), atom("i32")])]),
			atom("->"),
			atom("i32"),
			atom("body")
		]
		.into()
	);
	let (obj, env) = eval_to_obj("(lambda [x (y i32)] -> i32 body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", None), ("y", Some(LispType::Integer))], rest: None, post: [] },
		ret_ty: Some(LispType::Integer),
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_one_arg_no_type_with_return() {
	let result = parse::parse("(lambda [x] -> i32 body)").unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([atom("x")]),
			atom("->"),
			atom("i32"),
			atom("body")
		]
		.into()
	);
	let (obj, env) = eval_to_obj("(lambda [x] -> i32 body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", None)], rest: None, post: [] },
		ret_ty: Some(LispType::Integer),
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_one_typed_arg_no_return() {
	let result = parse::parse("(lambda [(x i32)] body)").unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([list([atom("x"), atom("i32")])]),
			atom("body")
		]
		.into()
	);
	let (obj, env) = eval_to_obj("(lambda [(x i32)] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", Some(LispType::Integer))], rest: None, post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_two_untyped_args_no_return() {
	let result = parse::parse("(lambda [x y] body)").unwrap();
	assert_eq!(
		result,
		list([atom("lambda"), array([atom("x"), atom("y")]), atom("body")])
	);
	let (obj, env) = eval_to_obj("(lambda [x y] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", None), ("y", None)], rest: None, post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn lambda_two_statements() {
	let result = parse::parse("(lambda [x] (println x) (+ x 1))").unwrap();
	assert_eq!(
		result,
		list([
			atom("lambda"),
			array([atom("x")]),
			list([atom("println"), atom("x")]),
			list([atom("+"), atom("x"), int(1)])
		])
	);
	let (obj, _env) = eval_to_obj("(lambda [x] (println x) (+ x 1))");
	assert_matches!(
		obj,
		LispObject::Lambda {
			params: LambdaArgs {
				pre: [("x", None)],
				rest: None,
				post: []
			},
			ret_ty: None,
			body: [_, _],
		}
	);
}

#[test]
fn string() {
	let code = "\"hi\"";
	let expected = LispParseTree::String("hi".into());
	let result = parse::parse(code);
	assert_eq!(Ok(expected), result);
}

#[quickcheck]
fn any_string(s: [u8; 12]) {
	let mut buffer = SmallString::<[u8; 26]>::new();
	let mut expected = String::new();
	buffer.push('"');
	for c in s.into_iter() {
		match c {
			b'"' => {
				buffer.push('\\');
				buffer.push('"');
				expected.push('"');
			}
			b'\\' => {
				buffer.push('\\');
				buffer.push('\\');
				expected.push('\\');
			}
			b'\n' => {
				buffer.push('\\');
				buffer.push('n');
				expected.push('\n');
			}
			b'\r' => {
				buffer.push('\\');
				buffer.push('r');
				expected.push('\r');
			}
			b'\t' => {
				buffer.push('\\');
				buffer.push('t');
				expected.push('\t');
			}
			c if c.is_ascii_graphic() || c == b' ' => {
				buffer.push(c as char);
				expected.push(c as char);
			}
			c if (0xA0..=0xFF).contains(&c) => {
				buffer.push(char::from(c));
				expected.push(char::from(c));
			}
			c => {
				buffer.push('\\');
				buffer.push('x');
				buffer.push(char::from_digit((c >> 4) as u32, 16).expect("hex digit"));
				buffer.push(char::from_digit((c & 0x0F) as u32, 16).expect("hex digit"));
				expected.push(c as char);
			}
		}
	}
	buffer.push('"');
	let result = parse::parse(&buffer);
	assert_eq!(Ok(LispParseTree::String(expected)), result);
}

#[test]
fn macro_no_args() {
	let result = parse::parse("(macro [] body)").unwrap();
	assert_eq!(result, list([atom("macro"), array([]), atom("body")]));
	let (obj, env) = eval_to_obj("(macro [] body)");
	assert_matches!(obj, LispObject::Macro {
		params: MacroArgs { pre: [], rest: None, post: [] },
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn test_unquote() {
	let code1 = ",x";
	let res1 = parse::parse(code1);

	let code2 = ", x";
	let res2 = parse::parse(code2);

	let expected = LispParseTree::Unquote(Box::new(LispParseTree::Atom("x".into())));

	assert_eq!(res1, Ok(expected.clone()));
	assert_eq!(res2, Ok(expected));
}

#[test]
fn variable_args() {
	let code = "(lambda [rest&] body)";
	let result = parse::parse(code).unwrap();
	assert_eq!(
		result,
		list([
			atom("lambda"),
			array([atom("rest"), atom("&")]),
			atom("body")
		])
	);
	let (obj, env) = eval_to_obj("(lambda [rest&] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [], rest: Some(("rest", None)), post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn variable_args_typed() {
	let code = "(lambda [(rest i32)&] body)";
	let result = parse::parse(code).unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([list([atom("rest"), atom("i32")]), atom("&")]),
			atom("body"),
		]
		.into()
	);
	let (obj, env) = eval_to_obj("(lambda [(rest i32)&] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [], rest: Some(("rest", Some(LispType::Integer))), post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn variable_args_head() {
	let code = "(lambda [x rest&] body)";
	let result = parse::parse(code).unwrap();
	assert_eq!(
		result,
		list([
			atom("lambda"),
			array([atom("x"), atom("rest"), atom("&")]),
			atom("body")
		])
	);
	let (obj, env) = eval_to_obj("(lambda [x rest&] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", None)], rest: Some(("rest", None)), post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn variable_args_head_typed() {
	let code = "(lambda [(x f64) (rest i32)&] body)";
	let result = parse::parse(code).unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([
				list([atom("x"), atom("f64")]),
				list([atom("rest"), atom("i32")]),
				atom("&"),
			]),
			atom("body"),
		]
		.into()
	);
	let (obj, env) = eval_to_obj("(lambda [(x f64) (rest i32)&] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [("x", Some(LispType::Float))], rest: Some(("rest", Some(LispType::Integer))), post: [] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn variable_args_tail() {
	let code = "(lambda [rest& tail] body)";
	let result = parse::parse(code).unwrap();
	assert_eq!(
		result,
		list([
			atom("lambda"),
			array([atom("rest"), atom("&"), atom("tail")]),
			atom("body")
		])
	);
	let (obj, env) = eval_to_obj("(lambda [rest& tail] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [], rest: Some(("rest", None)), post: [("tail", None)] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn variable_args_tail_typed() {
	let code = "(lambda [(rest atom)& (tail i32)] body)";
	let result = parse::parse(code).unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([
				list([atom("rest"), atom("atom")]),
				atom("&"),
				list([atom("tail"), atom("i32")]),
			]),
			atom("body"),
		]
		.into()
	);
	let (obj, env) = eval_to_obj("(lambda [(rest atom)& (tail i32)] body)");
	assert_matches!(obj, LispObject::Lambda {
		params: LambdaArgs { pre: [], rest: Some(("rest", Some(LispType::Atom))), post: [("tail", Some(LispType::Integer))] },
		ret_ty: None,
		body: [body_atom],
	} if matches!(body_atom.get(&env), LispObject::Atom("body")));
}

#[test]
fn variable_args_head_tail() {
	let code = "(lambda [first rest& last] body)";
	let result = parse::parse(code).unwrap();
	assert_eq!(
		result,
		list([
			atom("lambda"),
			array([atom("first"), atom("rest"), atom("&"), atom("last")]),
			atom("body"),
		])
	);
	let res = eval("((lambda [x y] (+ x y)) 3 4)");
	assert_eq!(res, Ok(int(7)));
}

#[test]
fn variable_args_head_tail_typed() {
	let code = "(lambda [(first i32) (rest i32)& (last i32)] body)";
	let result = parse::parse(code).unwrap();
	assert_eq!(
		result,
		vec![
			atom("lambda"),
			array([
				list([atom("first"), atom("i32")]),
				list([atom("rest"), atom("i32")]),
				atom("&"),
				list([atom("last"), atom("i32")]),
			]),
			atom("body"),
		]
		.into()
	);
	let res = eval("((lambda [(x i32) (y i32)] (+ x y)) 3 4)");
	assert_eq!(res, Ok(int(7)));
}

#[test]
fn bad_variable_args_only_one() {
	let code = "(lambda [x& y&] body)";
	let result = eval(code);
	assert_eq!(
		result,
		Err(RuntimeError::BrokenLambda {
			msg: "Lambda args may not contain multiple var-args sections"
		})
	);
}

#[test]
fn bad_variable_args_only_one_typed() {
	let code = "(lambda [(x atom)& (y i32)&] body)";
	let result = eval(code);
	assert_eq!(
		result,
		Err(RuntimeError::BrokenLambda {
			msg: "Lambda args may not contain multiple var-args sections"
		})
	);
}

#[test]
fn call_varargs() {
	let code = r#"
		(set 'f (lambda [x (rest i32)& y]
				(println rest)
				(+ x y)))
		(set 'res (f 1 2 3 4))
		(println res)
		res
	"#;
	let res = eval(code);
	let expected = int(1 + 4);
	assert_eq!(res, Ok(expected));
}

#[test]
fn cant_access_out_of_scope_vars() {
	let code = r#"
		(set 'f (lambda [x] z))
		(set 'g (lambda [x] (set 'z 5) (f x)))
		(g 3)
	"#;
	let res = eval(code);
	let expected = Err(RuntimeError::UndefinedVariable("z".into()));
	assert_eq!(res, expected);
}

#[test]
fn macros_have_no_scope() {
	let code = r#"
		(set 'f (macro [x] z))
		(set 'g (lambda [x] (set 'z 5) (f x)))
		(g 3)
	"#;
	let res = eval(code);
	let expected = int(5);
	assert_eq!(res, Ok(expected));
}

#[test]
fn can_access_captures() {
	let code = r#"
		(set 'g (lambda [] (set 'z 5) (lambda [] z)))
		((g))
	"#;
	let res = eval(code);
	let expected = int(5);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macros_dont_capture() {
	let code = r#"
		(set 'g (lambda [] (set 'z 5) (macro [] z)))
		((g))
	"#;
	let res = eval(code);
	let expected = Err(RuntimeError::UndefinedVariable("z".into()));
	assert_eq!(res, expected);
}

#[test]
fn functions_do_evaluate_args() {
	let code = r#"
		(set 'x 0)
		(defun f [g]
			(set 'a x)
			g
			(set 'b x)
			g
			(set 'c x)
			[a b c]
		)
		(f ((lambda [] (set 'x (+ x 1)))))
	"#;
	let res = eval(code);
	let expected = array([int(1), int(1), int(1)]);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macros_dont_evaluate_args() {
	let code = r#"
		(set 'x 0)
		(defun f [g]
			(set 'a x)
			g
			(set 'b x)
			g
			(set 'c x)
			[a b c]
		)
		(f ((macro [] `(set 'x (+ x 1)))))
	"#;
	let res = eval(code);
	let expected = array([int(0), int(1), int(2)]);
	assert_eq!(res, Ok(expected));
}

#[test]
fn macros_evaluate_at_lambda_resolve_time() {
	let code = r#"
		(set 'x 0)
		(defun f [g]
			(set 'a x)
			g
			(set 'b x)
			g
			(set 'c x)
			[a b c]
		)
		(f ((macro [] (set 'x (+ x 1)))))
	assert_eq!(res, Ok(expected));
}

#[test]
fn setq_many() {
	let code = r#"
		(setq x 5
		      y 3)
		(+ x y)
	"#;
	let res = eval(code);
	let expected = array([int(3), int(3), int(3)]);
	assert_eq!(res, Ok(expected));
}
