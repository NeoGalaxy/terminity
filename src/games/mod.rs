pub mod stratego;
pub mod sttt;

pub trait CLGame {
	fn run(term: console::Term) -> std::io::Result<()>;
}
