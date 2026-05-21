use crate::lisp_object::{LispObject, SmallString};

type Env = std::collections::HashMap<SmallString, LispObject>;

pub fn eval(obj: &LispObject, env: &mut Env) -> LispObject {
	match obj {
		LispObject::Pair(func, args) => {
			let mut args = args.as_ref().clone();
			let LispObject::Lambda {
				args,
				ret_ty: _,
				body,
			} = eval(func, env)
			else {
				panic!("Type error: {func} is not a function")
			};
			*body
		}
		LispObject::Atom(atom) => env[&*atom].clone(),
		_ => obj.clone(),
	}
}

pub fn new_env() -> Env {
	[].into_iter().collect()
}
