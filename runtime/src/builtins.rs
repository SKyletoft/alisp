use smallvec::SmallVec;

use crate::{
	eval::RuntimeError,
	lisp_object::{Env, LispObject, ObjectReference, runtime_object::lisp_to_string},
	parse,
};

pub fn set<'a, const N: usize>(
	env: &mut Env<'a, N>,
	id: LispObject<'a, N>,
	val: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	if let LispObject::Quote(obj_ref) = id
		&& let LispObject::Atom(ident) = obj_ref.get(env)
	{
		let ident = ident.clone();
		*env.get_stack_var_mut(&ident) = val.clone();
		Ok(val)
	} else {
		Err(RuntimeError::AssignmentToNonVariable)
	}
}

pub fn add<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(LispObject::Float(x + y)),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok(LispObject::Integer(x + y)),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn mul<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(LispObject::Float(x * y)),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok(LispObject::Integer(x * y)),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn sub<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(LispObject::Float(x - y)),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok(LispObject::Integer(x - y)),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn div<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(LispObject::Float(x / y)),
		(LispObject::Integer(x), LispObject::Integer(y)) if y != 0 => {
			Ok(LispObject::Integer(x / y))
		}
		(LispObject::Integer(_), LispObject::Integer(_)) => Err(RuntimeError::DivisionByZero),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn r#mod<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Integer(x), LispObject::Integer(y)) if y != 0 => {
			Ok(LispObject::Integer(x % y))
		}
		(LispObject::Integer(_), LispObject::Integer(_)) => Err(RuntimeError::DivisionByZero),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn eq<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok((x == y).into()),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok((x == y).into()),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn lt<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok((x < y).into()),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok((x < y).into()),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn gt<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok((x > y).into()),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok((x > y).into()),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn le<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok((x <= y).into()),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok((x <= y).into()),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn ge<'a, const N: usize>(
	_: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok((x >= y).into()),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok((x >= y).into()),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn print<'a, const N: usize>(
	env: &mut Env<'a, N>,
	arg: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	print!("{}", lisp_to_string(env, &arg));
	Ok(LispObject::Atom("nil".into()))
}

pub fn println<'a, const N: usize>(
	env: &mut Env<'a, N>,
	arg: LispObject<'a, N>,
) -> Result<LispObject<'a, N>, RuntimeError> {
	println!("{}", lisp_to_string(env, &arg));
	Ok(LispObject::Atom("nil".into()))
}

pub fn r#macro<'a, const N: usize>(
	env: &mut Env<'a, N>,
	xs: &[ObjectReference<'a, N>],
) -> Result<LispObject<'a, N>, RuntimeError> {
	let [arr_ref, body @ ..] = xs else {
		return Err(RuntimeError::BrokenMacro {
			msg: "macro must have an argument array",
		});
	};
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

	if let Some(LispObject::Atom("->" | "→")) = body.first().map(|r| r.get(env)) {
		return Err(RuntimeError::BrokenMacro {
			msg: "macro should not have a return type",
		});
	}
	Ok(LispObject::Macro {
		params,
		body: body.to_vec(),
	})
}

pub fn lambda<'a, const N: usize>(
	env: &mut Env<'a, N>,
	xs: &[ObjectReference<'a, N>],
) -> Result<LispObject<'a, N>, RuntimeError> {
	let [arr_ref, rest @ ..] = xs else {
		return Err(RuntimeError::BrokenLambda {
			msg: "lambda must have an argument array",
		});
	};
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

	let (ret_ty, body) = match rest {
		[arrow, type_ref, body @ ..] if let LispObject::Atom("->" | "→") = arrow.get(env) => {
			let LispObject::Atom(type_name) = type_ref.get(env) else {
				return Err(RuntimeError::BrokenLambda {
					msg: "lambda return type expected after ->",
				});
			};
			let ty = crate::parse::parse_type(type_name);
			(Some(ty), body.to_vec())
		}
		[arrow, ..] if let LispObject::Atom("->" | "→") = arrow.get(env) => {
			return Err(RuntimeError::BrokenLambda {
				msg: "lambda return type expected after ->",
			});
		}
		body => (None, body.to_vec()),
	};
	Ok(LispObject::Lambda {
		params,
		ret_ty,
		body,
	})
}

pub fn defun<'a, const N: usize>(
	env: &mut Env<'a, N>,
	arg: &[ObjectReference<'a, N>],
) -> Result<LispObject<'a, N>, RuntimeError> {
	println(env, LispObject::Array(arg.into()))?;
	let [id, lam @ ..] = arg else {
		return Err(RuntimeError::BrokenLambda { msg: "from defun" });
	};
	let id = LispObject::Quote(*id);
	let val = lambda(env, lam)?;
	set(env, id, val)
}
