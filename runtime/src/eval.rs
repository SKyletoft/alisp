use crate::lisp_object::{LispParseTree, LispType, SmallString, Env};

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
		LispParseTree::Pair(func, args) => {
			let mut args = args.as_ref().clone();
			let function = eval(func, env)?;
			let LispParseTree::Lambda {
				params,
				ret_ty,
				body,
			} = function
			else {
				return Err(RuntimeError::TypeError {
					expected: Some("function".into()),
					actual: function.type_of(),
				});
			};
			let mut params_iter = params.into_iter().peekable();
			for (arg, (_, param_type)) in std::iter::zip(&mut args, &mut params_iter) {
				let evaled_arg = eval(&arg, env)?;
				type_guard(&evaled_arg.type_of(), &param_type)?;
			}
			if args.peek().is_some() {
				Err(RuntimeError::TooManyArguments)?;
			}
			if params_iter.peek().is_some() {
				Err(RuntimeError::NoCurrying)?;
			}
			let ret = *body;
			type_guard(&ret_ty, &ret.type_of())?;
			ret
		}
		// LispParseTree::Atom(atom) => env
		//	.get(atom)
		//	.cloned()
		//	.ok_or(RuntimeError::UndefinedVariable)?,
		_ => obj.clone(),
	};
	Ok(res)
}
