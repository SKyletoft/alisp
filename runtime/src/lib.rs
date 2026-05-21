#![feature(deref_patterns)]

pub mod eval;
pub mod lisp_object;
pub mod parse;

#[cfg(test)]
mod test;

pub fn eval(code: &str) -> lisp_object::LispObject {
    let parsed = parse::parse(code).unwrap();
    let mut env = eval::new_env();
    eval::eval(&parsed, &mut env)
}
