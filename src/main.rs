extern crate termion;
extern crate tui;

mod widgets;
mod layout;
mod message_buffer;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::widgets::Widget;
use tui::layout::Rect;

use termion::event;
use termion::input::TermRead;

use message_buffer::{Message, MessageBuffer};

pub type TerminalBackend = Terminal<MouseBackend>;

enum Event {
    Tick,
    Input(event::Key),
}

pub struct App {
    size: Rect,
    messages: MessageBuffer,
    history_scroll: usize,
}

fn main() {
    let backend = MouseBackend::new().unwrap();
    let terminal = Terminal::new(backend).unwrap();

    let mut messages = vec![
        Message {
            timestamp: "1111111.0".to_owned(),
            from: "Mange",
            body: "OMG",
        },
        Message {
            timestamp: "1111112.0".to_owned(),
            from: "Mange",
            body: "Does this really work?",
        },
        Message {
            timestamp: "1111113.0".to_owned(),
            from: "Socrates",
            body: "...well, yeah?",
        },
        Message {
            timestamp: "1111114.0".to_owned(),
            from: "Socrates",
            body: "What did you expect?",
        },
        Message {
            timestamp: "1111115.0".to_owned(),
            from: "Jonas",
            body: "lol did u RIIR for Slack?",
        },
        Message {
            timestamp: "1111116.0".to_owned(),
            from: "Christoffer",
            body: "RIIR?",
        },
        Message {
            timestamp: "1111117.0".to_owned(),
            from: "Jonas",
            body: "RIIR = Rewrite It In Rust",
        },
        Message {
            timestamp: "1111118.0".to_owned(),
            from: "Mange",
            body: "Rewrite-it-in-rust",
        },
        Message {
            timestamp: "1111119.0".to_owned(),
            from: "Jonas",
            body: ":smurf:",
        },
        Message {
            timestamp: "1111120.0".to_owned(),
            from: "Mange",
            body: ":okay:",
        },
        Message {
            timestamp: "1111121.0".to_owned(),
            from: "Christoffer",
            body: "🏅 Mange for being wasteful of your life",
        },
        Message {
            timestamp: "1111122.0".to_owned(),
            from: "Christoffer",
            body: "You only get to live, what? Like 70 years or so. And you spend",
        },
        Message {
            timestamp: "1111123.0".to_owned(),
            from: "Christoffer",
            body: "it on RIIR Slack now?",
        },
        Message {
            timestamp: "1111124.0".to_owned(),
            from: "Mange",
            body: ":(",
        },
        Message {
            timestamp: "1111125.0".to_owned(),
            from: "Mange",
            body: "Lucky me that this isn't the real Chstistoffer",
        },
        Message {
            timestamp: "1111126.0".to_owned(),
            from: "Christoffer",
            body: "Exactly, I'm just a figment of your imagination.",
        },
    ];

    for n in (0..50) {
        messages.insert(
            n,
            Message {
                timestamp: (1110001.0 + n as f32).to_string(),
                from: "Example",
                body: "Yet another example that you can use to test scrolling and other nice things like that. Also this line should wrap in most window sizes that you are realisticly using when developing this UI prototype. At least on common font sizes.\nRight?",
            }
        );
    }

    let size = terminal.size().unwrap();
    let mut app = App {
        history_scroll: 0,
        messages: messages.into(),
        size,
    };
    app.run(terminal).unwrap();
}

impl App {
    fn run(&mut self, mut terminal: TerminalBackend) -> Result<(), io::Error> {
        terminal.clear()?;
        terminal.hide_cursor()?;

        let (tx, rx) = mpsc::channel();
        let input_tx = tx.clone();

        thread::spawn(move || {
            let stdin = io::stdin();
            for c in stdin.keys() {
                let evt = c.unwrap();
                input_tx.send(Event::Input(evt)).unwrap();
                if evt == event::Key::Char('q') {
                    break;
                }
            }
        });
        thread::spawn(move || {
            let tx = tx.clone();
            loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(time::Duration::from_millis(200));
            }
        });

        loop {
            let size = terminal.size()?;
            if size != self.size {
                terminal.resize(size)?;
                self.size = size;
            }
            self.draw(&mut terminal)?;
            let evt = rx.recv().unwrap();
            match evt {
                Event::Input(input) => match input {
                    event::Key::Char('q') => break,
                    event::Key::Char('j') => self.scroll_down(),
                    event::Key::Char('k') => self.scroll_up(),
                    _ => {}
                },
                Event::Tick => {}
            }
        }

        terminal.show_cursor()?;
        terminal.clear()?;
        Ok(())
    }

    fn scroll_down(&mut self) {
        // NOTE: Scroll value is distance from bottom
        self.history_scroll = self.history_scroll.saturating_sub(1);
    }

    fn scroll_up(&mut self) {
        // NOTE: Scroll value is distance from bottom
        // TODO: How to prevent scrolling past the end of the history? Do we need to render a
        // canvas here too?
        self.history_scroll = self.history_scroll.saturating_add(1);
    }

    fn draw(&mut self, terminal: &mut TerminalBackend) -> Result<(), io::Error> {
        let size = self.size;

        let history_scroll = self.history_scroll;

        layout::render(self, terminal);
        terminal.draw()
    }
}
