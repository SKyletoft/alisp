use smallvec::SmallVec;

use crate::{
	lisp_object::{Env, LispObject, LispType, ObjectReference, SmallString},
	parse,
};

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
	expr: ObjectReference<'a>,
	env: &mut Env<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let res = match env.get(expr) {
		LispObject::Quasiquote(inner) => expand_once(env, *inner)?,
		LispObject::Quote(inner) => *inner,
		_ => eval_inner(expr, env)?,
	};
	Ok(res)
}

pub fn eval_inner<'a>(
	expr: ObjectReference<'a>,
	env: &mut Env<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let res = match env.get(expr) {
		LispObject::Pair(f, xs) if let LispObject::Atom("lambda") = f.get(env) => {
			eval_lambda_object(env, *xs)?
		}
		LispObject::Pair(f, xs) if let LispObject::Atom("macro") = f.get(env) => {
			eval_macro_object(env, *xs)?
		}
		LispObject::Pair(f, x) => {
			let mut args_iter = *x;
			let function = eval_inner(*f, env)?.get(env).clone();
			match function {
				LispObject::Lambda {
					params,
					ret_ty,
					body,
				} => {
					if env.stack.len() > RECURSION_LIMIT {
						return Err(RuntimeError::StackOverflow);
					}
					let mut stack_frame = Vec::new();
					for (param_name, param_type) in params.iter() {
						let arg = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
						let evalled_arg = eval_top(arg, env)?;
						type_guard(param_type, &Some(evalled_arg.get(env).type_of()))?;
						stack_frame.push((param_name.clone(), evalled_arg));
					}
					env.stack.push(stack_frame);
					if args_iter.get(env).next(env).is_some() {
						return Err(RuntimeError::TooManyArguments);
					}
					let result = body
						.into_iter()
						.map(|e| eval_top(e, env))
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
						.map(|b| eval_top(b, env))
						.last()
						.unwrap_or(Ok(expr))?
				}
				LispObject::BuiltinDyadic(f) => {
					let l_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let l_evalled = eval_inner(l_ref, env)?;
					let l = env.get(l_evalled).clone();

					let r_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let r_evalled = eval_inner(r_ref, env)?;
					let r = env.get(r_evalled).clone();

					f(l, r, env)?
				}
				LispObject::BuiltinMonadic(f) => {
					let arg_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let evalled = eval_inner(arg_ref, env)?;
					let arg = env.get(evalled).clone();
					f(arg, env)?
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

		LispObject::Unquote(expr) => eval_inner(expr, env)?,

		LispObject::BuiltinDyadic(_)
		| LispObject::BuiltinMonadic(_)
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

fn eval_macro_object<'a>(
	env: &mut Env<'a>,
	mut xs: ObjectReference<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let arr_ref = xs.next(env).ok_or(RuntimeError::BrokenMacro {
		msg: "macro must have an argument array",
	})?;
	let LispObject::Array(args) = arr_ref.get(env) else {
		return Err(RuntimeError::BrokenMacro {
			msg: "macro must have an argument array",
		});
	};

	let params = parse::parse_macro_args(
		args.iter()
			.map(|arg_ref| match arg_ref.get(env) {
				LispObject::Atom(name) => Ok(name.clone()),
				_ => Err(RuntimeError::BrokenMacro {
					msg: "Non-argument in argument position",
				}),
			})
			.collect::<Result<SmallVec<_>, _>>()?,
	)
	.map_err(|msg| RuntimeError::BrokenMacro { msg })?;

	let Some(body_first) = xs.next(env) else {
		return Err(RuntimeError::BrokenMacro {
			msg: "macro must have a body",
		});
	};
	let body = match body_first.get(env) {
		LispObject::Atom("->" | "→") => {
			return Err(RuntimeError::BrokenMacro {
				msg: "macro should not have a return type",
			});
		}
		_ => std::iter::chain(std::iter::once(body_first), xs.iter(env)).collect(),
	};
	Ok(env.create_object(LispObject::Macro { params, body }))
}

fn eval_lambda_object<'a>(
	env: &mut Env<'a>,
	mut xs: ObjectReference<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let arr_ref = xs.next(env).ok_or(RuntimeError::BrokenLambda {
		msg: "lambda must have an argument array",
	})?;
	let LispObject::Array(args) = arr_ref.get(env) else {
		return Err(RuntimeError::BrokenLambda {
			msg: "lambda must have an argument array",
		});
	};

	let params = parse::parse_lambda_args(
		args.iter()
			.map(|arg_ref| match arg_ref.get(env) {
				LispObject::Atom(name) => Ok((name.clone(), None)),
				LispObject::Pair(name_ref, rest_ref)
					if let LispObject::Atom(name) = name_ref.get(env)
						&& let LispObject::Pair(ty_ref, nil_ref) = rest_ref.get(env)
						&& let LispObject::Atom(type_name) = ty_ref.get(env)
						&& let LispObject::Atom("nil") = nil_ref.get(env) =>
				{
					let ty = crate::parse::parse_type(type_name);
					Ok((name.clone(), Some(ty)))
				}
				_ => Err(RuntimeError::BrokenLambda {
					msg: "Non-argument in argument position",
				}),
			})
			.collect::<Result<SmallVec<_>, _>>()?,
	)
	.map_err(|msg| RuntimeError::BrokenLambda { msg })?;

	let Some(body_first) = xs.next(env) else {
		return Err(RuntimeError::BrokenLambda {
			msg: "lambda must have a body",
		});
	};
	let (ret_ty, body) = match body_first.get(env) {
		LispObject::Atom("->" | "→")
			if let Some(type_name_ref) = xs.next(env)
				&& let LispObject::Atom(type_name) = type_name_ref.get(env) =>
		{
			let ty = crate::parse::parse_type(type_name);
			let body = xs.iter(env).collect();
			(Some(ty), body)
		}
		LispObject::Atom("->" | "→") => {
			return Err(RuntimeError::BrokenLambda {
				msg: "lambda return type expected after ->",
			});
		}
		_ => {
			let body = std::iter::chain(std::iter::once(body_first), xs.iter(env)).collect();
			(None, body)
		}
	};
	Ok(env.create_object(LispObject::Lambda {
		params,
		ret_ty,
		body,
	}))
}
