use crate::lisp_object::LispObject;

pub fn parse(code: &str) -> Result<LispObject, ()> {
	match code.chars().next() {
		None => Err(()),
		Some('(') => {
			let (inner, rest) = until_close(&code[1..])?;
			dbg!(inner, rest);
			todo!()
		}
		Some(digit) if digit.is_ascii_digit() => todo!(),
		Some('"') => todo!(),
		Some('\'') => todo!(),
		Some(c) => todo!(),
	}
}

fn until_close(line: &str) -> Result<(&str, &str), ()> {
	let idx = line
		.chars()
		.scan(1, |acc, curr| {
			let this = match curr {
				'(' => 1,
				')' => -1,
				_ => 0,
			};
			*acc += this;
			Some(*acc)
		})
		.position(|x| x == 0)
		.ok_or(())?;
	Ok(line.split_at(idx))
}

#[cfg(test)]
mod test {
	use crate::lisp_object::LispObject;

	#[test]
	fn parse() {
		let code = "(1 2 3)";
		let expected = LispObject::from(vec![1, 2, 3]);
		let result = super::parse(&code);
		assert_eq!(result, Ok(expected));
	}
}
