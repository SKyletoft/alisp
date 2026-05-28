use nom::{
	IResult, Parser,
	branch::alt,
	character::complete::{char, digit1, multispace0, satisfy},
	combinator::{map, opt, recognize},
	multi::many0,
	sequence::pair,
};
use smallvec::SmallVec;

use crate::lisp_object::{LispParseTree, LispType, SmallString};

pub fn pre_evaluate_lambdas(list: LispParseTree) -> Result<LispParseTree, &'static str> {
	match list {
		LispParseTree::Pair(LispParseTree::Atom("lambda" | "λ"), mut list) => {
			let Some(LispParseTree::Array(args)) = list.next() else {
				return Err("lambda must have an argument list");
			};
			let args = args
				.into_iter()
				.map(parse_argument)
				.collect::<Result<SmallVec<_>, _>>()?;
			let Some(body_or_arrow) = list.next() else {
				return Err("lambda must have a body");
			};
			let (ret_ty, body) = match body_or_arrow {
				LispParseTree::Atom("->") => {
					let Some(LispParseTree::Atom(type_name)) = list.next() else {
						return Err("lambda return type expected after ->");
					};
					let ty = parse_type(&type_name);
					(Some(ty), list.into_iter().collect())
				}
				body => {
					let mut body_vec = vec![body];
					body_vec.extend(*list);
					(None, body_vec)
				}
			};

			Ok(LispParseTree::Lambda {
				params: args,
				ret_ty,
				body,
			})
		}
		LispParseTree::Pair(LispParseTree::Atom("macro"), _) => todo!(),
		LispParseTree::Pair(head, tail) => {
			let head = pre_evaluate_lambdas(*head)?;
			let tail = pre_evaluate_lambdas(*tail)?;
			Ok(LispParseTree::Pair(Box::new(head), Box::new(tail)))
		}
		LispParseTree::Array(lisp_parse_trees) => {
			let res = lisp_parse_trees
				.into_iter()
				.map(pre_evaluate_lambdas)
				.collect::<Result<Vec<_>, _>>()?
				.into_boxed_slice();
			Ok(LispParseTree::Array(res))
		}
		LispParseTree::Lambda {
			params,
			ret_ty,
			body,
		} => {
			let body = body
				.into_iter()
				.map(pre_evaluate_lambdas)
				.collect::<Result<Vec<_>, _>>()?;
			Ok(LispParseTree::Lambda {
				params,
				ret_ty,
				body,
			})
		}
		LispParseTree::Quote(inner) => {
			let inner = pre_evaluate_lambdas(*inner)?;
			Ok(LispParseTree::Quote(Box::new(inner)))
		}
		LispParseTree::Macro { params, body } => {
			let body = body
				.into_iter()
				.map(pre_evaluate_lambdas)
				.collect::<Result<Vec<_>, _>>()?;
			Ok(LispParseTree::Macro { params, body })
		}
		other => Ok(other),
	}
}

pub fn parse(code: &str) -> Result<LispParseTree, String> {
	match parse_object(code) {
		Ok(("", ret)) => Ok(ret),
		Ok((s, _)) => Err(format!("Remaining text: {s}")),
		Err(e) => Err(format!("Inner error: {e:?}")),
	}
}

pub fn parse_many(
	code: &str,
) -> Result<SmallVec<[LispParseTree; 1]>, nom::Err<nom::error::Error<&str>>> {
	let mut ret = SmallVec::new();
	let mut remaining = code;
	while !remaining.is_empty() {
		(remaining, _) = multispace0(remaining)?;
		if remaining.is_empty() {
			break;
		}
		let obj;
		(remaining, obj) = parse_object(remaining)?;
		ret.push(obj);
	}
	Ok(ret)
}

fn parse_object(code: &str) -> IResult<&str, LispParseTree> {
	alt((
		parse_quote,
		parse_float,
		parse_integer,
		parse_string,
		parse_list,
		parse_array,
		// parse_map,
		parse_atom,
	))
	.parse(code)
}

