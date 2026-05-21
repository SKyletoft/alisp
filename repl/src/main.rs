fn main() {
	let res = runtime::eval("(print \"Hello world\")");
	println!("{res}")
}
