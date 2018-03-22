extern crate serde_yaml;
extern crate termion;
extern crate tui;

#[macro_use]
extern crate serde_derive;

mod canvas;
mod widgets;
mod layout;
mod message_buffer;

use std::io;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::fs::File;
use std::cell::{Ref, RefCell};

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::layout::Rect;

use termion::event;
use termion::input::TermRead;

use message_buffer::{Message, MessageBuffer};
use canvas::Canvas;

pub type TerminalBackend = Terminal<MouseBackend>;

enum Event {
    Tick,
    Input(event::Key),
}

pub struct App {
    size: Rect,
    messages: MessageBuffer,
    history_scroll: usize,
    chat_canvas: RefCell<Option<Canvas>>,
}

impl App {
    fn rendered_chat_canvas(&self, width: u16) -> Ref<Canvas> {
        // Populate RefCell inside this scope when not present.
        {
            let mut cache = self.chat_canvas.borrow_mut();
            if cache.is_none() {
                let canvas = self.messages.render_as_canvas(width);
                *cache = Some(canvas);
            }
        }

        Ref::map(self.chat_canvas.borrow(), |option| option.as_ref().unwrap())
    }
}

fn load_fixtures() -> Vec<Message> {
    let file = File::open("fixtures.yml").unwrap();
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).unwrap()
}

fn main() {
    let backend = MouseBackend::new().unwrap();
    let terminal = Terminal::new(backend).unwrap();

    let mut messages = load_fixtures();

    for n in 0..50 {
        messages.insert(
            n,
            Message {
                timestamp: (1110001.0 + n as f32).to_string(),
                from: "Example".into(),
                body: "Yet another example that you can use to test scrolling and other nice things like that. Also this line should wrap in most window sizes that you are realisticly using when developing this UI prototype. At least on common font sizes.\nRight?".into(),
            }
        );
    }

    let size = terminal.size().unwrap();
    let mut app = App {
        history_scroll: 0,
        messages: messages.into(),
        chat_canvas: RefCell::new(None),
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
                self.chat_canvas.replace(None);
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
        layout::render(self, terminal);
        terminal.draw()
    }
}

#[allow(unused)]
pub(crate) fn render_buffer(buf: &tui::buffer::Buffer) -> String {
    let mut s = format!("Buffer area: {:?}\r\n", buf.area());
    let width = buf.area().width;
    for (i, cell) in buf.content().iter().enumerate() {
        if i > 0 && i as u16 % width == 0 {
            s.push_str("\r\n");
        }
        s.push(cell.symbol.chars().next().unwrap());
    }
    s
}
