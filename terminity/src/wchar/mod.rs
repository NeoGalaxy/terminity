use std::ops::Deref;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WChar(char);

pub struct ControlCharError;

impl TryFrom<char> for WChar {
	type Error = ControlCharError;

	fn try_from(value: char) -> Result<Self, Self::Error> {
		if value.is_control() {
			Err(ControlCharError)
		} else {
			Ok(Self(value))
		}
	}
}

impl Deref for WChar {
	type Target = char;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl WChar {
	pub fn width(&self) -> u16 {
		// Only control chars return None. Here, we can't have a control char
		self.0.width().expect("WChar internal error: a non Cc char has no width.") as u16
	}

	/// # Safety
	///
	/// It is only safe to use this function with characters that are not control characters.
	/// Consider using instead `wchar!('<char>')` for a static check or
	/// `let <wchar_name>: WChar = <char_variable>.try_into()` for a dynamic one.
	pub unsafe fn from_char_unchecked(c: char) -> Self {
		Self(c)
	}
}
