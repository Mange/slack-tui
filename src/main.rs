extern crate chrono;
extern crate dotenv;
extern crate slack;
extern crate termion;
extern crate tui;

#[macro_use]
extern crate serde_derive;

mod canvas;
mod chat;
mod widgets;
mod layout;
mod messages;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::cell::{Cell, Ref, RefCell};

use chrono::prelude::*;
use tui::Terminal;
use tui::backend::MouseBackend;
use tui::layout::Rect;

use slack::api as slack_api;

use termion::event;
use termion::input::TermRead;

use canvas::Canvas;
use chat::{Channel, ChannelID, ChannelList};

pub type TerminalBackend = Terminal<MouseBackend>;

enum Event {
    Tick,
    Input(event::Key),
    Connected,
    Disconnected,
}

pub struct App {
    size: Rect,
    messages: messages::Buffer,
    history_scroll: usize,
    chat_canvas: RefCell<Option<Canvas>>,
    last_chat_height: Cell<u16>,
    channels: ChannelList,
    selected_channel_id: Option<ChannelID>,
}

impl App {
    fn current_history_scroll(&self) -> usize {
        self.history_scroll
            .min(self.last_chat_height.get() as usize)
    }

    fn selected_channel(&self) -> Option<&Channel> {
        self.selected_channel_id
            .as_ref()
            .and_then(|id| self.channels.get(id))
    }

    fn rendered_chat_canvas(&self, width: u16, height: u16) -> Ref<Canvas> {
        // Populate RefCell inside this scope when not present.
        {
            let mut cache = self.chat_canvas.borrow_mut();
            if cache.is_none() {
                let canvas = self.messages.render_as_canvas(width);
                *cache = Some(canvas);
            }
        }

        self.last_chat_height.replace(height);

        Ref::map(self.chat_canvas.borrow(), |option| option.as_ref().unwrap())
    }

    fn prepopulate(&mut self, response: slack_api::rtm::StartResponse) {
        if let Some(channels) = response.channels {
            self.channels = channels.iter().flat_map(Channel::from_slack).collect();
            self.selected_channel_id = channels
                .iter()
                .find(|c| c.is_member.unwrap_or(false))
                .and_then(|c| c.id.clone().map(ChannelID::from));
        }
    }
}

struct SlackEventHandler {
    tx: mpsc::Sender<Event>,
}

impl slack::EventHandler for SlackEventHandler {
    fn on_connect(&mut self, rtm: &slack::RtmClient) {
        self.tx.send(Event::Connected);
    }
    fn on_close(&mut self, rtm: &slack::RtmClient) {
        self.tx.send(Event::Disconnected);
    }
    fn on_event(&mut self, rtm: &slack::RtmClient, slack_event: slack::Event) {}
}

fn main() {
    dotenv::dotenv().ok();

    let terminal = Terminal::new(MouseBackend::new().unwrap()).unwrap();

    let size = terminal.size().unwrap();
    let mut app = App {
        history_scroll: 0,
        messages: messages::Buffer::new(),
        chat_canvas: RefCell::new(None),
        last_chat_height: Cell::new(0),
        channels: ChannelList::new(),
        selected_channel_id: None,
        size,
    };
    app.run(terminal).unwrap();
}

impl App {
    fn run(&mut self, mut terminal: TerminalBackend) -> Result<(), io::Error> {
        let rtm = match slack::RtmClient::login(&::std::env::var("SLACK_API_TOKEN").unwrap()) {
            Ok(client) => client,
            Err(error) => panic!("Failed to login to Slack: {}", error),
        };
        self.prepopulate(rtm.start_response().clone());

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

        let timer_tx = tx.clone();
        thread::spawn(move || loop {
            timer_tx.send(Event::Tick).unwrap();
            thread::sleep(time::Duration::from_millis(200));
        });

        let slack_tx = tx.clone();
        thread::spawn(move || {
            rtm.run(&mut SlackEventHandler { tx: slack_tx });
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
                    event::Key::Char('b') => self.create_fake_message(),
                    event::Key::Char('B') => self.add_loading_message(),
                    _ => {}
                },
                Event::Connected => self.add_loading_message(),
                Event::Disconnected => {
                    // TODO: Show disonnected status, try to reconnect, etc.
                    break;
                }
                Event::Tick => {}
            }
        }

        terminal.show_cursor()?;
        terminal.clear()?;
        Ok(())
    }

    fn scroll_down(&mut self) {
        // NOTE: Scroll value is distance from bottom
        self.history_scroll = self.current_history_scroll().saturating_sub(1);
    }

    fn scroll_up(&mut self) {
        // NOTE: Scroll value is distance from bottom
        let chat_canvas_height = {
            let last_canvas = self.chat_canvas.borrow();
            if last_canvas.is_none() {
                return;
            }
            last_canvas.as_ref().unwrap().height()
        };
        let chat_viewport_height = self.last_chat_height.get();

        // If the canvas is smaller than the viewport, lock to bottom.
        if chat_canvas_height <= chat_viewport_height {
            self.history_scroll = 0;
        } else {
            let max_scroll = chat_canvas_height - chat_viewport_height;
            self.history_scroll = (self.current_history_scroll() + 1).min(max_scroll as usize);
        }
    }

    fn add_loading_message(&mut self) {
        let time = Local::now();

        self.messages.add(messages::LoadingMessage {
            from_timestamp: time.timestamp_subsec_millis().to_string(),
        });
        self.chat_canvas.replace(None);
    }

    fn create_fake_message(&mut self) {
        let time = Local::now();

        self.messages.add(messages::StandardMessage {
            from: "Fake Message".into(),
            body: format!("This is a fake message generated at: {}", time),
            timestamp: time.timestamp_subsec_millis().to_string(),
        });
        self.chat_canvas.replace(None);
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
