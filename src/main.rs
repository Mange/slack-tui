extern crate chrono;
extern crate dotenv;
extern crate slack;
extern crate termion;
extern crate tui;

#[allow(unused_imports)]
#[macro_use]
extern crate failure;

mod canvas;
mod channel_selector;
mod chat;
mod input_manager;
mod layout;
mod messages;
mod widgets;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::cell::{Cell, Ref, RefCell};

use chrono::prelude::*;
use tui::Terminal;
use tui::backend::MouseBackend;
use tui::layout::Rect;

use termion::event;
use termion::input::TermRead;

use failure::{Error, Fail, ResultExt};

use canvas::Canvas;
use chat::{Channel, ChannelID, ChannelList};
use input_manager::KeyManager;
use channel_selector::ChannelSelector;
use messages::Message;

pub type TerminalBackend = Terminal<MouseBackend>;

#[derive(Debug)]
enum Event {
    Error(Box<Error>),
    Tick,
    Input(event::Key),
    Connected,
    Disconnected,
    Message(Box<Message>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    History,
    SelectChannel,
}

pub struct App {
    channels: ChannelList,
    chat_canvas: RefCell<Option<Canvas>>,
    current_mode: Mode,
    history_scroll: usize,
    is_loading_more_messages: bool,
    last_chat_height: Cell<u16>,
    messages: messages::Buffer,
    selected_channel_id: Option<ChannelID>,
    size: Rect,
    team_name: String,

    // For Mode::SelectChannel
    channel_selector: ChannelSelector,
}

struct SlackEventHandler {
    tx: mpsc::Sender<Event>,
}

impl slack::EventHandler for SlackEventHandler {
    fn on_connect(&mut self, _rtm: &slack::RtmClient) {
        let _ = self.tx.send(Event::Connected);
    }

    fn on_close(&mut self, _rtm: &slack::RtmClient) {
        let _ = self.tx.send(Event::Disconnected);
    }

    fn on_event(&mut self, _rtm: &slack::RtmClient, slack_event: slack::Event) {
        match self.handle_event(slack_event) {
            Ok(_) => {}
            Err(error) => {
                self.tx.send(Event::Error(Box::new(error))).ok();
            }
        }
    }
}

impl SlackEventHandler {
    fn handle_event(&mut self, slack_event: slack::Event) -> Result<(), Error> {
        match slack_event {
            slack::Event::Message(msg) => self.new_message(*msg)?,
            _ => {}
        }

        Ok(())
    }

    fn new_message(&mut self, msg: slack::Message) -> Result<(), Error> {
        match Message::from_slack_message(&msg)? {
            Some(message) => self.tx.send(Event::Message(Box::new(message)))?,
            None => {}
        }
        Ok(())
    }
}

fn main() {
    dotenv::dotenv().ok();
    let mut terminal = match MouseBackend::new().and_then(|backend| Terminal::new(backend)) {
        Ok(val) => val,
        Err(error) => print_error_and_exit(error.into()),
    };

    match run(&mut terminal) {
        Ok(_) => {}
        Err(error) => {
            let _ = terminal.show_cursor();
            let _ = terminal.clear();
            drop(terminal);
            print_error_and_exit(error);
        }
    }
}

fn print_error_and_exit(error: Error) -> ! {
    for (i, cause) in error.causes().enumerate() {
        if i == 0 {
            eprintln!("Error: {}", cause);
        } else {
            let indentation = 4 * i;
            eprintln!("{0:1$}Caused by: {2}", "", indentation, cause);
        }

        #[cfg(debug_assertions)]
        {
            if let Some(backtrace) = cause.backtrace() {
                println!("{:#?}", backtrace);
            }
        }
    }
    eprintln!("\n...Sorry :(");

    ::std::process::exit(1);
}

fn run(terminal: &mut TerminalBackend) -> Result<(), Error> {
    let slack_api_token = ::std::env::var("SLACK_API_TOKEN")
        .context("Could not read SLACK_API_TOKEN environment variable")?;
    let rtm = slack::RtmClient::login(&slack_api_token).context("Could not log in to Slack")?;

    let size = terminal.size()?;
    let mut app = App::new(size, &rtm)?;
    app.run(terminal, rtm)
}

impl App {
    fn new(size: Rect, rtm: &slack::RtmClient) -> Result<App, Error> {
        let response = rtm.start_response();
        let channels: ChannelList = response
            .channels
            .clone()
            .expect("Slack did not provide a channel list on login")
            .iter()
            .flat_map(Channel::from_slack)
            .collect();
        // TODO: Pick a channel using a more intelligent way...
        let selected_channel_id = channels
            .iter()
            .find(|&(_id, channel)| channel.is_member())
            .map(|(id, _channel)| id.clone());

        let team_name = response
            .team
            .as_ref()
            .and_then(|team| team.name.as_ref())
            .cloned()
            .ok_or_else(|| format_err!("Slack did not provide a Team Name on login"))?;

        Ok(App {
            channel_selector: ChannelSelector::new(),
            channels,
            chat_canvas: RefCell::new(None),
            current_mode: Mode::History,
            history_scroll: 0,
            is_loading_more_messages: false,
            last_chat_height: Cell::new(0),
            messages: messages::Buffer::new(),
            selected_channel_id,
            size,
            team_name,
        })
    }

