use crate::lisp_object::{Env, LispObject, LispType, ObjectReference, SmallString};

const RECURSION_LIMIT: usize = 50;

#[derive(Debug, PartialEq, derive_more::Display)]
pub enum RuntimeError {
	#[display("Undefined variable: {_0:?}")]
	UndefinedVariable(SmallString),
	#[display("Type error: {expected:?} ≠ {actual:?}")]
	TypeError {
		expected: Option<LispType>,
		actual: Option<LispType>,
	},
	#[display("Too many arguments")]
	TooManyArguments,
	#[display("alisp doesn't support curried functions (= not enough arguments in function call)")]
	NoCurrying,
	#[display("Too many arguments in macro")]
	TooManyArgumentsMacro,
	#[display("alisp doesn't support curried macros (= not enough arguments in macro call)")]
	NoCurryingMacro,
	#[display("Division by zero")]
	DivisionByZero,
	#[display("Assignment to non-identifier")]
	AssignmentToNonVariable,
	#[display("Invalid lambda construction: {msg}")]
	BrokenLambda { msg: &'static str },
	#[display("Invalid macro construction: {msg}")]
	BrokenMacro { msg: &'static str },
	#[display("Stack overflow")]
	StackOverflow,
}

impl std::error::Error for RuntimeError {}

fn type_guard(expected: &Option<LispType>, actual: &Option<LispType>) -> Result<(), RuntimeError> {
	match (expected, actual) {
		(Some(x), Some(y)) if x != y => Err(RuntimeError::TypeError {
			expected: expected.clone(),
			actual: actual.clone(),
		}),
		_ => Ok(()),
	}
}

pub fn eval_top<'a>(
	env: &mut Env<'a>,
	expr: ObjectReference<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let res = match env.get(expr) {
		LispObject::Quasiquote(inner) => expand_once(env, *inner)?,
		LispObject::Quote(inner) => *inner,
		_ => eval_inner(env, expr)?,
	};
	Ok(res)
}

pub fn eval_inner<'a>(
	env: &mut Env<'a>,
	expr: ObjectReference<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let res = match env.get(expr) {
		LispObject::Pair(f, x) => {
			let mut args_iter = *x;
			let function = eval_inner(env, *f)?.get(env).clone();
			match function {
				LispObject::Lambda {
					params,
					ret_ty,
					body,
				} => {
					if env.stack.len() > RECURSION_LIMIT {
						return Err(RuntimeError::StackOverflow);
					}
					let stack_frame = {
						let mut evalled = Vec::new();
						while let Some(arg) = args_iter.next(env) {
							evalled.push(eval_top(env, arg)?);
						}
						if evalled.len() < params.pre.len() + params.post.len() {
							return Err(RuntimeError::NoCurrying);
						}
						if evalled.len() != params.pre.len() + params.post.len()
							&& params.rest.is_none()
						{
							return Err(RuntimeError::TooManyArguments);
						}
						let mut out = Vec::new();

						for ((param_name, param_type), evalled_arg) in params
							.pre
							.iter()
							.zip(evalled[..params.pre.len()].iter().copied())
						{
							type_guard(param_type, &Some(evalled_arg.get(env).type_of()))?;
							out.push((param_name.clone(), evalled_arg));
						}

						if let Some((rest_name, ty)) = &params.rest {
							let rest_vals = evalled
								[params.pre.len()..evalled.len() - params.post.len()]
								.to_vec()
								.into_boxed_slice();
							for val in rest_vals.iter() {
								type_guard(ty, &Some(val.get(env).type_of()))?;
							}
							out.push((
								rest_name.clone(),
								env.create_object(LispObject::Array(rest_vals)),
							));
						}

						for ((param_name, param_type), evalled_arg) in params
							.post
							.iter()
							.zip(evalled[evalled.len() - params.post.len()..].iter().copied())
						{
							type_guard(param_type, &Some(evalled_arg.get(env).type_of()))?;
							out.push((param_name.clone(), evalled_arg));
						}

						out
					};
					env.stack.push(stack_frame);

					let result = body
						.into_iter()
						.map(|e| eval_top(env, e))
						.last()
						.unwrap_or(Ok(expr))?;
					type_guard(&ret_ty, &Some(result.get(env).type_of()))?;
					env.stack.pop();
					result
				}
				LispObject::Macro { params, body } => {
					let mut stack_frame = Vec::new();
					for param_name in params.iter() {
						let arg = args_iter.next(env).ok_or(RuntimeError::NoCurryingMacro)?;
						stack_frame.push((param_name.clone(), arg));
					}
					if args_iter.next(env).is_some() {
						return Err(RuntimeError::TooManyArgumentsMacro);
					}

					env.stack.push(stack_frame);
					let expanded = body
						.iter()
						.copied()
						.map(|b| expand_once(env, b))
						.collect::<Result<Vec<_>, _>>()?;
					env.stack.pop();

					expanded
						.into_iter()
						.map(|b| eval_top(env, b))
						.last()
						.unwrap_or(Ok(expr))?
				}
				LispObject::BuiltinDyadic(f) => {
					let l_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let l_evalled = eval_inner(env, l_ref)?;
					let l = env.get(l_evalled).clone();

					let r_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let r_evalled = eval_inner(env, r_ref)?;
					let r = env.get(r_evalled).clone();

					let res = f(env, l, r)?;
					env.create_object(res)
				}
				LispObject::BuiltinMonadic(f) => {
					let arg_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let evalled = eval_inner(env, arg_ref)?;
					let arg = env.get(evalled).clone();
					let res = f(env, arg)?;
					env.create_object(res)
				}
					let args: Vec<_> = args_iter.iter(env).collect();
				LispObject::BuiltinVarargMacro(f) => {
					let res = f(env, &args)?;
					env.create_object(res)
				}
				_ => {
					return Err(RuntimeError::TypeError {
						expected: Some(LispType::Function),
						actual: None,
					});
				}
			}
		}
		LispObject::Atom(id) => env.get_stack_var(id)?,
		LispObject::Quasiquote(inner) => expand_once(env, *inner)?,
		LispObject::Array(xs) => {
			let xs = xs
				.clone()
				.iter()
				.map(|&x| eval_inner(env, x))
				.collect::<Result<Vec<_>, RuntimeError>>()?
				.into_boxed_slice();
			let obj = LispObject::Array(xs);
			env.create_object(obj)
		}
		_ => expr,
	};

