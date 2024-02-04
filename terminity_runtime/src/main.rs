use std::convert::Infallible;
use std::fmt::Write;
use std::fs::File;
use std::sync::mpsc::TrySendError;
use std::{fmt::Display, sync::mpsc, thread};

use terminity::{Event, EventPoller, Game, Widget};

fn main() {}

struct LineDisp<'a, W: Widget + ?Sized>(usize, &'a W);

impl<W: Widget + ?Sized> Display for LineDisp<'_, W> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.1.display_line(f, self.0)
	}
}

struct E;

impl EventPoller for E {}
impl Iterator for E {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		todo!()
	}
}

#[repr(C)]
struct GameHandler {}

fn game_runner() {
	let (display_send_size, _display_rcv_size) = mpsc::sync_channel(10);
	let (display_send, _display_rcv) = mpsc::sync_channel(20);
	let (_buffer_send, buffer_rcv) = mpsc::sync_channel(20);
	thread::spawn(move || {
		let mut buffers = Vec::with_capacity(20);
		for _ in 0..20 {
			buffers.push(String::with_capacity(20));
		}

		#[allow(clippy::let_unit_value)]
		let mut game = <()>::start::<File>(None);

		loop {
			game.disp(|w| {
				let (width, height) = w.size();
				display_send_size.send((width, height)).unwrap();
				for i in 0..height {
					let mut buff = buffers.pop().unwrap_or_else(|| buffer_rcv.recv().unwrap());
					write!(&mut buff, "{}", LineDisp(i, w)).unwrap();
					match display_send.try_send(buff) {
						Err(TrySendError::Full(b)) => buffers.push(b),
						Err(_) => {
							panic!()
						}
						Ok(()) => (),
					}
				}
			});
			game.update(E);
		}
	});
}
