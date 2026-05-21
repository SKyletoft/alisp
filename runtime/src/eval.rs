use crate::lisp_object::{LispObject, LispType, SmallString};

type Env = std::collections::HashMap<SmallString, LispObject>;

#[derive(Debug)]
pub enum RuntimeError {
	UndefinedVariable,
	TypeError {
		expected: Option<LispType>,
		actual: Option<LispType>,
	},
}

pub fn eval(obj: &LispObject, env: &mut Env) -> Result<LispObject, RuntimeError> {
	let res = match obj {
		LispObject::Pair(func, _args) => {
			let function = eval(func, env)?;
			let LispObject::Lambda {
				args: _,
				ret_ty: _,
				body,
			} = function
			else {
				return Err(RuntimeError::TypeError {
					expected: Some("function".into()),
					actual: function.type_of(),
				});
			};
			*body
		}
		LispObject::Atom(atom) => env
			.get(&*atom)
			.cloned()
			.ok_or(RuntimeError::UndefinedVariable)?,
		_ => obj.clone(),
	};
	Ok(res)
}

pub fn new_env() -> Env {
	[].into_iter().collect()
}
