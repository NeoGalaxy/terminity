# The Terminity project

This project's goal is to democratize a bit more the use of terminals, especially for small games.

Terminity sets up an environment for games to run in, and registers multiple games to play.

This project is at an extremely early development stage. To try it, clone the project, go into
`/terminity` and run `cargo run Chess` or `cargo run SuperTicTacToe` (with
[Rust installed](https://www.rust-lang.org/tools/install)). Currently, it has only been tested on
Ubuntu's `gnome-terminal`, please let me know if any other environment works/doesn't work. It isn't
expected to work on windows 8 and older, but might work thanks to
[crossterm](https://crates.io/crates/crossterm).

The very long term goals of this project are to to:

 1. Make it easier to build good UI in terminal
 2. Make terminal games accessible to anyone and everyone. This would be through a windows `.exe` and a smartphone app.
 3. Setup a P2P (peer to peer) system allowing to play the games online with anyone, and giving the
	programmers an API to setup an online (or local offline) game without a mandatory need for a server. 

This project is of course not feasible alone. If you want to support this project in any way,
anyway from being part of the community to be an active developer, please do!! If enough people are
interested, a Discord and/or a Subreddit are to expect.
