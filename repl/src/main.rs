use runtime::{
	eval,
	lisp_object::{self, Env, ObjectReference},
	parse,
};

fn main() {
	let code = "(println \"Hello World!\")";
	let mut env = Env::wait_for_new();
	let parsed = parse::parse_many(code).unwrap();
	let obj = parsed.into_iter().fold(env.nil(), |_, node| {
		let obj = ObjectReference::from_parse_object(node, &mut env);
		eval::eval(obj, &mut env).unwrap()
	});
	let res = lisp_object::lisp_object_to_parse_tree(obj.get(&env), &env);
	println!("{res}")
}
