use runtime::{
	eval,
	lisp_object::{Env, LispParseTree},
	parse,
};

fn main() {
	let res = eval("(print \"Hello world\")");
	println!("{res}")
}

pub fn eval(code: &str) -> LispParseTree {
	let parsed = parse::parse(code).unwrap();
	let mut env = Env::new().unwrap();
	eval::eval(&parsed, &mut env).unwrap()
}