fn is_atom_continue(c: char) -> bool {
	c.is_ascii_alphanumeric()
		|| [
			'_', '-', '+', '*', '/', '|', '&', '.', ';', '~', '!', '@', '`', '´', '$', '€', '£',
			'¤', '%', '#', '\\', '^', '<', '>', '=',
		]
		.contains(&c)
}

fn is_atom_start(c: char) -> bool {
	c.is_ascii_alphabetic()
		|| [
			'_', '-', '+', '*', '/', '|', '&', '.', ';', ',', '~', '!', '@', '`', '´', '$', '€',
			'£', '¤', '%', '#', '\\', '^', '<', '>', '=',
		]
		.contains(&c)
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
	recognize(pair(
		satisfy(is_atom_start),
		many0(satisfy(is_atom_continue)),
	))
	.parse(input)
}

fn parse_atom(input: &str) -> IResult<&str, LispParseTree> {
	let (rest, id) = parse_identifier(input)?;
	Ok((rest, LispParseTree::Atom(id.into())))
}

fn parse_integer(input: &str) -> IResult<&str, LispParseTree> {
	// -?[0..9]+
	map(recognize(pair(opt(char('-')), digit1)), |digits: &str| {
		LispParseTree::Integer(digits.parse().expect("Nom should've validated this?"))
	})
	.parse(input)
}

fn parse_float(input: &str) -> IResult<&str, LispParseTree> {
	// -?[0..9]+.[0..9]*
	map(
		recognize((opt(char('-')), digit1, char('.'), opt(digit1))),
		|digits: &str| LispParseTree::Float(digits.parse().expect("Nom should've validated this?")),
	)
	.parse(input)
}

fn parse_quote(input: &str) -> IResult<&str, LispParseTree> {
	let (res, _) = char('\'')(input)?;
	let (res, obj) = parse_object(res)?;
	Ok((res, LispParseTree::Quote(Box::new(obj))))
}

fn parse_string(input: &str) -> IResult<&str, LispParseTree> {
	let err = nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char));

	let mut iter = input.char_indices();
	let Some((_, '"')) = iter.next() else {
		return Err(err);
	};

	let mut out = String::new();
	let mut escaped = false;
	while let Some((idx, ch)) = iter.next() {
		if escaped {
			escaped = false;
			if ch == 'x' {
				let (_, hi) = iter.next().ok_or(err.clone())?;
				let (_, lo) = iter.next().ok_or(err.clone())?;
				let value = hi
					.to_digit(16)
					.zip(lo.to_digit(16))
					.map(|(hi, lo)| (hi << 4) | lo)
					.ok_or(err.clone())?;
				out.push(char::from_u32(value).expect("hex escape should be valid"));
				continue;
			}
			out.push(match ch {
				'"' => '"',
				'\\' => '\\',
				'0' => '\0',
				'a' => '\x07',
				'b' => '\x08',
				'f' => '\x0c',
				'n' => '\n',
				'r' => '\r',
				'v' => '\x0b',
				't' => '\t',
				other => other,
			});
			continue;
		}

		match ch {
			'\\' => escaped = true,
			'"' => {
				let rest = &input[idx + ch.len_utf8()..];
				return Ok((rest, LispParseTree::String(out)));
			}
			other => out.push(other),
		}
	}

	Err(nom::Err::Error(nom::error::Error::new(
		input,
		nom::error::ErrorKind::Char,
	)))
}

fn parse_list_gen<const OPEN: char, const CLOSE: char>(
	input: &str,
) -> IResult<&str, Vec<LispParseTree>> {
	let mut list = Vec::new();
	let (mut rem, _) = pair(char(OPEN), multispace0).parse(input)?;

	loop {
		(rem, _) = multispace0(rem)?;
		if let Ok((r, _)) = char::<&str, nom::error::Error<&str>>(CLOSE)(rem) {
			rem = r;
			break;
		}

		let item;
		(rem, item) = parse_object(rem)?;
		list.push(item);
	}

	Ok((rem, list))
}

