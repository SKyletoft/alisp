use std::collections::VecDeque;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum LispObject {
	Atom(String),
	Integer(i32 /* TODO: Bigints */),
	Float(f64),
	Pair(Box<LispObject>, Box<LispObject>),
	Lambda {
		args: Vec<(String, Option<LispType>)>,
		ret_ty: Option<LispType>,
		body: Box<LispObject>,
	},
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum LispType {
	Named(String),
}

impl From<String> for LispObject {
	fn from(s: String) -> Self {
		LispObject::Atom(s)
	}
}

impl From<&str> for LispObject {
	fn from(s: &str) -> Self {
		LispObject::Atom(s.to_string())
	}
}

impl From<i32> for LispObject {
	fn from(n: i32) -> Self {
		LispObject::Integer(n)
	}
}

impl From<f64> for LispObject {
	fn from(n: f64) -> Self {
		LispObject::Float(n)
	}
}

impl From<(LispObject, LispObject)> for LispObject {
	fn from((car, cdr): (LispObject, LispObject)) -> Self {
		LispObject::Pair(Box::new(car), Box::new(cdr))
	}
}

impl<T: Into<LispObject>> From<Vec<T>> for LispObject {
	fn from(v: Vec<T>) -> Self {
		let nil = LispObject::Atom("nil".to_string());
		v.into_iter()
			.map(Into::into)
			.rfold(nil, |acc, obj| LispObject::Pair(Box::new(obj), Box::new(acc)))
	}
}

impl fmt::Display for LispType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			LispType::Named(s) => write!(f, "{s}"),
		}
	}
}

impl fmt::Display for LispObject {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			LispObject::Atom(s) => write!(f, "{s}"),
			LispObject::Integer(n) => write!(f, "{n}"),
			LispObject::Float(n) => write!(f, "{n}"),
			LispObject::Pair(car, cdr) => {
				write!(f, "({car}")?;
				display_cdr(f, cdr)?;
				write!(f, ")")
			}
		}
	}
}

fn display_cdr(f: &mut fmt::Formatter<'_>, cdr: &LispObject) -> fmt::Result {
	match cdr {
		LispObject::Atom(s) if s == "nil" => Ok(()),
		LispObject::Pair(car, cdr) => {
			write!(f, " {car}")?;
			display_cdr(f, cdr)
		}
		other => write!(f, " . {other}"),
	}
}

impl<T: Into<LispObject>> From<VecDeque<T>> for LispObject {
	fn from(v: VecDeque<T>) -> Self {
		let nil = LispObject::Atom("nil".to_string());
		v.into_iter()
			.map(Into::into)
			.rfold(nil, |acc, obj| LispObject::Pair(Box::new(obj), Box::new(acc)))
	}
}
