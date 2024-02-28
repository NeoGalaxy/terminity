use std::ops::Deref;

use crate::{wchar::WChar, widgets::Widget};

#[derive(Debug)]
pub struct WidgetLine<'a> {
	pub(super) width: u16,
	pub(super) content: &'a str,
}

#[derive(Debug)]
pub struct WidgetLineBuffer {
	width: u16,
	content: String,
}

impl Deref for WidgetLine<'_> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.content
	}
}

impl Deref for WidgetLineBuffer {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.content
	}
}

impl<'a> WidgetLine<'a> {
	pub fn width(&self) -> u16 {
		self.width
	}

	pub fn content(&self) -> &str {
		self.content
	}

	///
	/// # Safety
	///
	/// This has to be a valid WidgetLine content. That means that:
	///
	/// * The first cell of the slice shall be the width of the string
	/// *
	pub unsafe fn from_parts_unchecked(v: &'a str, w: u16) -> Self {
		Self { width: w, content: v }
	}

	// Remaining methods to do:
	//
	// chars
	// char_indices
	// get
	// get_mut
	// into_string
	// lines
	// repeat
	// replace
	// replacen
	// match
	// match_indices
	// rmatch
	// rmatch_indices
	// split*
	// rsplit*
	// strip*
	// to_*
	// trim*
	//
	// as_bytes_mut
	// as_mut_ptr
	// make_ascii_*
	//
	//
	// traits:
	//
	// Add<&'a str>
	// Add<&str>
	// AddAssign<&'a str>
	// AddAssign<&str>
	// AsMut<str>
	// AsRef<[u8]>
	// AsRef<OsStr>
	// AsRef<Path>
	// AsRef<str>
	// AsciiExt
	// Borrow<str>
	// BorrowMut<str>
	// Clone for
	// Concat<str>
	// Debug
	// Default
	// Display
	// 'a> Extend<&'a str>
	// From<&mut str>
	// From<&str> for
	// From<&str> for Box
	// 'a> From<&str> for Box<dyn Error + Sync + Send +
	// From<&str> for
	// 'a> From<&'a str> for Cow<
	// From<&str> for
	// From<&str>
	// From<&str> for
	// From<Cow<'_, str>> for
	// From<String> for
	// 'a, 'b> FromIterator<&'b str> for Cow<
	// 'a> FromIterator<&'a str>
	// Hash
	// I> Index<I>
	// I> IndexMut<I>
	// S> Join<&str> for
	// Ord
	// 'a, 'b> PartialEq<&'b str> for Cow<
	// PartialEq<&str>
	// 'a, 'b> PartialEq<&'a str>
	// 'a, 'b> PartialEq<Cow<'a, str>> for &
	// 'a, 'b> PartialEq<Cow<'a, str>>
	// PartialEq<OsStr>
	// 'a> PartialEq<OsString> for &
	// PartialEq<OsString>
	// 'a, 'b> PartialEq<String> for &
	// 'a, 'b> PartialEq<String>
	// 'a, 'b> PartialEq<str> for Cow<
	// PartialEq<str>
	// PartialEq<str>
	// 'a, 'b> PartialEq<str>
	// PartialEq
	// PartialOrd<str>
	// PartialOrd<str>
	// PartialOrd
	// 'a, 'b> Pattern<'a> for &
	// SliceIndex<str> for (Bound<usize>,
	// SliceIndex<str> for
	// SliceIndex<str> for
	// SliceIndex<str>
	// SliceIndex<str> for
	// SliceIndex<str> for
	// SliceIndex<str> for
	// ToOwned
	// ToSocketAddrs
	// 'a> TryFrom<&'a OsStr> for &
	// ConstParamTy
	// Eq
	// StructuralEq
	// StructuralPartialEq
}

impl Widget for WidgetLine<'_> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		write!(f, "{}", &**self)
	}

	fn size(&self) -> crate::Size {
		crate::Size { width: self.width(), height: 1 }
	}
}

impl WidgetLineBuffer {
	pub fn width(&self) -> u16 {
		self.width
	}

	// Remaining methods to do:
	//
	// as_mut_str
	// as_mut_vec
	// as_str
	// clear
	// drain
	// insert
	// insert_str
	// into_*
	// leak
	// new
	// pop
	// push
	// push_str
	// remove
	// replace_range
	// reserve*
	// retain
	// shrink_to*
	// split_off
	// truncate
	// try_reserve*
	// with_capacity
	//
	//
	//
	// Traits:
	//
	// Add<&str>
	// AddAssign<&str>
	// AsMut<str>
	// AsRef<OsStr>
	// AsRef<Path>
	// AsRef<[u8]>
	// AsRef<str>
	// Borrow<str>
	// BorrowMut<str>
	// Clone
	// Debug
	// Default
	// Deref
	// DerefMut
	// Display
	// Eq
	// Extend<&'a char>
	// Extend<&'a str>
	// Extend<Box<str>>
	// Extend<Cow<'a, str>>
	// Extend<String>
	// Extend<char>
	// From<&'a String>
	// From<&String>
	// From<&mut str>
	// From<&str>
	// From<Box<str>>
	// From<Cow<'a, str>>
	// From<String>
	// From<char>
	// FromIterator<&'a char>
	// FromIterator<&'a str>
	// FromIterator<Box<str>>
	// FromIterator<Cow<'a, str>>
	// FromIterator<String>
	// FromIterator<String>
	// FromIterator<char>
	// FromStr
	// Hash
	// Index<Range<usize>>
	// Index<RangeFrom<usize>>
	// Index<RangeFull>
	// Index<RangeInclusive<usize>>
	// Index<RangeTo<usize>>
	// Index<RangeToInclusive<usize>>
	// IndexMut<Range<usize>>
	// IndexMut<RangeFrom<usize>>
	// IndexMut<RangeFull>
	// IndexMut<RangeInclusive<usize>>
	// IndexMut<RangeTo<usize>>
	// IndexMut<RangeToInclusive<usize>>
	// Ord
	// PartialEq
	// PartialEq<&'a str>
	// PartialEq<Cow<'a, str>>
	// PartialEq<String>
	// PartialEq<String>
	// PartialEq<String>
	// PartialEq<str>
	// PartialOrd
	// Pattern<'a>
	// StructuralEq
	// StructuralPartialEq
	// ToSocketAddrs
}

#[derive(Debug, Clone, Copy)]
pub struct NondisplayableChar(char);

impl<'a> TryFrom<&'a str> for WidgetLine<'a> {
	type Error = NondisplayableChar;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		let mut width = 0;
		for c in value.chars() {
			if let Ok(c) = WChar::try_from(c) {
				width += c.width();
			} else {
				return Err(NondisplayableChar(c));
			}
		}
		Ok(Self { width, content: value })
	}
}

impl TryFrom<String> for WidgetLineBuffer {
	type Error = NondisplayableChar;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let mut width = 0;
		for c in value.chars() {
			if let Ok(c) = WChar::try_from(c) {
				width += c.width();
			} else {
				return Err(NondisplayableChar(c));
			}
		}
		Ok(Self { width, content: value })
	}
}
