use std::{collections::VecDeque, fmt};

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
		v.into_iter().map(Into::into).rfold(nil, |acc, obj| {
			LispObject::Pair(Box::new(obj), Box::new(acc))
		})
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
			LispObject::Pair(car, cdr) => {
				write!(f, "'")?;
				write_pair(f, car, cdr)
			}
			other => write_elem(f, other),
		}
	}
}

fn write_elem(f: &mut fmt::Formatter<'_>, obj: &LispObject) -> fmt::Result {
	match obj {
		LispObject::Atom(s) => write!(f, "{s}"),
		LispObject::Integer(n) => write!(f, "{n}"),
		LispObject::Float(n) => write!(f, "{n}"),
		LispObject::Pair(car, cdr) => write_pair(f, car, cdr),
		LispObject::Lambda { args, ret_ty, body } => {
			write!(f, "(λ [")?;
			for (i, (name, ty)) in args.iter().enumerate() {
				if i > 0 {
					write!(f, " ")?;
				}
				match ty {
					Some(ty) => write!(f, "({name} {ty})")?,
					None => write!(f, "{name}")?,
				}
			}
			write!(f, "]")?;
			if let Some(ret_ty) = ret_ty {
				write!(f, " -> {ret_ty}")?;
			}
			write!(f, " ")?;
			write_elem(f, body)?;
			write!(f, ")")
		}
	}
}

fn write_pair(f: &mut fmt::Formatter<'_>, car: &LispObject, cdr: &LispObject) -> fmt::Result {
	write!(f, "(")?;
	write_elem(f, car)?;
	write_cdr(f, cdr)?;
	write!(f, ")")
}

fn write_cdr(f: &mut fmt::Formatter<'_>, cdr: &LispObject) -> fmt::Result {
	match cdr {
		LispObject::Atom(s) if s == "nil" => Ok(()),
		LispObject::Pair(car, cdr) => {
			write!(f, " ")?;
			write_elem(f, car)?;
			write_cdr(f, cdr)
		}
		other => {
			write!(f, " . ")?;
			write_elem(f, other)
		}
	}
}

impl<T: Into<LispObject>> From<VecDeque<T>> for LispObject {
	fn from(v: VecDeque<T>) -> Self {
		let nil = LispObject::Atom("nil".to_string());
		v.into_iter().map(Into::into).rfold(nil, |acc, obj| {
			LispObject::Pair(Box::new(obj), Box::new(acc))
		})
	}
}