	Ok(res)
}

fn expand_once<'a>(
	env: &mut Env<'a>,
	obj_ref: ObjectReference<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let obj = obj_ref.get(env).clone();
	let res = match obj {
		LispObject::Pair(head, tail) => {
			let head_expanded = expand_once(env, head)?;
			let tail_expanded = expand_once(env, tail)?;
			if (head, tail) == (head_expanded, tail_expanded) {
				obj_ref
			} else {
				env.create_object(LispObject::Pair(head_expanded, tail_expanded))
			}
		}
		LispObject::Array(object_references) => {
			let result: Vec<_> = object_references
				.iter()
				.map(|elem| expand_once(env, *elem))
				.collect::<Result<_, _>>()?;
			if result.as_slice() == object_references.as_ref() {
				obj_ref
			} else {
				env.create_object(LispObject::Array(result.into_boxed_slice()))
			}
		}
		LispObject::Lambda {
			params,
			ret_ty,
			body,
		} => {
			let result_body: Vec<_> = body
				.iter()
				.map(|expr| expand_once(env, *expr))
				.collect::<Result<_, _>>()?;
			if result_body == body {
				obj_ref
			} else {
				env.create_object(LispObject::Lambda {
					params,
					ret_ty,
					body: result_body,
				})
			}
		}
		LispObject::Macro { params, body } => {
			let result_body: Vec<_> = body
				.iter()
				.map(|expr| expand_once(env, *expr))
				.collect::<Result<Vec<_>, _>>()?;
			if result_body == body {
				obj_ref
			} else {
				env.create_object(LispObject::Macro {
					params,
					body: result_body,
				})
			}
		}

		LispObject::Unquote(expr) => eval_inner(env, expr)?,

		LispObject::BuiltinDyadic(_)
		| LispObject::BuiltinMonadic(_)
		| LispObject::BuiltinVarargMacro(_)
		| LispObject::Quote(_)
		| LispObject::Quasiquote(_)
		| LispObject::Atom(_)
		| LispObject::Integer(_)
		| LispObject::Float(_)
		| LispObject::String(_) => obj_ref,
	};
	Ok(res)
}

pub fn expand<'a>(
	env: &mut Env<'a>,
	params: &[SmallString],
	mut first_arg: ObjectReference<'a>,
	body: &[ObjectReference<'a>],
) -> Result<Vec<ObjectReference<'a>>, RuntimeError> {
	let mut stack_frame = Vec::new();
	for param_name in params.iter() {
		let arg = first_arg.next(env).ok_or(RuntimeError::NoCurryingMacro)?;
		stack_frame.push((param_name.clone(), arg));
	}
	if first_arg.next(env).is_some() {
		return Err(RuntimeError::TooManyArgumentsMacro);
	}

	env.stack.push(stack_frame);
	let res = body
		.iter()
		.copied()
		.map(|b| expand_once(env, b))
		.collect::<Result<Vec<_>, RuntimeError>>()?;
	env.stack.pop();
	Ok(res)
}