    fn run(&mut self, terminal: &mut TerminalBackend, rtm: slack::RtmClient) -> Result<(), Error> {
        terminal.clear()?;
        terminal.hide_cursor()?;

        let (tx, rx) = mpsc::channel();
        let input_tx = tx.clone();

        thread::spawn(move || {
            let stdin = io::stdin();
            for c in stdin.keys() {
                match c {
                    Ok(evt) => {
                        input_tx.send(Event::Input(evt)).ok();
                    }
                    Err(error) => {
                        let failure = error.context("Cannot parse STDIN bytes as an event");
                        input_tx.send(Event::Error(Box::new(failure.into()))).ok();
                        break;
                    }
                }
            }
        });

        let timer_tx = tx.clone();
        thread::spawn(move || loop {
            timer_tx.send(Event::Tick).ok();
            thread::sleep(time::Duration::from_millis(200));
        });

        let slack_tx = tx.clone();
        thread::spawn(move || {
            rtm.run(&mut SlackEventHandler { tx: slack_tx });
        });

        let mut key_manager = KeyManager::new();

        loop {
            let size = terminal.size()?;
            if size != self.size {
                terminal.resize(size)?;
                self.size = size;
                self.chat_canvas.replace(None);
            }
            self.draw(terminal)?;
            let evt = rx.recv()?;
            match evt {
                Event::Error(error) => return Err(*error),
                Event::Input(input) => match key_manager.handle_key(self, input) {
                    input_manager::Outcome::Continue => {}
                    input_manager::Outcome::Quit => break,
                },
                Event::Connected => {}
                Event::Disconnected => {
                    // TODO: Show disonnected status, try to reconnect, etc.
                    break;
                }
                Event::Message(message) => self.add_message(*message),
                Event::Tick => {}
            }
        }

        terminal.show_cursor()?;
        terminal.clear()?;
        Ok(())
    }

    fn current_history_scroll(&self) -> usize {
        self.history_scroll
            .min(self.last_chat_height.get() as usize)
    }

    fn selected_channel(&self) -> Option<&Channel> {
        self.selected_channel_id
            .as_ref()
            .and_then(|id| self.channels.get(id))
    }

    fn selected_channel_id(&self) -> Option<&ChannelID> {
        self.selected_channel_id.as_ref()
    }

    fn rendered_chat_canvas(
        &self,
        current_channel_id: &ChannelID,
        width: u16,
        height: u16,
    ) -> Ref<Canvas> {
        // Populate RefCell inside this scope when not present.
        {
            let mut cache = self.chat_canvas.borrow_mut();
            if cache.is_none() {
                let canvas = self.messages.render_as_canvas(
                    current_channel_id,
                    width,
                    self.is_loading_more_messages,
                );
                *cache = Some(canvas);
            }
        }

        self.last_chat_height.replace(height);

        // By now we know that chat_canvas is_some(), so unwrap should be safe. Option::as_ref
        // returns a new option of a reference to inner value, so it's fine to consume that Option.
        Ref::map(self.chat_canvas.borrow(), |option| option.as_ref().unwrap())
    }

    fn enter_mode(&mut self, new_mode: Mode) {
        self.current_mode = new_mode;
    }

    fn scroll_down(&mut self) {
        // NOTE: Scroll value is distance from bottom
        self.history_scroll = self.current_history_scroll().saturating_sub(1);
    }

    fn scroll_up(&mut self) {
        // NOTE: Scroll value is distance from bottom
        let chat_canvas_height = {
            match *self.chat_canvas.borrow() {
                Some(ref canvas) => canvas.height(),
                None => return,
            }
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

    fn select_channel_from_selector(&mut self) {
        let id = self.channel_selector.select(&self.channels);
        let message = format!(
            "Switching to channel {}",
            self.channels
                .get(&id)
                .map(Channel::name)
                .unwrap_or("(unknown channel)")
        );
        self.add_fake_message(Some(&message));
        self.selected_channel_id = Some(id);
    }

    fn toggle_loading_state(&mut self) {
        self.is_loading_more_messages = !self.is_loading_more_messages;
        self.chat_canvas.replace(None);
    }

    fn add_message(&mut self, message: Message) {
        self.messages.add(message);
        self.chat_canvas.replace(None);
    }

    fn add_fake_message(&mut self, msg: Option<&str>) {
        let time = Local::now();

        let message = match msg {
            Some(msg) => String::from(msg),
            None => format!("This is a fake message generated at: {}", time),
        };

        let channel_id = match self.selected_channel_id() {
            Some(val) => val.clone(),
            None => return,
        };

        self.messages.add(messages::StandardMessage {
            from: "Fake Message".into(),
            body: message,
            message_id: time.into(),
            thread_id: time.into(),
            channel_id,
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
