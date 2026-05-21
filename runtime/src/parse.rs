use nom::{
	IResult, Parser,
	branch::alt,
	bytes::complete::tag,
	character::complete::{char, digit1, multispace0, satisfy},
	combinator::{map, opt, recognize},
	multi::many0,
	sequence::pair,
};

use crate::lisp_object::{LispParseTree, LispType, SmallString};

pub fn parse(code: &str) -> Result<LispParseTree, ()> {
	match parse_object(code) {
		Ok(("", ret)) => Ok(ret),
		Ok((s, _)) => {
			eprintln!("Remaining text: {s}");
			Err(())
		}
		e => {
			let _ = dbg!(e);
			Err(())
		}
	}
}

fn parse_object(code: &str) -> IResult<&str, LispParseTree> {
	alt((
		parse_atom,
		parse_float,
		parse_integer,
		parse_lambda,
		parse_list::<'(', ')'>,
		parse_list::<'[', ']'>,
		parse_list::<'{', '}'>,
	))
	.parse(code)
}

fn is_atom_continue(c: char) -> bool {
	c.is_ascii_alphanumeric()
		|| [
			'_', '-', '+', '*', '/', '|', '&', '.', ';', ',', '~', '!', '@', '`', '´', '$', '€',
			'£', '¤', '%', '#', '\\', '^', '<', '>', '=',
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

fn parse_atom(input: &str) -> IResult<&str, LispParseTree> {
	map(
		recognize(pair(
			satisfy(is_atom_start),
			many0(satisfy(is_atom_continue)),
		)),
		|s: &str| LispParseTree::Atom(s.into()),
	)
	.parse(input)
}

fn parse_integer(input: &str) -> IResult<&str, LispParseTree> {
	// [0..9]+
	map(digit1, |digits: &str| {
		LispParseTree::Integer(digits.parse().expect("Nom should've validated this?"))
	})
	.parse(input)
}

fn parse_float(input: &str) -> IResult<&str, LispParseTree> {
	// [0..9]+.[0..9]*
	let parser = |s| -> IResult<&str, (&str, Option<&str>)> {
		let (rem, int) = digit1.parse(s)?;
		let (rem, _) = tag(".")(rem)?;
		let (rem, frac) = opt(digit1).parse(rem)?;
		Ok(dbg!((rem, (int, frac))))
	};
	let (rem, res) = dbg!(recognize(parser).parse(input)?);
	let n = LispParseTree::Float(res.parse().expect("Nom should've validated this?"));

	Ok((rem, n))
}

fn parse_list<const OPEN: char, const CLOSE: char>(input: &str) -> IResult<&str, LispParseTree> {
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

	Ok((rem, list.into()))
}

fn parse_argument(
	obj: LispParseTree,
) -> Result<(SmallString, Option<LispType>), nom::Err<nom::error::Error<&'static str>>> {
	match obj {
		LispParseTree::Atom(name) => Ok((name, None)),
		LispParseTree::Pair(LispParseTree::Atom(name), LispParseTree::Pair(LispParseTree::Atom(ty), rest))
			if *rest == LispParseTree::Atom("nil".into()) =>
		{
			Ok((name, Some(LispType::Named(ty))))
		}
		_ => Err(nom::Err::Error(nom::error::Error::new(
			"Non-argument in argument position",
			nom::error::ErrorKind::Tag,
		))),
	}
}

fn parse_lambda(input: &str) -> IResult<&str, LispParseTree> {
	dbg!(input);
	let (rem, mut list) = dbg!(parse_list::<'(', ')'>(input)?);
	let Some(LispParseTree::Atom("lambda")) = dbg!(list.next()) else {
		return Err(nom::Err::Error(nom::error::Error::new(
			"lambda list was empty?",
			nom::error::ErrorKind::Tag,
		)));
	};
	let Some(args) = dbg!(list.next()) else {
		panic!();
	};
	let args = args
		.map(parse_argument)
		.take(10_000)
		.collect::<Result<Vec<_>, _>>()?;
	let Some(body_or_arrow) = dbg!(list.next()) else {
		panic!();
	};
	let (ret_ty, body) = match body_or_arrow {
		LispParseTree::Atom("->") => {
			let Some(LispParseTree::Atom(type_name)) = dbg!(list.next()) else {
				panic!();
			};
			(Some(LispType::Named(type_name)), Box::new(list))
		}
		body => (None, Box::new(vec![body].into())),
	};
	let ret = LispParseTree::Lambda { params: args, ret_ty, body };
	Ok((rem, ret))
}

#[cfg(test)]
mod test {
	use crate::lisp_object::LispParseTree;

	#[test]
	fn int_list() {
		let code = "(1 2 3)";
		let expected = LispParseTree::from(vec![1, 2, 3]);
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
				params: vec![],
				ret_ty: None,
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
			}
		);
	}

	#[test]
	fn lambda_one_arg_no_types() {
		let result = super::parse("(lambda [x] body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: vec![("x".into(), None)],
				ret_ty: None,
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
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
				params: vec![
					("x".into(), Some(LispType::Named("i32".into()))),
					("y".into(), None),
				],
				ret_ty: Some(LispType::Named("bool".into())),
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
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
				params: vec![("x".into(), Some(LispType::Named("i32".into())))],
				ret_ty: Some(LispType::Named("i32".into())),
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
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
				params: vec![
					("x".into(), None),
					("y".into(), Some(LispType::Named("i32".into()))),
				],
				ret_ty: None,
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
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
				params: vec![
					("x".into(), None),
					("y".into(), Some(LispType::Named("i32".into()))),
				],
				ret_ty: Some(LispType::Named("i32".into())),
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
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
				params: vec![("x".into(), None)],
				ret_ty: Some(LispType::Named("i32".into())),
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
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
				params: vec![("x".into(), Some(LispType::Named("i32".into())))],
				ret_ty: None,
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
			}
		);
	}

	#[test]
	fn lambda_two_untyped_args_no_return() {
		let result = super::parse("(lambda [x y] body)").unwrap();
		assert_eq!(
			result,
			LispParseTree::Lambda {
				params: vec![("x".into(), None), ("y".into(), None)],
				ret_ty: None,
				body: Box::new(vec![LispParseTree::Atom("body".into())].into()),
			}
		);
	}
}
