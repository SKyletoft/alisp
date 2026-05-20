#![feature(deref_patterns)]

use lisp_object::LispObject;

mod eval;
mod lisp_object;
mod parse;
#[cfg(test)]
mod test;

fn main() {
	let res = eval("(print \"Hello world\")");
	println!("{res}")
}

fn eval(code: &str) -> LispObject {
	let parsed = parse::parse(code).unwrap();
	let mut env = eval::new_env();
	eval::eval(&parsed, &mut env)
}
