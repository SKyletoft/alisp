use crate::lisp_object::{LispParseTree, LispType, SmallString};

type Env = std::collections::HashMap<SmallString, LispParseTree>;

#[derive(Debug)]
pub enum RuntimeError {
	UndefinedVariable,
	TypeError {
		expected: Option<LispType>,
		actual: Option<LispType>,
	},
}

pub fn eval(obj: &LispParseTree, env: &mut Env) -> Result<LispParseTree, RuntimeError> {
	let res = match obj {
		LispParseTree::Pair(func, _args) => {
			let function = eval(func, env)?;
			let LispParseTree::Lambda {
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
		LispParseTree::Atom(atom) => env
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
