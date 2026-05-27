pub use parse_tree::{LispParseTree, LispType};
pub use runtime_object::{Env, LispObject, LispObjectIterator, ObjectReference};

pub type SmallString = smallstr::SmallString<[u8; 23]>;

mod parse_tree {
	use std::{collections::VecDeque, fmt};

	use smallvec::SmallVec;

	use super::SmallString;

	#[derive(Debug, PartialEq, Clone, derive_more::From)]
	pub enum LispParseTree {
		Atom(SmallString),
		Integer(i32 /* TODO: Bigints */),
		Float(f64),
		Pair(Box<LispParseTree>, Box<LispParseTree>),
		// Array(Box<[LispParseTree]>),
		// Map(Box<[(SmallString, LispParseTree)]>),
		String(String),
		Lambda {
			params: SmallVec<[(SmallString, Option<LispType>); 1]>,
			ret_ty: Option<LispType>,
			body: Vec<LispParseTree>,
		},
		Quote(Box<LispParseTree>),
	}

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
						for (i, expr) in body.iter().enumerate() {
							if i > 0 {
								write!(f, " ")?;
							}
							write_elem(f, expr)?;
						}
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
				_ => {
					let ret = std::mem::replace(self, LispParseTree::Integer(0));
					*self = LispParseTree::Atom("nil".into());
					Some(ret)
				}
			}
		}

		#[allow(dead_code)]
		pub(crate) fn peek(&self) -> Option<&LispParseTree> {
			match self {
				LispParseTree::Atom(s) if s == "nil" => None,
				LispParseTree::Pair(car, _) => Some(car),
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
				LispParseTree::Quote(_) => LispType::Code,
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
		#[display("function")]
		Function,
		#[display("string")]
		String,
		#[display("code")]
		Code,
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
		#[allow(dead_code)]
		pub(crate) fn iter<'b>(self, env: &'b Env<'a, N>) -> LispObjectIterator<'a, 'b, N> {
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
				LispParseTree::String(_) => todo!(),
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
		Pair(ObjectReference<'a, N>, ObjectReference<'a, N>),
		Lambda {
			params: SmallVec<[(SmallString, Option<LispType>); 1]>,
			ret_ty: Option<LispType>,
			body: Vec<ObjectReference<'a, N>>,
		},
		BuiltinDyadic(BuiltinDyadicFn<'a, N>),
		BuiltinMonadic(BuiltinMonadicFn<'a, N>),
		Quote(ObjectReference<'a, N>),
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
				LispObject::Pair(..) => LispType::Pair,
				LispObject::Lambda { .. }
				| LispObject::BuiltinDyadic(_)
				| LispObject::BuiltinMonadic(_) => LispType::Function,
				LispObject::Quote(_) => LispType::Code,
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
			LispObject::Pair(car, cdr) => write_lisp_pair(f, *car, *cdr, env),
			LispObject::Lambda {
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
		}
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
			match self {
				LispObject::Atom(s) => f.debug_tuple("Atom").field(s).finish(),
				LispObject::Integer(i) => f.debug_tuple("Integer").field(i).finish(),
				LispObject::Float(fl) => f.debug_tuple("Float").field(fl).finish(),
				LispObject::Pair(a, b) => f.debug_tuple("Pair").field(a).field(b).finish(),
				LispObject::Lambda {
					params,
					ret_ty,
					body,
				} => f
					.debug_struct("Lambda")
					.field("params", params)
					.field("ret_ty", ret_ty)
					.field("body", body)
					.finish(),
				LispObject::BuiltinDyadic { .. } | LispObject::BuiltinMonadic { .. } => {
					f.debug_struct("Builtin").finish()
				}
				LispObject::Quote(inner) => f.debug_tuple("Quote").field(inner).finish(),
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

		pub(crate) stack: Vec<(SmallString, ObjectReference<'a, N>)>,
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
		pub fn new() -> Result<Self, ()> {
			if ENV_IN_USE[N].swap(true, Ordering::SeqCst) {
				Err(())
			} else {
				let mut ret = Self {
					objects: HashMap::new(),
					monotonic_object_count: 0,
					stack: Vec::new(),
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
			self.stack.push((name.into(), fn_ref));
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
			self.stack.push((name.into(), fn_ref));
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
				.rev()
				.find(|(s, _)| id == s.as_str())
				.map(|(_, val)| *val)
				.ok_or(RuntimeError::UndefinedVariable)
		}

		pub fn get_stack_var_mut(&mut self, id: &str) -> &mut LispObject<'a, N> {
			let obj_ref = self
				.stack
				.iter()
				.rev()
				.find(|(s, _)| id == s.as_str())
				.map(|(_, val)| *val);
			let r = match obj_ref {
				Some(r) => r,
				None => {
					let r = self.create_object(LispObject::Atom("nil".into()));
					self.stack.push((id.into(), r));
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
	}
}
