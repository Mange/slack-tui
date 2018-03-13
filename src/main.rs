extern crate termion;
extern crate tui;

mod widgets;
mod layout;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::widgets::{Block, Borders, SelectableList, Widget};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};

use termion::event;
use termion::input::TermRead;

use widgets::ChatHistory;

pub type TerminalBackend = Terminal<MouseBackend>;

enum Event {
    Tick,
    Input(event::Key),
}

struct Message {
    from: &'static str,
    body: &'static str,
}

pub struct App {
    size: Rect,
    messages: Vec<Message>,
    history_scroll: usize,
}

impl Message {
    fn render_as_string(&self) -> String {
        format!(
            "{{fg=cyan;mod=bold {from}}}\n{text}\n\n",
            from = self.from,
            text = self.body
        )
    }
}

fn main() {
    let backend = MouseBackend::new().unwrap();
    let terminal = Terminal::new(backend).unwrap();

    let messages = vec![
        Message {
            from: "Mange",
            body: "OMG",
        },
        Message {
            from: "Mange",
            body: "Does this really work?",
        },
        Message {
            from: "Socrates",
            body: "...well, yeah?",
        },
        Message {
            from: "Socrates",
            body: "What did you expect?",
        },
        Message {
            from: "Jonas",
            body: "lol did u RIIR for Slack?",
        },
        Message {
            from: "Christoffer",
            body: "RIIR?",
        },
        Message {
            from: "Jonas",
            body: "RIIR = Rewrite It In Rust",
        },
        Message {
            from: "Mange",
            body: "Rewrite-it-in-rust",
        },
        Message {
            from: "Jonas",
            body: ":smurf:",
        },
        Message {
            from: "Mange",
            body: ":okay:",
        },
        Message {
            from: "Christoffer",
            body: "🏅 Mange for being wasteful of your life",
        },
        Message {
            from: "Christoffer",
            body: "You only get to live, what? Like 70 years or so. And you spend",
        },
        Message {
            from: "Christoffer",
            body: "it on RIIR Slack now?",
        },
        Message {
            from: "Mange",
            body: ":(",
        },
        Message {
            from: "Mange",
            body: "Lucky me that this isn't the real Chstistoffer",
        },
        Message {
            from: "Christoffer",
            body: "Exactly, I'm just a figment of your imagination.",
        },
    ];

    let size = terminal.size().unwrap();
    let mut app = App {
        history_scroll: 0,
        messages,
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
        self.history_scroll = self.history_scroll.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.history_scroll = self.history_scroll.saturating_sub(1);
    }

    fn draw(&mut self, terminal: &mut TerminalBackend) -> Result<(), io::Error> {
        let size = self.size;

        let history_scroll = self.history_scroll;

        layout::render(self, terminal);
        terminal.draw()
    }
}
