use std::collections::HashMap;
use crossterm::QueueableCommand;
use std::io::Write;

mod stratego;
mod sttt;

pub struct GameWrapper {
	game: Box<dyn Game + Send + Sync>,
	pub name: &'static str
}

lazy_static! {
    /// This is an example for using doc comment attributes
    pub static ref REGISTERY: HashMap<&'static str, GameWrapper> = {
        let mut m = HashMap::new();
        m.insert("SuperTicTacToe", GameWrapper {
        	game: Box::new(sttt::SuperTTT()),
        	name: "SuperTicTacToe"
        }
        );
		m.insert("Stratego", GameWrapper {
			game: Box::new(stratego::Stratego()),
			name: "Stratego"
		}
		);
        m
    };
}

impl GameWrapper {
	pub fn run(&self, out: &mut dyn Write) -> std::io::Result<()> {
    	crossterm::terminal::enable_raw_mode()?;
	    out.queue(crossterm::terminal::EnterAlternateScreen)?
	            .queue(crossterm::cursor::SavePosition)?;
        let res = self.game.run(out);
	    out.queue(crossterm::terminal::LeaveAlternateScreen)?
	            .queue(crossterm::cursor::RestorePosition)?
	            .flush()?;
	    crossterm::terminal::disable_raw_mode()?;
	    res
	}
}

trait Game {
	fn run(&self, out: &mut dyn Write) -> std::io::Result<()>;
}

pub fn get(name: &str) -> Option<&'static GameWrapper> {
	REGISTERY.get(name)
}
