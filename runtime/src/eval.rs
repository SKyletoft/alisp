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
					let result = eval(body, env)?;
					type_guard(&ret_ty, &Some(result.get(env).type_of()))?;
					result
				}
				LispObject::BuiltinDyadic { f } => {
					let l_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let l_evalled = eval(l_ref, env)?;
					let l = env.get(l_evalled).clone();

					let r_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let r_evalled = eval(r_ref, env)?;
					let r = env.get(r_evalled).clone();

					f(l, r, env)?
				}
				LispObject::BuiltinMonadic { f } => {
					let arg_ref = args_iter.next(env).ok_or(RuntimeError::NoCurrying)?;
					let evalled = eval(arg_ref, env)?;
					let arg = env.get(evalled).clone();
					f(arg, env)?
				}
				_ => {
					return Err(RuntimeError::TypeError {
						expected: Some("function".into()),
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
