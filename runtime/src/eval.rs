use crate::lisp_object::{LispParseTree, LispType, SmallString};

type Env = std::collections::HashMap<SmallString, LispParseTree>;

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
}

fn type_guard(a: &Option<LispType>, b: &Option<LispType>) -> Result<(), RuntimeError> {
	match (a, b) {
		(Some(x), Some(y)) if x != y => Err(RuntimeError::TypeError {
			expected: a.clone(),
			actual: b.clone(),
		}),
		_ => Ok(()),
	}
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
