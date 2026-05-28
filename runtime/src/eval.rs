use smallvec::SmallVec;

use crate::lisp_object::{Env, LispObject, LispType, ObjectReference};

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum RuntimeError {
	#[display("Undefined variable")]
	UndefinedVariable,
	#[display("Type error: {expected:?} ≠ {actual:?}")]
	TypeError {
		expected: Option<LispType>,
		actual: Option<LispType>,
	},
	#[display("Too many arguments")]
	TooManyArguments,
	#[display("alisp doesn't support curried functions (= not enough arguments in function call)")]
	NoCurrying,
	#[display("Division by zero")]
	DivisionByZero,
	#[display("Assignment to non-identifier")]
	AssignmentToNonVariable,
	#[display("Invalid lambda construction: {msg}")]
	BrokenLambda { msg: &'static str },
}

fn type_guard(expected: &Option<LispType>, actual: &Option<LispType>) -> Result<(), RuntimeError> {
	match (expected, actual) {
		(Some(x), Some(y)) if x != y => Err(RuntimeError::TypeError {
			expected: expected.clone(),
			actual: actual.clone(),
		}),
		_ => Ok(()),
	}
}

pub fn eval<'a>(
	expr: ObjectReference<'a>,
	env: &mut Env<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let obj = env.get(expr);
	let res = match obj {
		LispObject::Pair(f, xs) if matches!(f.get(env), LispObject::Atom("lambda")) => {
			eval_lambda(env, *xs)?
		}
		LispObject::Pair(f, x) => {
			let mut args_iter = *x;
			let function = eval(*f, env)?.get(env).clone();
			match function {
				LispObject::Lambda {
					params,
					ret_ty,
					body,
				} => {
					let ret_ty = ret_ty.clone();
					for (param_name, param_type) in params.clone().into_iter() {
						let arg = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
						let evalled_arg = eval(arg, env)?;
						type_guard(&param_type, &Some(evalled_arg.get(env).type_of()))?;
						env.stack.push((param_name.clone(), evalled_arg));
					}
					if args_iter.get(env).next(env).is_some() {
						return Err(RuntimeError::TooManyArguments);
					}
					let result = body
						.into_iter()
						.map(|e| eval(e, env))
						.last()
						.unwrap_or(Ok(expr))?;
					type_guard(&ret_ty, &Some(result.get(env).type_of()))?;
					result
				}
				LispObject::BuiltinDyadic(f) => {
					let l_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let l_evalled = eval(l_ref, env)?;
					let l = env.get(l_evalled).clone();

					let r_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let r_evalled = eval(r_ref, env)?;
					let r = env.get(r_evalled).clone();

					f(l, r, env)?
				}
				LispObject::BuiltinMonadic(f) => {
					let arg_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let evalled = eval(arg_ref, env)?;
					let arg = env.get(evalled).clone();
					f(arg, env)?
				}
				func => {
					dbg!(func, args_iter);
					return Err(RuntimeError::TypeError {
						expected: Some(LispType::Function),
						actual: None,
					});
				}
			}
		}
		LispObject::Atom(id) => env
			.stack
			.iter()
			.rev()
			.find(|(s, _)| id == s)
			.map(|(_, val)| *val)
			.ok_or(RuntimeError::UndefinedVariable)?,
		_ => expr,
	};

	Ok(res)
}

fn eval_lambda<'a>(
	env: &mut Env<'a>,
	mut xs: ObjectReference<'a>,
) -> Result<ObjectReference<'a>, RuntimeError> {
	let arr_ref = xs.next(env).ok_or(	RuntimeError::BrokenLambda {
		msg: "lambda must have an argument array",
	})?;
	let LispObject::Array(args) = arr_ref.get(env) else {
		return Err(RuntimeError::BrokenLambda {
			msg: "lambda must have an argument array",
		});
	};
	let args = args
		.iter()
		.map(|arg_ref| match arg_ref.get(env) {
			LispObject::Atom(name) => Ok((name.clone(), None)),
			LispObject::Pair(name_ref, rest_ref)
				if let LispObject::Atom(name) = name_ref.get(env)
					&& let LispObject::Pair(ty_ref, nil_ref) = rest_ref.get(env)
					&& let LispObject::Atom(ty_str) = ty_ref.get(env)
					&& let LispObject::Atom("nil") = nil_ref.get(env) =>
			{
				let ty = match ty_str.as_str() {
					"i32" => LispType::Integer,
					"f64" => LispType::Float,
					id => LispType::Named(id.into()),
				};
				Ok((name.clone(), Some(ty)))
			}
			_ => Err(RuntimeError::BrokenLambda {
				msg: "Non-argument in argument position",
			}),
		})
		.collect::<Result<SmallVec<_>, _>>()?;
	let Some(body_first) = xs.next(env) else {
		return Err(RuntimeError::BrokenLambda { msg: "lambda must have a body" });
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
			let body = [body_first].into_iter().chain(xs.iter(env)).collect();
			(None, body)
		}
	};
	Ok(env.create_object(LispObject::Lambda {
		params: args,
		ret_ty,
		body,
	}))
}
