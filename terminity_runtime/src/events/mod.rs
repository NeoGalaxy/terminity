use crossterm::event::KeyEvent;
use terminity::events::{Event, KeyPress, KeyRelease, Mouse, MouseButton, MouseKind, Position};
use terminity::events::{KeyCode, KeyModifiers};

pub(crate) fn from_crossterm(ct_evt: crossterm::event::Event) -> Option<Event> {
	match ct_evt {
		crossterm::event::Event::FocusGained => Some(Event::FocusChange { has_focus: true }),

		crossterm::event::Event::FocusLost => Some(Event::FocusChange { has_focus: false }),

		crossterm::event::Event::Key(
			k @ KeyEvent { kind: crossterm::event::KeyEventKind::Release, .. },
		) => Some(Event::KeyRelease(KeyRelease {
			code: match k.code {
				crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
				crossterm::event::KeyCode::Enter => KeyCode::Enter,
				crossterm::event::KeyCode::Left => KeyCode::Left,
				crossterm::event::KeyCode::Right => KeyCode::Right,
				crossterm::event::KeyCode::Up => KeyCode::Up,
				crossterm::event::KeyCode::Down => KeyCode::Down,
				// crossterm::event::KeyCode::Home => KeyCode::Home,
				// crossterm::event::KeyCode::End => KeyCode::End,
				crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
				crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
				crossterm::event::KeyCode::Tab => KeyCode::Tab,
				crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
				crossterm::event::KeyCode::Delete => KeyCode::Delete,
				// crossterm::event::KeyCode::Insert => KeyCode::Insert,
				crossterm::event::KeyCode::F(f) => KeyCode::F(f),
				crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
				// crossterm::event::KeyCode::Null => KeyCode::Null,
				crossterm::event::KeyCode::Esc => KeyCode::Esc,
				// crossterm::event::KeyCode::CapsLock => KeyCode::CapsLock,
				// crossterm::event::KeyCode::ScrollLock => KeyCode::ScrollLock,
				// crossterm::event::KeyCode::NumLock => KeyCode::NumLock,
				// crossterm::event::KeyCode::PrintScreen => KeyCode::PrintScreen,
				// crossterm::event::KeyCode::Pause => KeyCode::Pause,
				// crossterm::event::KeyCode::Menu => KeyCode::Menu,
				// crossterm::event::KeyCode::KeypadBegin => KeyCode::KeypadBegin,
				// crossterm::event::KeyCode::Media(_) => KeyCode::Media(_),
				// crossterm::event::KeyCode::Modifier(_) => KeyCode::Modifier(_),
				_ => return None,
			},
			modifiers: modifiers_from_crossterm(&k.modifiers, Some(&k.state)),
		})),

		crossterm::event::Event::Key(k) => Some(Event::KeyPress(KeyPress {
			code: match k.code {
				crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
				crossterm::event::KeyCode::Enter => KeyCode::Enter,
				crossterm::event::KeyCode::Left => KeyCode::Left,
				crossterm::event::KeyCode::Right => KeyCode::Right,
				crossterm::event::KeyCode::Up => KeyCode::Up,
				crossterm::event::KeyCode::Down => KeyCode::Down,
				// crossterm::event::KeyCode::Home => KeyCode::Home,
				// crossterm::event::KeyCode::End => KeyCode::End,
				crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
				crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
				crossterm::event::KeyCode::Tab => KeyCode::Tab,
				crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
				crossterm::event::KeyCode::Delete => KeyCode::Delete,
				// crossterm::event::KeyCode::Insert => KeyCode::Insert,
				crossterm::event::KeyCode::F(f) => KeyCode::F(f),
				crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
				// crossterm::event::KeyCode::Null => KeyCode::Null,
				crossterm::event::KeyCode::Esc => KeyCode::Esc,
				// crossterm::event::KeyCode::CapsLock => KeyCode::CapsLock,
				// crossterm::event::KeyCode::ScrollLock => KeyCode::ScrollLock,
				// crossterm::event::KeyCode::NumLock => KeyCode::NumLock,
				// crossterm::event::KeyCode::PrintScreen => KeyCode::PrintScreen,
				// crossterm::event::KeyCode::Pause => KeyCode::Pause,
				// crossterm::event::KeyCode::Menu => KeyCode::Menu,
				// crossterm::event::KeyCode::KeypadBegin => KeyCode::KeypadBegin,
				// crossterm::event::KeyCode::Media(_) => KeyCode::Media(_),
				// crossterm::event::KeyCode::Modifier(_) => KeyCode::Modifier(_),
				_ => return None,
			},
			modifiers: modifiers_from_crossterm(&k.modifiers, Some(&k.state)),
			repeated: k.kind == crossterm::event::KeyEventKind::Repeat,
		})),

		crossterm::event::Event::Mouse(m) => Some(Event::Mouse(Mouse {
			kind: match m.kind {
				crossterm::event::MouseEventKind::Down(b) => {
					MouseKind::Down(button_from_crossterm(&b))
				}
				crossterm::event::MouseEventKind::Up(b) => MouseKind::Up(button_from_crossterm(&b)),
				crossterm::event::MouseEventKind::Drag(b) => {
					MouseKind::Drag(button_from_crossterm(&b))
				}
				crossterm::event::MouseEventKind::Moved => MouseKind::Moved,
				crossterm::event::MouseEventKind::ScrollDown => MouseKind::ScrollDown,
				crossterm::event::MouseEventKind::ScrollUp => MouseKind::ScrollUp,
			},
			position: Position { line: m.row, column: m.column },
			modifiers: modifiers_from_crossterm(&m.modifiers, None),
		})),

		crossterm::event::Event::Paste(_) => None,

		crossterm::event::Event::Resize(_, _) => None,
	}
}

fn modifiers_from_crossterm(
	mods: &crossterm::event::KeyModifiers,
	state: Option<&crossterm::event::KeyEventState>,
) -> KeyModifiers {
	KeyModifiers {
		shift: mods.contains(crossterm::event::KeyModifiers::SHIFT),
		control: mods.contains(crossterm::event::KeyModifiers::CONTROL),
		alt: mods.contains(crossterm::event::KeyModifiers::ALT),
		start: mods.contains(crossterm::event::KeyModifiers::SUPER),
		hyper: mods.contains(crossterm::event::KeyModifiers::HYPER),
		meta: mods.contains(crossterm::event::KeyModifiers::META),
		keypad: state.map_or(false, |s| s.contains(crossterm::event::KeyEventState::KEYPAD)),
		caps_lock: state.map_or(false, |s| s.contains(crossterm::event::KeyEventState::CAPS_LOCK)),
		num_lock: state.map_or(false, |s| s.contains(crossterm::event::KeyEventState::NUM_LOCK)),
	}
}

fn button_from_crossterm(button: &crossterm::event::MouseButton) -> MouseButton {
	match button {
		crossterm::event::MouseButton::Left => MouseButton::Left,
		crossterm::event::MouseButton::Right => MouseButton::Right,
		crossterm::event::MouseButton::Middle => MouseButton::Middle,
	}
}