fn parse_list(input: &str) -> IResult<&str, LispParseTree> {
	let (rem, list) = parse_list_gen::<'(', ')'>(input)?;
	Ok((rem, LispParseTree::from(list)))
}

fn parse_array(input: &str) -> IResult<&str, LispParseTree> {
	let (rem, list) = parse_list_gen::<'[', ']'>(input)?;
	Ok((rem, LispParseTree::Array(list.into_boxed_slice())))
}

pub(crate) fn parse_type(input: &str) -> LispType {
	assert!(parse_identifier(input).is_ok());
	match input {
		"i32" => LispType::Integer,
		"f64" => LispType::Float,
		id => LispType::Named(id.into()),
	}
}

fn parse_argument(obj: LispParseTree) -> Result<(SmallString, Option<LispType>), &'static str> {
	match obj {
		LispParseTree::Atom(name) => Ok((name, None)),
		LispParseTree::Pair(
			LispParseTree::Atom(name),
			LispParseTree::Pair(LispParseTree::Atom(ty), LispParseTree::Atom("nil")),
		) => {
			let ty = parse_type(&ty);
			Ok((name, Some(ty)))
		}
		_ => Err("Non-argument in argument position"),
	}
}

#[cfg(test)]
mod test {
	use quickcheck_macros::quickcheck;
	use smallstr::SmallString;
	use smallvec::smallvec;

	use crate::lisp_object::LispParseTree;

	fn atom(s: &str) -> LispParseTree {
		LispParseTree::Atom(s.into())
	}

	fn quote(l: LispParseTree) -> LispParseTree {
		LispParseTree::Quote(Box::new(l))
	}

	fn list<const N: usize>(ls: [LispParseTree; N]) -> LispParseTree {
		ls.into_iter().collect::<Vec<_>>().into()
	}

	fn array<const N: usize>(ls: [LispParseTree; N]) -> LispParseTree {
		LispParseTree::Array(Box::new(ls))
	}

	#[allow(non_upper_case_globals)]
	const int: fn(i32) -> LispParseTree = LispParseTree::Integer;

	#[allow(non_upper_case_globals)]
	const float: fn(f64) -> LispParseTree = LispParseTree::Float;

