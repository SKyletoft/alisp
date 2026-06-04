pub use parse_tree::{LispParseTree, LispType};
pub use runtime_object::{Env, LispObject, LispObjectIterator, ObjectReference};

pub type SmallString = smallstr::SmallString<[u8; 23]>;

pub(crate) mod parse_tree {
	use std::{collections::VecDeque, fmt};

	use smallvec::SmallVec;

	use super::SmallString;

	#[derive(Debug, PartialEq, Clone, variantly::Variantly)]
	pub enum LispParseTree {
		Atom(SmallString),
		Integer(i32 /* TODO: Bigints */),
		Float(f64),
		Pair(Box<LispParseTree>, Box<LispParseTree>),
		Array(Box<[LispParseTree]>),
		// Map(Box<[(SmallString, LispParseTree)]>),
		String(String),
		Lambda {
			params: SmallVec<[(SmallString, Option<LispType>); 1]>,
			ret_ty: Option<LispType>,
			body: Vec<LispParseTree>,
		},
		Quote(Box<LispParseTree>),
		Quasiquote(Box<LispParseTree>),
		Unquote(Box<LispParseTree>),
		Macro {
			params: SmallVec<[SmallString; 1]>,
			body: Vec<LispParseTree>,
		},
	}

	#[cfg(test)]
	pub(crate) fn atom(s: &str) -> LispParseTree {
		LispParseTree::Atom(s.into())
	}

	#[cfg(test)]
	pub(crate) fn quote(l: LispParseTree) -> LispParseTree {
		LispParseTree::Quote(Box::new(l))
	}

	#[cfg(test)]
	pub(crate) fn quasiquote(l: LispParseTree) -> LispParseTree {
		LispParseTree::Quasiquote(Box::new(l))
	}

	#[cfg(test)]
	pub(crate) fn unquote(l: LispParseTree) -> LispParseTree {
		LispParseTree::Unquote(Box::new(l))
	}

	#[cfg(test)]
	pub(crate) fn list<const N: usize>(ls: [LispParseTree; N]) -> LispParseTree {
		ls.into_iter().collect::<Vec<_>>().into()
	}

	#[cfg(test)]
	pub(crate) fn array<const N: usize>(ls: [LispParseTree; N]) -> LispParseTree {
		LispParseTree::Array(Box::new(ls))
	}

	#[cfg(test)]
	#[allow(non_upper_case_globals)]
	pub(crate) const int: fn(i32) -> LispParseTree = LispParseTree::Integer;

	#[cfg(test)]
	#[allow(non_upper_case_globals)]
	pub(crate) const float: fn(f64) -> LispParseTree = LispParseTree::Float;

