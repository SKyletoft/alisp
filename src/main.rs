#![feature(deref_patterns)]

use lisp_object::LispObject;

mod eval;
mod lisp_object;
mod parse;
#[cfg(test)]
mod test;

fn main() {
	println!("Hello, world!");
}

fn eval(code: &str) -> LispObject {
	todo!()
}
