use crossterm::terminal::Clear;
use std::fmt::{Formatter, Display};

pub mod widgets;

pub trait Widget {
	fn displ_line(&self, f: &mut Formatter<'_>, line: u16) -> std::fmt::Result;
	fn size(&self) -> &(u16, u16);
}

impl Display for dyn Widget {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
	    for i in 0..self.size().1 {
	    	self.displ_line(f, i)?;
	    	f.write_str(&format!("{}\n\r", Clear(crossterm::terminal::ClearType::UntilNewLine)))?;
	    }
	    Ok(())
	}
}