	impl From<bool> for LispParseTree {
		fn from(value: bool) -> Self {
			match value {
				true => "t".into(),
				false => "nil".into(),
			}
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

	impl<T: Into<LispParseTree>> From<VecDeque<T>> for LispParseTree {
		fn from(v: VecDeque<T>) -> Self {
			let nil = LispParseTree::Atom("nil".into());
			v.into_iter().map(Into::into).rfold(nil, |acc, obj| {
				LispParseTree::Pair(Box::new(obj), Box::new(acc))
			})
		}
	}

	impl fmt::Display for LispParseTree {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			fn write_elem(f: &mut fmt::Formatter<'_>, obj: &LispParseTree) -> fmt::Result {
				match obj {
					LispParseTree::Atom(s) => write!(f, "{s}"),
					LispParseTree::Integer(n) => write!(f, "{n}"),
					LispParseTree::Float(n) => write!(f, "{n}"),
					LispParseTree::Pair(car, cdr) => write_pair(f, car, cdr),
					LispParseTree::String(s) => write!(f, "{s:?}"),
					LispParseTree::Quote(inner) => write!(f, "'{inner}"),
					LispParseTree::Quasiquote(_) => todo!(),
					LispParseTree::Unquote(_) => todo!(),
					LispParseTree::Array(arr) => write_array(f, arr, write_elem),
					LispParseTree::Lambda {
						params,
						ret_ty,
						body,
					} => {
						write!(f, "(λ ")?;
						write_array(f, params, |f, (name, ty)| match ty {
							Some(ty) => write!(f, "({name} {ty})"),
							None => write!(f, "{name}"),
						})?;
						if let Some(ret_ty) = ret_ty {
							write!(f, " -> {ret_ty}")?;
						}
						write!(f, " ")?;
						for (i, expr) in body.iter().enumerate() {
							if i > 0 {
								write!(f, " ")?;
							}
							write_elem(f, expr)?;
						}
						write!(f, ")")
					}
					LispParseTree::Macro { .. } => todo!(),
				}
			}

			fn write_array<T>(
				f: &mut fmt::Formatter<'_>,
				arr: &[T],
				mut write_elem: impl FnMut(&mut fmt::Formatter<'_>, &T) -> fmt::Result,
			) -> fmt::Result {
				write!(f, "[")?;
				for (i, elem) in arr.iter().enumerate() {
					if i > 0 {
						write!(f, " ")?;
					}
					write_elem(f, elem)?;
				}
				write!(f, "]")
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
			write_elem(f, self)
		}
	}

	impl LispParseTree {
		pub(crate) fn next(&mut self) -> Option<LispParseTree> {
			match self {
				LispParseTree::Atom(s) if s == "nil" => None,
				LispParseTree::Pair(this, next) => {
					let this = std::mem::replace(this.as_mut(), LispParseTree::Integer(0));
					let next = std::mem::replace(next.as_mut(), LispParseTree::Integer(0));
					*self = next;
					Some(this)
				}
				LispParseTree::Array(arr) => {
					// [] is considered a ZST despite being dynamically sized as manually checked on godbolt in Rust 1.95
					let old = std::mem::replace(arr, Box::new([]));
					let mut iter = old.into_vec().into_iter();
					let this = iter.next()?;
					*self = LispParseTree::Array(iter.collect());
					Some(this)
				}
				_ => {
					let ret = std::mem::replace(self, LispParseTree::Atom("nil".into()));
					Some(ret)
				}
			}
		}

		#[allow(dead_code)]
		pub(crate) fn peek(&self) -> Option<&LispParseTree> {
			match self {
				LispParseTree::Atom(s) if s == "nil" => None,
				LispParseTree::Pair(car, _) => Some(car),
				LispParseTree::Array(arr) => arr.first(),
				other => Some(other),
			}
		}

		#[allow(dead_code)]
		pub(crate) fn type_of(&self) -> Option<LispType> {
			let res = match self {
				LispParseTree::Atom(_) => LispType::Atom,
				LispParseTree::Integer(_) => LispType::Integer,
				LispParseTree::Float(_) => LispType::Float,
				LispParseTree::Pair(..) => LispType::Pair,
				LispParseTree::Lambda { .. } => LispType::Function,
				LispParseTree::String(_) => LispType::String,
				LispParseTree::Array(..) => LispType::Array,
				LispParseTree::Quote(_)
				| LispParseTree::Quasiquote(_)
				| LispParseTree::Unquote(_) => todo!(),
				LispParseTree::Macro { .. } => LispType::Macro,
			};
			Some(res)
		}
	}

	impl IntoIterator for LispParseTree {
		type IntoIter = LispParseTreeIterator;
		type Item = LispParseTree;

		fn into_iter(self) -> Self::IntoIter {
			LispParseTreeIterator(self)
		}
	}

	pub struct LispParseTreeIterator(LispParseTree);

	impl Iterator for LispParseTreeIterator {
		type Item = LispParseTree;

		fn next(&mut self) -> Option<Self::Item> {
			self.0.next()
		}
	}

	#[derive(Debug, PartialEq, Clone, Hash, derive_more::From, derive_more::Display)]
	pub enum LispType {
		#[display("{_0}")]
		Named(SmallString),
		#[display("atom")]
		Atom,
		#[display("i32")]
		Integer,
		#[display("f64")]
		Float,
		#[display("list")]
		Pair,
		#[display("array")]
		Array,
		#[display("function")]
		Function,
		#[display("string")]
		String,
		#[display("code")]
		Code,
		#[display("macro")]
		Macro,
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
	use std::{
		collections::HashMap,
		fmt,
		marker::PhantomData,
		rc::Rc,
		sync::atomic::{AtomicBool, Ordering},
	};

	use smallvec::SmallVec;

	use super::{LispParseTree, LispType, SmallString};
	use crate::eval::RuntimeError;

	#[derive(Clone, Copy, PartialEq, Eq, Hash)]
	pub struct ObjectReference<'a, const N: usize = 0>(usize, PhantomData<&'a ()>);

	impl<'a, const N: usize> fmt::Debug for ObjectReference<'a, N> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "ObjectReference({})", self.0)
		}
	}

