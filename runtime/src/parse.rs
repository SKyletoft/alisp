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

#[allow(clippy::result_unit_err)]
pub fn parse(code: &str) -> Result<LispParseTree, String> {
	match parse_object(code) {
		Ok(("", ret)) => Ok(ret),
		Ok((s, _)) => Err(format!("Remaining text: {s}")),
		Err(e) => Err(format!("Inner error: {e:?}")),
	}
}

#[allow(clippy::result_unit_err)]
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
		parse_lambda,
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

fn parse_type(input: &str) -> IResult<&str, LispType> {
	match input.as_bytes() {
		[b'i', b'3', b'2', ..] => Ok((&input[3..], LispType::Integer)),
		[b'f', b'6', b'4', ..] => Ok((&input[3..], LispType::Float)),
		_ => parse_identifier(input).map(|(r, id)| (r, LispType::Named(id.into()))),
	}
}

fn parse_argument(
	obj: LispParseTree,
) -> Result<(SmallString, Option<LispType>), nom::Err<nom::error::Error<&'static str>>> {
	match obj {
		LispParseTree::Atom(name) => Ok((name, None)),
		LispParseTree::Pair(
			LispParseTree::Atom(name),
			LispParseTree::Pair(LispParseTree::Atom(ty), LispParseTree::Atom("nil")),
		) => {
			let Ok(("", ty)) = parse_type(&ty) else {
				return Err(nom::Err::Error(nom::error::Error::new(
					"Unparseable type name",
					nom::error::ErrorKind::Tag,
				)));
			};
			Ok((name, Some(ty)))
		}
		_ => Err(nom::Err::Error(nom::error::Error::new(
			"Non-argument in argument position",
			nom::error::ErrorKind::Tag,
		))),
	}
}

fn parse_lambda(input: &str) -> IResult<&str, LispParseTree> {
	let (rem, mut list) = parse_list(input)?;
	let Some(LispParseTree::Atom("lambda" | "λ")) = list.next() else {
		return Err(err("lambda list was empty?"));
	};
	let Some(args) = list.next() else {
		return Err(err("lambda must have an argument list"));
	};
	let args = args
		.into_iter()
		.map(parse_argument)
		.take(10_000)
		.collect::<Result<SmallVec<_>, _>>()?;
	let Some(body_or_arrow) = list.next() else {
		return Err(err("lambda must have a body"));
	};
	let (ret_ty, body) = match body_or_arrow {
		LispParseTree::Atom("->") => {
			let Some(LispParseTree::Atom(type_name)) = list.next() else {
				return Err(err("lambda return type expected after ->"));
			};
			let ty = parse_type(&type_name)
				.map_err(|_| err("lambda return type is unparseable"))?
				.1;
			(Some(ty), list.into_iter().collect())
		}
		body => {
			let mut body_vec = vec![body];
			body_vec.extend(list.into_iter());
			(None, body_vec)
		}
	};
	let ret = LispParseTree::Lambda {
		params: args,
		ret_ty,
		body,
	};
	Ok((rem, ret))
}

fn err(msg: &str) -> nom::Err<nom::error::Error<&str>> {
	nom::Err::Error(nom::error::Error::new(msg, nom::error::ErrorKind::Tag))
}

#[cfg(test)]
mod test {
	use quickcheck_macros::quickcheck;
	use smallstr::SmallString;
	use smallvec::smallvec;

	use crate::lisp_object::LispParseTree;

	#[test]
	fn neg_numbers() {
		let code = "(-5 -2.44)";
		let expected = LispParseTree::from(vec![
			LispParseTree::Integer(-5),
			LispParseTree::Float(-2.44),
		]);
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
		let expected = LispParseTree::from(vec![1, 2, 3]);
		let result = super::parse(code);
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn quoted_int_list() {
		let code = "'(1 2 3)";
		let expected = LispParseTree::Quote(Box::new(LispParseTree::from(vec![1, 2, 3])));
		let result = super::parse(code);
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn int_array() {
		let code = "[1 2 3]";
		let expected = LispParseTree::Array(
			vec![
				LispParseTree::Integer(1),
				LispParseTree::Integer(2),
				LispParseTree::Integer(3),
			]
			.into_boxed_slice(),
		);
		let result = super::parse(code);
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn quoted_int_array() {
		let code = "'[1 2 3]";
		let expected = LispParseTree::Quote(Box::new(LispParseTree::Array(
			vec![
				LispParseTree::Integer(1),
				LispParseTree::Integer(2),
				LispParseTree::Integer(3),
			]
			.into_boxed_slice(),
		)));
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
	fn float() {
		let code = "123.456";
		let result = dbg!(super::parse(code));
		// Parse the string instead of hardcoding the float to make sure we use the same
		// stdlib float parser and not some custom one from rustc
		let expected = LispParseTree::Float(code.parse().unwrap());
		assert_eq!(result, Ok(expected));
	}

	#[test]
	fn lambda_no_args() {
		let result = super::parse("(lambda [] body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![],
				ret_ty: None,
				body: vec![LispParseTree::Atom("body".into())].into(),
			}
		);
	}

	#[test]
	fn lambda_one_arg_no_types() {
		let result = super::parse("(lambda [x] body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None)],
				ret_ty: None,
				body: vec![LispParseTree::Atom("body".into())],
			}
		);
	}

	#[test]
	fn lambda_typed_untyped_args_with_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [(x i32) y] -> bool body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), Some(LispType::Integer)), ("y".into(), None),],
				ret_ty: Some(LispType::Named("bool".into())),
				body: vec![LispParseTree::Atom("body".into())],
			}
		);
	}

	#[test]
	fn lambda_one_typed_arg_with_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [(x i32)] -> i32 body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), Some(LispType::Integer))],
				ret_ty: Some(LispType::Integer),
				body: vec![LispParseTree::Atom("body".into())],
			}
		);
	}

	#[test]
	fn lambda_untyped_then_typed_args_no_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [x (y i32)] body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None), ("y".into(), Some(LispType::Integer)),],
				ret_ty: None,
				body: vec![LispParseTree::Atom("body".into())],
			}
		);
	}

	#[test]
	fn lambda_untyped_then_typed_args_with_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [x (y i32)] -> i32 body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None), ("y".into(), Some(LispType::Integer)),],
				ret_ty: Some(LispType::Integer),
				body: vec![LispParseTree::Atom("body".into())],
			}
		);
	}

	#[test]
	fn lambda_one_arg_no_type_with_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [x] -> i32 body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None)],
				ret_ty: Some(LispType::Integer),
				body: vec![LispParseTree::Atom("body".into())],
			}
		);
	}

	#[test]
	fn lambda_one_typed_arg_no_return() {
		use crate::lisp_object::LispType;

		let result = super::parse("(lambda [(x i32)] body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), Some(LispType::Integer))],
				ret_ty: None,
				body: vec![LispParseTree::Atom("body".into())],
			}
		);
	}

	#[test]
	fn lambda_two_untyped_args_no_return() {
		let result = super::parse("(lambda [x y] body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None), ("y".into(), None)],
				ret_ty: None,
				body: vec![LispParseTree::Atom("body".into())],
			}
		);
	}

	#[test]
	fn lambda_two_statements() {
		let result = super::parse("(lambda [x] (println x) (+ x 1))").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: smallvec![("x".into(), None)],
				ret_ty: None,
				body: vec![
					vec![
						LispParseTree::Atom("println".into()),
						LispParseTree::Atom("x".into())
					]
					.into(),
					vec![
						LispParseTree::Atom("+".into()),
						LispParseTree::Atom("x".into()),
						LispParseTree::Integer(1)
					]
					.into()
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
}
