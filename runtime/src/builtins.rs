use crate::{
	eval::RuntimeError,
	lisp_object::{Env, LispObject, ObjectReference, runtime_object::lisp_to_string},
};

pub fn set<'a, const N: usize>(
	env: &mut Env<'a, N>,
	id: LispObject<'a, N>,
	val: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	if let LispObject::Quote(obj_ref) = id
		&& let LispObject::Atom(ident) = obj_ref.get(env)
	{
		let ident = ident.clone();
		*env.get_stack_var_mut(&ident) = val;
		Ok(obj_ref)
	} else {
		Err(RuntimeError::AssignmentToNonVariable)
	}
}

pub fn add<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
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
	}
}

pub fn mul<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
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
	}
}

pub fn sub<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
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
	}
}

pub fn div<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => {
			Ok(env.create_object(LispObject::Float(x / y)))
		}
		(LispObject::Integer(x), LispObject::Integer(y)) if y != 0 => {
			Ok(env.create_object(LispObject::Integer(x / y)))
		}
		(LispObject::Integer(_), LispObject::Integer(_)) => Err(RuntimeError::DivisionByZero),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn r#mod<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Integer(x), LispObject::Integer(y)) if y != 0 => {
			Ok(env.create_object(LispObject::Integer(x % y)))
		}
		(LispObject::Integer(_), LispObject::Integer(_)) => Err(RuntimeError::DivisionByZero),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn eq<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(env.create_object((x == y).into())),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok(env.create_object((x == y).into())),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn lt<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(env.create_object((x < y).into())),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok(env.create_object((x < y).into())),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn gt<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(env.create_object((x > y).into())),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok(env.create_object((x > y).into())),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn le<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(env.create_object((x <= y).into())),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok(env.create_object((x <= y).into())),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn ge<'a, const N: usize>(
	env: &mut Env<'a, N>,
	l: LispObject<'a, N>,
	r: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	match (l, r) {
		(LispObject::Float(x), LispObject::Float(y)) => Ok(env.create_object((x >= y).into())),
		(LispObject::Integer(x), LispObject::Integer(y)) => Ok(env.create_object((x >= y).into())),
		(l, r) => Err(RuntimeError::TypeError {
			expected: Some(l.type_of()),
			actual: Some(r.type_of()),
		}),
	}
}

pub fn print<'a, const N: usize>(
	env: &mut Env<'a, N>,
	arg: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	print!("{}", lisp_to_string(env, &arg));
	Ok(env.create_object(LispObject::Atom("nil".into())))
}

pub fn println<'a, const N: usize>(
	env: &mut Env<'a, N>,
	arg: LispObject<'a, N>,
) -> Result<ObjectReference<'a, N>, RuntimeError> {
	println!("{}", lisp_to_string(env, &arg));
	Ok(env.create_object(LispObject::Atom("nil".into())))
}
