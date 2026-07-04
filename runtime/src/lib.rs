#![feature(deref_patterns)]

pub mod builtins;
pub mod eval;
pub mod lisp_object;
pub mod parse;

#[cfg(test)]
mod test;