	#[test]
	fn neg_numbers() {
		let code = "(-5 -2.44)";
		let expected = list([int(-5), float(-2.44)]);
		let result = super::parse(code);
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
		let result = super::parse(&code);
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
		let result = super::parse(code);
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn quoted_int_list() {
		let code = "'(1 2 3)";
		let expected = quote(list([int(1), int(2), int(3)]));
		let result = super::parse(code);
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn int_array() {
		let code = "[1 2 3]";
		let expected = array([int(1), int(2), int(3)]);
		let result = super::parse(code);
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn quoted_int_array() {
		let code = "'[1 2 3]";
		let expected = quote(array([int(1), int(2), int(3)]));
		let result = super::parse(code);
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn whitespace_after_number() {
		let code = "123a";
		let result = dbg!(super::parse(code));
		assert!(result.is_err());
	}

	#[test]
	fn floats() {
		let code = "123.456";
		let result = dbg!(super::parse(code));
		// Parse the string instead of hardcoding the float to make sure we use the same
		// stdlib float parser and not some custom one from rustc
		let expected = float(code.parse().unwrap());
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn lambda_no_args() {
		let result = super::parse("(lambda [] body)").unwrap();
		assert_eq!(result, list([atom("lambda"), array([]), atom("body")]));
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![],
				ret_ty: None,
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn define_lambda_no_args() {
		let result = super::parse("(set 'f (lambda [] body))").unwrap();
		assert_eq!(
			result,
			list([
				atom("set"),
				quote(atom("f")),
				list([atom("lambda"), array([]), atom("body")])
			])
		);
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			list([
				atom("set"),
				quote(atom("f")),
				LispParseTree::Lambda {
					params: smallvec![],
					ret_ty: None,
					body: vec![atom("body")],
				}
			])
		);
	}

	#[test]
	fn lambda_in_lambda() {
		let result = super::parse("(set 'f (lambda [] (set 'g (lambda [x] x))))").unwrap();
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
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			list([
				atom("set"),
				quote(atom("f")),
				LispParseTree::Lambda {
					params: smallvec![],
					ret_ty: None,
					body: vec![list([
						atom("set"),
						quote(atom("g")),
						LispParseTree::Lambda {
							params: smallvec![("x".into(), None)],
							ret_ty: None,
							body: vec![atom("x")]
						}
					])]
				}
			])
		);
	}

	#[test]
	fn lambda_one_arg_no_types() {
		let result = super::parse("(lambda [x] body)").unwrap();
		assert_eq!(
			result,
			list([atom("lambda"), array([atom("x")]), atom("body")])
		);
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None)],
				ret_ty: None,
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn lambda_typed_untyped_args_with_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [(x i32) y] -> bool body)").unwrap();
		assert_eq!(
			result,
			vec![
				atom("lambda"),
				LispParseTree::Array(Box::new([vec![atom("x"), atom("i32")].into(), atom("y")])),
				atom("->"),
				LispParseTree::from("bool"),
				atom("body")
			]
			.into()
		);
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), Some(LispType::Integer)), ("y".into(), None),],
				ret_ty: Some(LispType::Named("bool".into())),
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn lambda_one_typed_arg_with_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [(x i32)] -> i32 body)").unwrap();
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
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), Some(LispType::Integer))],
				ret_ty: Some(LispType::Integer),
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn lambda_untyped_then_typed_args_no_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [x (y i32)] body)").unwrap();
		assert_eq!(
			result,
			list([
				atom("lambda"),
				array([atom("x"), list([atom("y"), atom("i32")])]),
				atom("body")
			])
		);
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None), ("y".into(), Some(LispType::Integer)),],
				ret_ty: None,
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn lambda_untyped_then_typed_args_with_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [x (y i32)] -> i32 body)").unwrap();
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
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None), ("y".into(), Some(LispType::Integer)),],
				ret_ty: Some(LispType::Integer),
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn lambda_one_arg_no_type_with_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [x] -> i32 body)").unwrap();
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
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None)],
				ret_ty: Some(LispType::Integer),
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn lambda_one_typed_arg_no_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [(x i32)] body)").unwrap();
		assert_eq!(
			result,
			vec![
				atom("lambda"),
				array([list([atom("x"), atom("i32")])]),
				atom("body")
			]
			.into()
		);
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), Some(LispType::Integer))],
				ret_ty: None,
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn lambda_two_untyped_args_no_return() {
		let result = super::parse("(lambda [x y] body)").unwrap();
		assert_eq!(
			result,
			list([atom("lambda"), array([atom("x"), atom("y")]), atom("body")])
		);
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None), ("y".into(), None)],
				ret_ty: None,
				body: vec![atom("body")],
			}
		);
	}

	#[test]
	fn lambda_two_statements() {
		let result = super::parse("(lambda [x] (println x) (+ x 1))").unwrap();
		assert_eq!(
			result,
			list([
				atom("lambda"),
				array([atom("x")]),
				list([atom("println"), atom("x")]),
				list([atom("+"), atom("x"), int(1)])
			])
		);
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None)],
				ret_ty: None,
				body: vec![
					list([atom("println"), atom("x")]),
					list([atom("+"), atom("x"), int(1)])
				],
			}
		);
	}

	#[test]
	fn string() {
		let code = "\"hi\"";
		let expected = LispParseTree::String("hi".into());
		let result = super::parse(code);
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
		let result = super::parse(&buffer);
		assert_eq!(Ok(LispParseTree::String(expected)), result);
	}

	#[test]
	fn macro_no_args() {
		let result = super::parse("(macro [] body)").unwrap();
		assert_eq!(result, list([atom("macro"), array([]), atom("body")]));
		let with_lambdas = super::pre_evaluate_lambdas(result).unwrap();
		assert_eq!(
			with_lambdas,
			LispParseTree::Macro {
				params: smallvec![],
				body: vec![atom("body")],
			}
		);
	}
}
