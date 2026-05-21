pub use parse_tree::{LispParseTree, LispType};
pub use runtime_object::{Env, LispObject, ObjectReference};

pub type SmallString = smallstr::SmallString<[u8; 23]>;

mod parse_tree {
	use std::{collections::VecDeque, fmt};

	use super::SmallString;

	#[derive(Debug, PartialEq, Clone, derive_more::From)]
	pub enum LispParseTree {
		Atom(SmallString),
		Integer(i32 /* TODO: Bigints */),
		Float(f64),
		Pair(Box<LispParseTree>, Box<LispParseTree>),
		Lambda {
			params: Vec<(SmallString, Option<LispType>)>,
			ret_ty: Option<LispType>,
			body: Box<LispParseTree>,
		},
	}

	impl From<String> for LispParseTree {
		fn from(s: String) -> Self {
			LispParseTree::Atom(s.into())
		}
	}

	impl From<&str> for LispParseTree {
		fn from(s: &str) -> Self {
			LispParseTree::Atom(s.into())
		}
	}

	impl From<(LispParseTree, LispParseTree)> for LispParseTree {
		fn from((car, cdr): (LispParseTree, LispParseTree)) -> Self {
			LispParseTree::Pair(Box::new(car), Box::new(cdr))
		}
	}

	impl<T: Into<LispParseTree>> From<Vec<T>> for LispParseTree {
		fn from(v: Vec<T>) -> Self {
			let nil = LispParseTree::Atom("nil".into());
			v.into_iter().map(Into::into).rfold(nil, |acc, obj| {
				LispParseTree::Pair(Box::new(obj), Box::new(acc))
			})
		}
	}

	impl fmt::Display for LispParseTree {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			match self {
				LispParseTree::Pair(car, cdr) => {
					write!(f, "'")?;
					write_pair(f, car, cdr)
				}
				other => write_elem(f, other),
			}
		}
	}

	fn write_elem(f: &mut fmt::Formatter<'_>, obj: &LispParseTree) -> fmt::Result {
		match obj {
			LispParseTree::Atom(s) => write!(f, "{s}"),
			LispParseTree::Integer(n) => write!(f, "{n}"),
			LispParseTree::Float(n) => write!(f, "{n}"),
			LispParseTree::Pair(car, cdr) => write_pair(f, car, cdr),
			LispParseTree::Lambda {
				params,
				ret_ty,
				body,
			} => {
				write!(f, "(λ [")?;
				for (i, (name, ty)) in params.iter().enumerate() {
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

	fn write_pair(
		f: &mut fmt::Formatter<'_>,
		car: &LispParseTree,
		cdr: &LispParseTree,
	) -> fmt::Result {
		write!(f, "(")?;
		write_elem(f, car)?;
		write_cdr(f, cdr)?;
		write!(f, ")")
	}

	fn write_cdr(f: &mut fmt::Formatter<'_>, cdr: &LispParseTree) -> fmt::Result {
		match cdr {
			LispParseTree::Atom("nil") => Ok(()),
			LispParseTree::Pair(car, cdr) => {
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

	impl<T: Into<LispParseTree>> From<VecDeque<T>> for LispParseTree {
		fn from(v: VecDeque<T>) -> Self {
			let nil = LispParseTree::Atom("nil".into());
			v.into_iter().map(Into::into).rfold(nil, |acc, obj| {
				LispParseTree::Pair(Box::new(obj), Box::new(acc))
			})
		}
	}

	impl Iterator for LispParseTree {
		type Item = LispParseTree;

		fn next(&mut self) -> Option<Self::Item> {
			match self {
				LispParseTree::Atom("nil") => None,
				LispParseTree::Pair(this, next) => {
					let this = std::mem::replace(this.as_mut(), LispParseTree::Integer(0));
					let next = std::mem::replace(next.as_mut(), LispParseTree::Integer(0));
					*self = next;
					Some(this)
				}
				_ => {
					let ret = std::mem::replace(self, LispParseTree::Integer(0));
					*self = LispParseTree::Atom("nil".into());
					Some(ret)
				}
			}
		}
	}

	impl LispParseTree {
		pub fn peek(&self) -> Option<&LispParseTree> {
			match self {
				LispParseTree::Atom(s) if s == "nil" => None,
				LispParseTree::Pair(car, _) => Some(car),
				other => Some(other),
			}
		}

		pub fn type_of(&self) -> Option<LispType> {
			let res = match self {
				LispParseTree::Atom(_) => "atom".into(),
				LispParseTree::Integer(_) => "i32".into(),
				LispParseTree::Float(_) => "f64".into(),
				LispParseTree::Pair(..) => "pair".into(),
				LispParseTree::Lambda { .. } => "function".into(),
			};
			Some(res)
		}
	}

	#[derive(Debug, PartialEq, Clone, Hash, derive_more::From, derive_more::Display)]
	pub enum LispType {
		#[display("{_0}")]
		Named(SmallString),
	}

	impl From<String> for LispType {
		fn from(s: String) -> Self {
			LispType::Named(s.into())
		}
	}

	impl From<&str> for LispType {
		fn from(s: &str) -> Self {
			LispType::Named(s.into())
		}
	}
}

mod runtime_object {
	use std::collections::HashMap;

	use super::{LispType, SmallString};

	#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
	pub struct ObjectReference(usize);

	pub enum LispObject {
		Atom(SmallString),
		Integer(i32),
		Float(f64),
		Pair(ObjectReference, ObjectReference),
		Lambda {
			params: Vec<(SmallString, Option<LispType>)>,
			ret_ty: Option<LispType>,
			body: ObjectReference,
		},
		Builtins {
			f: Box<dyn Fn(LispObject, LispObject) -> LispObject>,
		},
	}

	pub struct Env {
		objects: HashMap<ObjectReference, LispObject>,
		monotonic_object_count: usize,
	}

	impl Default for Env {
		fn default() -> Self {
			Self::new()
		}
	}

	impl Env {
		pub fn new() -> Self {
			Self {
				objects: HashMap::new(),
				monotonic_object_count: 0,
			}
		}

		pub fn create_object(&mut self) -> ObjectReference {
			let ret = ObjectReference(self.monotonic_object_count);
			self.monotonic_object_count = self
				.monotonic_object_count
				.checked_add(1)
				.expect("Object reference count overflow!");
			self.objects.insert(ret, LispObject::Atom("nil".into()));
			ret
		}

		pub fn get(&self, reference: ObjectReference) -> &LispObject {
			&self.objects[&reference]
		}
	}
}
