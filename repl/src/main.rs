use runtime::{eval, lisp_object::LispObject, parse};

fn main() {
	let res = eval("(print \"Hello world\")");
	println!("{res}")
}

pub fn eval(code: &str) -> LispObject {
	let parsed = parse::parse(code).unwrap();
	let mut env = eval::new_env();
	eval::eval(&parsed, &mut env).unwrap()
}
