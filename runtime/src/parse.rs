use std::assert_matches;

use nom::{
	IResult, Parser,
	branch::alt,
	character::complete::{char, digit1, multispace0, satisfy},
	combinator::{map, opt, recognize},
	multi::many0,
	sequence::pair,
};
use smallvec::SmallVec;

use crate::lisp_object::{LambdaArgs, LispParseTree, LispType, MacroArgs, SmallString};

pub(crate) fn parse_lambda_args(
	mut old: SmallVec<[(SmallString, Option<LispType>); 1]>,
) -> Result<LambdaArgs, &'static str> {
	let ret = match old.iter().position(|(c, _)| c == "&") {
		Some(idx) if old.iter().rev().position(|(c, _)| c == "&") != Some(old.len() - idx - 1) => {
			return Err("Lambda args may not contain multiple var-args sections");
		}
		Some(1) => {
			let rest = Some(old.remove(0));
			let and = old.remove(0);
			assert_matches!(and, ("&", None));
			let post = old;
			LambdaArgs {
				rest,
				post,
				..Default::default()
			}
		}
		Some(idx) if idx == old.len() - 1 => {
			let and = old.pop().expect("idx = len - 1 => len != 0");
			let rest = Some(old.pop().expect("idx = len - 1 => len != 0"));
			assert_matches!(and, ("&", None));
			let pre = old;
			LambdaArgs {
				pre,
				rest,
				..Default::default()
			}
		}
		Some(idx) => {
			assert!(idx < old.len());
			let pre = old[..idx - 1].into();
			let post = old[idx + 1..].into();
			let rest = Some(old[idx - 1].clone());
			LambdaArgs { pre, rest, post }
		}
		None => LambdaArgs {
			pre: old,
			..Default::default()
		},
	};
	Ok(ret)
}

pub(crate) fn parse_macro_args(
	mut old: SmallVec<[SmallString; 1]>,
) -> Result<MacroArgs, &'static str> {
	let ret = match old.iter().position(|c| c == "&") {
		Some(idx) if old.iter().rev().position(|c| c == "&") != Some(old.len() - idx - 1) => {
			return Err("Lambda args may not contain multiple var-args sections");
		}
		Some(1) => {
			let rest = Some(old.remove(0));
			let and = old.remove(0);
			assert_eq!(and, "&");
			let post = old;
			MacroArgs {
				rest,
				post,
				..Default::default()
			}
		}
		Some(idx) if idx == old.len() - 1 => {
			let and = old.pop().expect("idx = len - 1 => len != 0");
			let rest = Some(old.pop().expect("idx = len - 1 => len != 0"));
			assert_eq!(and, "&");
			let pre = old;
			MacroArgs {
				pre,
				rest,
				..Default::default()
			}
		}
		Some(idx) => {
			assert!(idx < old.len());
			let pre = old[..idx - 1].into();
			let post = old[idx + 1..].into();
			let rest = Some(old[idx - 1].clone());
			MacroArgs { pre, rest, post }
		}
		None => MacroArgs {
			pre: old,
			..Default::default()
		},
	};
	Ok(ret)
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
	let (rem, _) = multispace0(code)?;
	alt((
		parse_unquote,
		parse_quote,
		parse_quasiquote,
		parse_float,
		parse_integer,
		parse_string,
		parse_list,
		parse_array,
		// parse_map,
		parse_atom,
	))
	.parse(rem)
}

fn is_atom_continue(c: char) -> bool {
	c.is_ascii_alphanumeric()
		|| [
			'_', '-', '+', '*', '/', '|', '.', ';', '~', '!', '@', '`', '´', '$', '€', '£', '¤',
			'%', '#', '\\', '^', '<', '>', '=',
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

fn parse_unquote(input: &str) -> IResult<&str, LispParseTree> {
	let (res, _) = char(',')(input)?;
	let (res, obj) = parse_object(res)?;
	Ok((res, LispParseTree::Unquote(Box::new(obj))))
}

fn parse_quote(input: &str) -> IResult<&str, LispParseTree> {
	let (res, _) = char('\'')(input)?;
	let (res, obj) = parse_object(res)?;
	Ok((res, LispParseTree::Quote(Box::new(obj))))
}

fn parse_quasiquote(input: &str) -> IResult<&str, LispParseTree> {
	let (res, _) = char('`')(input)?;
	let (res, obj) = parse_object(res)?;
	Ok((res, LispParseTree::Quasiquote(Box::new(obj))))
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
		"atom" => LispType::Atom,
		id => LispType::Named(id.into()),
	}
}