	impl<'a, const N: usize> ObjectReference<'a, N> {
		pub fn iter<'b>(self, env: &'b Env<'a, N>) -> LispObjectIterator<'a, 'b, N> {
			LispObjectIterator {
				env,
				reference: self,
			}
		}

		#[inline(always)]
		pub fn get<'b>(&'b self, env: &'b Env<'a, N>) -> &'b LispObject<'a, N> {
			env.get(*self)
		}

		#[inline(always)]
		pub fn peek<'b>(&'b self, env: &'b Env<'a, N>) -> Option<&'b LispObject<'a, N>> {
			self.get(env).next(env)
		}

		#[inline(always)]
		pub fn next(&mut self, env: &Env<'a, N>) -> Option<ObjectReference<'a, N>> {
			match &self.get(env) {
				LispObject::Pair(this, next) => {
					let ret = *this;
					*self = *next;
					Some(ret)
				}
				_ => None,
			}
		}

		#[inline(always)]
		pub fn from_parse_object(
			parse_object: LispParseTree,
			env: &mut Env<'a, N>,
		) -> ObjectReference<'a, N> {
			match parse_object {
				LispParseTree::Atom(s) => env.create_object(LispObject::Atom(s)),
				LispParseTree::Integer(i) => env.create_object(LispObject::Integer(i)),
				LispParseTree::Float(f) => env.create_object(LispObject::Float(f)),
				LispParseTree::String(s) => env.create_object(LispObject::String(s)),
				LispParseTree::Array(arr) => {
					let arr: Vec<ObjectReference<'a, N>> = arr
						.into_vec()
						.into_iter()
						.map(|e| Self::from_parse_object(e, env))
						.collect();
					env.create_object(LispObject::Array(arr.into_boxed_slice()))
				}
				LispParseTree::Pair(car, cdr) => {
					let car = Self::from_parse_object(*car, env);
					let cdr = Self::from_parse_object(*cdr, env);
					env.create_object(LispObject::Pair(car, cdr))
				}
				LispParseTree::Lambda {
					params,
					ret_ty,
					body,
				} => {
					let body: Vec<ObjectReference<'a, N>> = body
						.into_iter()
						.map(|e| Self::from_parse_object(e, env))
						.collect();
					env.create_object(LispObject::Lambda {
						params,
						ret_ty,
						body,
					})
				}
				LispParseTree::Quote(inner) => {
					let inner = Self::from_parse_object(*inner, env);
					env.create_object(LispObject::Quote(inner))
				}
				LispParseTree::Unquote(inner) => {
					let inner = Self::from_parse_object(*inner, env);
					env.create_object(LispObject::Unquote(inner))
				}
				LispParseTree::Quasiquote(inner) => {
					let inner = Self::from_parse_object(*inner, env);
					env.create_object(LispObject::Quasiquote(inner))
				}
				LispParseTree::Macro { params, body } => {
					let body = body
						.into_iter()
						.map(|l| Self::from_parse_object(l, env))
						.collect();
					env.create_object(LispObject::Macro { params, body })
				}
			}
		}
	}

	type BuiltinDyadicFn<'a, const N: usize> = Rc<
		dyn Fn(
			LispObject<'a, N>,
			LispObject<'a, N>,
			&mut Env<'a, N>,
		) -> Result<ObjectReference<'a, N>, RuntimeError>,
	>;

	type BuiltinMonadicFn<'a, const N: usize> = Rc<
		dyn Fn(LispObject<'a, N>, &mut Env<'a, N>) -> Result<ObjectReference<'a, N>, RuntimeError>,
	>;

	#[derive(Clone)]
	pub enum LispObject<'a, const N: usize = 0> {
		Atom(SmallString),
		Integer(i32),
		Float(f64),
		String(String),
		Pair(ObjectReference<'a, N>, ObjectReference<'a, N>),
		Array(Box<[ObjectReference<'a, N>]>),
		Lambda {
			params: SmallVec<[(SmallString, Option<LispType>); 1]>,
			ret_ty: Option<LispType>,
			body: Vec<ObjectReference<'a, N>>,
		},
		Macro {
			params: SmallVec<[SmallString; 1]>,
			body: Vec<ObjectReference<'a, N>>,
		},
		BuiltinDyadic(BuiltinDyadicFn<'a, N>),
		BuiltinMonadic(BuiltinMonadicFn<'a, N>),
		Quote(ObjectReference<'a, N>),
		Quasiquote(ObjectReference<'a, N>),
		Unquote(ObjectReference<'a, N>),
	}

	impl<'a, const N: usize> LispObject<'a, N> {
		pub(crate) fn next<'b>(&'b self, env: &'b Env<'a, N>) -> Option<&'b LispObject<'a, N>> {
			match self {
				LispObject::Pair(_, next) => Some(env.get(*next)),
				_ => None,
			}
		}

		pub(crate) fn type_of(&self) -> LispType {
			match self {
				LispObject::Atom(_) => LispType::Atom,
				LispObject::Integer(_) => LispType::Integer,
				LispObject::Float(_) => LispType::Float,
				LispObject::String(_) => LispType::String,
				LispObject::Pair(..) => LispType::Pair,
				LispObject::Array(..) => LispType::Array,
				LispObject::Lambda { .. }
				| LispObject::BuiltinDyadic(_)
				| LispObject::BuiltinMonadic(_) => LispType::Function,
				LispObject::Macro { .. } => LispType::Macro,
				LispObject::Quote(_) | LispObject::Quasiquote(_) | LispObject::Unquote(_) => {
					LispType::Code
				}
			}
		}

		pub fn display(&self, f: &mut fmt::Formatter<'_>, env: &Env<'a, N>) -> fmt::Result {
			write_lisp_elem(f, self, env)
		}
	}

	fn write_lisp_elem<'a, const N: usize>(
		f: &mut fmt::Formatter<'_>,
		obj: &LispObject<'a, N>,
		env: &Env<'a, N>,
	) -> fmt::Result {
		match obj {
			LispObject::Atom(s) => write!(f, "{s}"),
			LispObject::Integer(n) => write!(f, "{n}"),
			LispObject::Float(n) => write!(f, "{n}"),
			LispObject::String(s) => write!(f, "{s:?}"),
			LispObject::Pair(car, cdr) => write_lisp_pair(f, *car, *cdr, env),
			LispObject::Array(arr) => {
				write_array(f, arr, |f, elem| write_lisp_elem(f, env.get(*elem), env))
			}
			LispObject::Lambda {
				params,
				ret_ty,
				body,
			} => {
				write!(f, "(λ ")?;
				write_array(f, params, |f, (name, ty)| match ty {
					Some(ty) => write!(f, "({name} {ty})"),
					None => write!(f, "{name}"),
				})?;
				if let Some(ret_ty) = ret_ty {
					write!(f, " -> {ret_ty}")?;
				}
				write!(f, " ")?;
				for expr in body.iter() {
					write_lisp_elem(f, env.get(*expr), env)?;
				}
				write!(f, ")")
			}
			LispObject::BuiltinDyadic(_) | LispObject::BuiltinMonadic(_) => {
				write!(f, "Builtin")
			}
			LispObject::Quote(inner) => {
				write!(f, "'")?;
				write_lisp_elem(f, env.get(*inner), env)
			}
			LispObject::Quasiquote(inner) => {
				write!(f, "`")?;
				write_lisp_elem(f, env.get(*inner), env)
			}
			LispObject::Unquote(inner) => {
				write!(f, ",")?;
				write_lisp_elem(f, env.get(*inner), env)
			}
			LispObject::Macro { .. } => todo!(),
		}
	}

	fn write_array<T>(
		f: &mut fmt::Formatter<'_>,
		arr: &[T],
		mut write_elem: impl FnMut(&mut fmt::Formatter<'_>, &T) -> fmt::Result,
	) -> fmt::Result {
		write!(f, "[")?;
		for (i, elem) in arr.iter().enumerate() {
			if i > 0 {
				write!(f, " ")?;
			}
			write_elem(f, elem)?;
		}
		write!(f, "]")
	}

	fn write_lisp_pair<'a, const N: usize>(
		f: &mut fmt::Formatter<'_>,
		car: ObjectReference<'a, N>,
		cdr: ObjectReference<'a, N>,
		env: &Env<'a, N>,
	) -> fmt::Result {
		write!(f, "(")?;
		write_lisp_elem(f, env.get(car), env)?;
		write_lisp_cdr(f, cdr, env)?;
		write!(f, ")")
	}

	fn write_lisp_cdr<'a, const N: usize>(
		f: &mut fmt::Formatter<'_>,
		cdr: ObjectReference<'a, N>,
		env: &Env<'a, N>,
	) -> fmt::Result {
		match env.get(cdr) {
			LispObject::Atom(s) if s == "nil" => Ok(()),
			LispObject::Pair(car, next) => {
				write!(f, " ")?;
				write_lisp_elem(f, env.get(*car), env)?;
				write_lisp_cdr(f, *next, env)
			}
			other => {
				write!(f, " . ")?;
				write_lisp_elem(f, other, env)
			}
		}
	}

	impl<'a, const N: usize> fmt::Debug for LispObject<'a, N> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			#[derive(Debug)]
			// Dead code analysis ignores Debug impls and this struct is exclusively here for its Debug impl
			#[allow(dead_code)]
			pub enum LispObjectDebug<'a, 'b, const N: usize = 0> {
				Atom(&'b SmallString),
				Integer(&'b i32),
				Float(&'b f64),
				String(&'b String),
				Pair(&'b ObjectReference<'a, N>, &'b ObjectReference<'a, N>),
				Array(&'b [ObjectReference<'a, N>]),
				Lambda {
					params: &'b SmallVec<[(SmallString, Option<LispType>); 1]>,
					ret_ty: &'b Option<LispType>,
					body: &'b Vec<ObjectReference<'a, N>>,
				},
				Macro {
					params: &'b SmallVec<[SmallString; 1]>,
					body: &'b Vec<ObjectReference<'a, N>>,
				},
				Quote(&'b ObjectReference<'a, N>),
				Quasiquote(&'b ObjectReference<'a, N>),
				Unquote(&'b ObjectReference<'a, N>),
			}

			match self {
				LispObject::BuiltinDyadic { .. } | LispObject::BuiltinMonadic { .. } => {
					write!(f, "Builtin")
				}
				LispObject::Atom(s) => write!(f, "{:?}", LispObjectDebug::<N>::Atom(s)),
				LispObject::Integer(v) => write!(f, "{:?}", LispObjectDebug::<N>::Integer(v)),
				LispObject::Float(v) => write!(f, "{:?}", LispObjectDebug::<N>::Float(v)),
				LispObject::String(v) => write!(f, "{:?}", LispObjectDebug::<N>::String(v)),
				LispObject::Pair(a, b) => write!(f, "{:?}", LispObjectDebug::<N>::Pair(a, b)),
				LispObject::Array(v) => write!(f, "{:?}", LispObjectDebug::<N>::Array(v)),
				LispObject::Lambda {
					params,
					ret_ty,
					body,
				} => write!(
					f,
					"{:?}",
					LispObjectDebug::<N>::Lambda {
						params,
						ret_ty,
						body
					}
				),
				LispObject::Macro { params, body } => {
					write!(f, "{:?}", LispObjectDebug::<N>::Macro { params, body })
				}
				LispObject::Quote(v) => write!(f, "{:?}", LispObjectDebug::<N>::Quote(v)),
				LispObject::Quasiquote(v) => write!(f, "{:?}", LispObjectDebug::<N>::Quasiquote(v)),
				LispObject::Unquote(v) => write!(f, "{:?}", LispObjectDebug::<N>::Unquote(v)),
			}
		}
	}

	pub struct LispObjectIterator<'a, 'b, const N: usize> {
		env: &'b Env<'a, N>,
		reference: ObjectReference<'a, N>,
	}

	impl<'a, 'b, const N: usize> Iterator for LispObjectIterator<'a, 'b, N> {
		type Item = ObjectReference<'a, N>;

		fn next(&mut self) -> Option<Self::Item> {
			let obj = self.env.get(self.reference);
			match obj {
				LispObject::Pair(this, next) => {
					self.reference = *next;
					Some(*this)
				}
				_ => None,
			}
		}
	}

	pub struct Env<'a, const N: usize = 0> {
		objects: HashMap<ObjectReference<'a, N>, LispObject<'a, N>>,
		monotonic_object_count: usize,

		pub(crate) stack: Vec<Vec<(SmallString, ObjectReference<'a, N>)>>,
	}

	impl<'a, const N: usize> fmt::Debug for Env<'a, N> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Env")
				.field("monotonic_object_count", &self.monotonic_object_count)
				.field("stack", &self.stack)
				.finish()?;
			writeln!(f)?;
			writeln!(f, "objects:")?;
			for (k, v) in &self.objects {
				write!(f, "  {k:?}: ")?;
				v.display(f, self)?;
				writeln!(f)?;
			}
			Ok(())
		}
	}
	static ENV_IN_USE: [AtomicBool; 64] = unsafe { std::mem::transmute([false; 64]) };

	impl<'a, const N: usize> Env<'a, N> {
		#[allow(clippy::result_unit_err)]
		pub fn new() -> Result<Self, ()> {
			if ENV_IN_USE[N].swap(true, Ordering::SeqCst) {
				Err(())
			} else {
				let mut ret = Self {
					objects: HashMap::new(),
					monotonic_object_count: 0,
					stack: vec![Vec::new()],
				};
				// let t_ref = ret.create_object(LispObject::Atom("t".into()));
				// ret.stack.push(("t".into(), t_ref));
				// let nil_ref = ret.create_object(LispObject::Atom("nil".into()));
				// ret.stack.push(("nil".into(), nil_ref));
				ret.push_builtin_dyadic("set", |id, val, env| {
					if let LispObject::Quote(obj_ref) = id
						&& let LispObject::Atom(ident) = obj_ref.get(env)
					{
						let ident = ident.clone();
						*env.get_stack_var_mut(&ident) = val;
						Ok(obj_ref)
					} else {
						Err(RuntimeError::AssignmentToNonVariable)
					}
				});
				ret.push_builtin_dyadic("+", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(LispObject::Float(x + y)))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) => {
						Ok(env.create_object(LispObject::Integer(x + y)))
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic("*", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(LispObject::Float(x * y)))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) => {
						Ok(env.create_object(LispObject::Integer(x * y)))
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic("-", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(LispObject::Float(x - y)))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) => {
						Ok(env.create_object(LispObject::Integer(x - y)))
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic("/", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(LispObject::Float(x / y)))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) if y != 0 => {
						Ok(env.create_object(LispObject::Integer(x / y)))
					}
					(LispObject::Integer(_), LispObject::Integer(_)) => {
						Err(RuntimeError::DivisionByZero)
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic("%", |l, r, env| match (l, r) {
					(LispObject::Integer(x), LispObject::Integer(y)) if y != 0 => {
						Ok(env.create_object(LispObject::Integer(x % y)))
					}
					(LispObject::Integer(_), LispObject::Integer(_)) => {
						Err(RuntimeError::DivisionByZero)
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic("=", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(if (x - y).abs() < f64::EPSILON {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) => {
						Ok(env.create_object(if x == y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic("<", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(if x < y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) => {
						Ok(env.create_object(if x < y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic(">", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(if x > y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) => {
						Ok(env.create_object(if x > y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic("<=", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(if x <= y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) => {
						Ok(env.create_object(if x <= y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_dyadic(">=", |l, r, env| match (l, r) {
					(LispObject::Float(x), LispObject::Float(y)) => {
						Ok(env.create_object(if x >= y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(LispObject::Integer(x), LispObject::Integer(y)) => {
						Ok(env.create_object(if x >= y {
							LispObject::Atom("t".into())
						} else {
							LispObject::Atom("nil".into())
						}))
					}
					(l, r) => Err(RuntimeError::TypeError {
						expected: Some(l.type_of()),
						actual: Some(r.type_of()),
					}),
				});
				ret.push_builtin_monadic("print", |arg, env| {
					print!("{}", lisp_to_string(&arg, env));
					Ok(env.create_object(LispObject::Atom("nil".into())))
				});
				ret.push_builtin_monadic("println", |arg, env| {
					println!("{}", lisp_to_string(&arg, env));
					Ok(env.create_object(LispObject::Atom("nil".into())))
				});
				Ok(ret)
			}
		}

		pub fn wait_for_new() -> Self {
			loop {
				if let Ok(e) = Env::new() {
					return e;
				}
				std::thread::yield_now();
			}
		}

		fn push_builtin_dyadic(
			&mut self,
			name: &str,
			f: impl Fn(
				LispObject<'a, N>,
				LispObject<'a, N>,
				&mut Env<'a, N>,
			) -> Result<ObjectReference<'a, N>, RuntimeError>
			+ 'static,
		) {
			let fn_ref = self.create_object(LispObject::BuiltinDyadic(Rc::new(f)));
			self.stack.last_mut().unwrap().push((name.into(), fn_ref));
		}

		fn push_builtin_monadic(
			&mut self,
			name: &str,
			f: impl Fn(
				LispObject<'a, N>,
				&mut Env<'a, N>,
			) -> Result<ObjectReference<'a, N>, RuntimeError>
			+ 'static,
		) {
			let fn_ref = self.create_object(LispObject::BuiltinMonadic(Rc::new(f)));
			self.stack.last_mut().unwrap().push((name.into(), fn_ref));
		}

		#[inline(always)]
		pub fn create_object(&mut self, obj: LispObject<'a, N>) -> ObjectReference<'a, N> {
			let ret = {
				let id = self.monotonic_object_count;
				ObjectReference(id, PhantomData)
			};
			self.monotonic_object_count = self
				.monotonic_object_count
				.checked_add(1)
				.expect("Object reference count overflow!");
			self.objects.insert(ret, obj);
			ret
		}

		#[inline(always)]
		pub fn get<'b>(&'b self, reference: ObjectReference<'a, N>) -> &'b LispObject<'a, N> {
			&self.objects[&reference]
		}

		#[inline(always)]
		pub fn get_mut(&mut self, reference: ObjectReference<'a, N>) -> &mut LispObject<'a, N> {
			self.objects
				.get_mut(&reference)
				.expect("References from the same Env must be valid")
		}

		#[inline(always)]
		pub fn nil(&mut self) -> ObjectReference<'a, N> {
			self.create_object(LispObject::Atom("nil".into()))
		}

		pub fn get_stack_var(&self, id: &str) -> Result<ObjectReference<'a, N>, RuntimeError> {
			self.stack
				.iter()
				.flat_map(|frame| frame.iter())
				.rev()
				.find(|(s, _)| id == s.as_str())
				.map(|(_, val)| *val)
				.ok_or(RuntimeError::UndefinedVariable(id.into()))
		}

		pub fn get_stack_var_mut(&mut self, id: &str) -> &mut LispObject<'a, N> {
			let obj_ref = self
				.stack
				.iter()
				.flat_map(|frame| frame.iter())
				.rev()
				.find(|(s, _)| id == s.as_str())
				.map(|(_, val)| *val);
			let r = match obj_ref {
				Some(r) => r,
				None => {
					let r = self.nil();
					let stack = match self.stack.last_mut() {
						Some(s) => s,
						None => self.stack.push_mut(Vec::new()),
					};
					stack.push((id.into(), r));
					r
				}
			};
			self.get_mut(r)
		}
	}

	impl<'a, const N: usize> Drop for Env<'a, N> {
		fn drop(&mut self) {
			ENV_IN_USE[N].store(false, Ordering::SeqCst);
		}
	}

	struct LispDisplay<'a, 'b, const N: usize>(&'b LispObject<'a, N>, &'b Env<'a, N>);
	impl<'a, 'b, const N: usize> fmt::Display for LispDisplay<'a, 'b, N> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write_lisp_elem(f, self.0, self.1)
		}
	}

	fn lisp_to_string<'a, const N: usize>(obj: &LispObject<'a, N>, env: &Env<'a, N>) -> String {
		format!("{}", LispDisplay(obj, env))
	}
}

pub fn lisp_object_to_parse_tree<'a>(obj: &LispObject<'a>, env: &Env<'a>) -> LispParseTree {
	match obj {
		LispObject::Atom(s) => LispParseTree::Atom(s.clone()),
		LispObject::Integer(i) => LispParseTree::Integer(*i),
		LispObject::Float(f) => LispParseTree::Float(*f),
		LispObject::String(s) => LispParseTree::String(s.clone()),
		LispObject::Pair(car, cdr) => {
			let car = lisp_object_to_parse_tree(env.get(*car), env);
			let cdr = lisp_object_to_parse_tree(env.get(*cdr), env);
			LispParseTree::Pair(Box::new(car), Box::new(cdr))
		}
		LispObject::Lambda {
			params,
			ret_ty,
			body,
		} => {
			let body: Vec<LispParseTree> = body
				.iter()
				.map(|e| lisp_object_to_parse_tree(env.get(*e), env))
				.collect();
			LispParseTree::Lambda {
				params: params.clone(),
				ret_ty: ret_ty.clone(),
				body,
			}
		}
		LispObject::BuiltinDyadic { .. } | LispObject::BuiltinMonadic { .. } => {
			LispParseTree::Atom("builtin".into())
		}
		LispObject::Quote(inner) => {
			LispParseTree::Quote(Box::new(lisp_object_to_parse_tree(env.get(*inner), env)))
		}
		LispObject::Quasiquote(inner) => {
			LispParseTree::Quasiquote(Box::new(lisp_object_to_parse_tree(env.get(*inner), env)))
		}
		LispObject::Unquote(inner) => {
			LispParseTree::Unquote(Box::new(lisp_object_to_parse_tree(env.get(*inner), env)))
		}
		LispObject::Array(arr) => {
			let arr: Vec<LispParseTree> = arr
				.iter()
				.map(|e| lisp_object_to_parse_tree(env.get(*e), env))
				.collect();
			LispParseTree::Array(arr.into_boxed_slice())
		}
		LispObject::Macro { .. } => todo!(),
	}
}

#[cfg(test)]
mod iter_test {
	use quickcheck_macros::quickcheck;

	use super::{
		LispParseTree,
		parse_tree::{array, int, list},
	};

	#[test]
	fn list_iter_empty() {
		let list: LispParseTree = Vec::<LispParseTree>::new().into();
		let items: Vec<LispParseTree> = list.into_iter().collect();
		assert!(items.is_empty());
	}

	#[test]
	fn list_iter_one() {
		let list: LispParseTree = list([int(42)]);
		let items: Vec<LispParseTree> = list.into_iter().collect();
		assert_eq!(items, vec![int(42)]);
	}

	#[test]
	fn list_iter_multi() {
		let list: LispParseTree = list([int(1), int(2), int(3)]);
		let items: Vec<LispParseTree> = list.into_iter().collect();
		assert_eq!(items, (1..=3).map(int).collect::<Vec<_>>());
	}

	#[test]
	fn array_iter_empty() {
		let arr = array([]);
		let items: Vec<LispParseTree> = arr.into_iter().collect();
		assert!(items.is_empty());
	}

	#[test]
	fn array_iter_one() {
		let arr = array([int(42)]);
		let items: Vec<LispParseTree> = arr.into_iter().collect();
		assert_eq!(items, vec![int(42)]);
	}

	#[test]
	fn array_iter_multi() {
		let arr = array([int(1), int(2), int(3)]);
		let items: Vec<LispParseTree> = arr.into_iter().collect();
		assert_eq!(items, (1..=3).map(int).collect::<Vec<_>>());
	}

	#[quickcheck]
	fn list_iter_equals_vec(v: Vec<i32>) {
		let list: LispParseTree = v.iter().map(|&n| int(n)).collect::<Vec<_>>().into();
		let items: Vec<LispParseTree> = list.into_iter().collect();
		let expected: Vec<LispParseTree> = v.into_iter().map(int).collect();
		assert_eq!(items, expected);
	}

	#[quickcheck]
	fn array_iter_equals_vec(v: Vec<i32>) {
		let arr = LispParseTree::Array(v.iter().map(|&n| int(n)).collect());
		let items: Vec<LispParseTree> = arr.into_iter().collect();
		let expected: Vec<LispParseTree> = v.into_iter().map(int).collect();
		assert_eq!(items, expected);
	}

	#[quickcheck]
	fn list_and_array_iter_match(v: Vec<i32>) {
		let list: LispParseTree = v.iter().map(|&n| int(n)).collect::<Vec<_>>().into();
		let arr = LispParseTree::Array(v.iter().map(|&n| int(n)).collect());
		let list_items: Vec<LispParseTree> = list.into_iter().collect();
		let arr_items: Vec<LispParseTree> = arr.into_iter().collect();
		assert_eq!(list_items, arr_items);
	}
}
