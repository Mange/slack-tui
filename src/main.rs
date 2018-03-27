extern crate chrono;
extern crate dotenv;
extern crate slack;
extern crate termion;
extern crate tui;

#[allow(unused_imports)]
#[macro_use]
extern crate failure;

mod data;
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
use chat::{Channel, ChannelID, ChannelList, User, UserID, UserList};
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
    chat_canvas: RefCell<Option<Canvas>>,
    current_mode: Mode,
    history_scroll: usize,
    last_chat_height: Cell<u16>,
    selected_channel_id: Option<ChannelID>,
    size: Rect,

    // Data
    channels: ChannelList,
    is_loading_more_messages: bool,
    loader: data::loader::Loader,
    messages: messages::Buffer,
    team_name: String,
    users: UserList,

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
        match Message::from_slack_message(&msg, None)? {
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

fn format_error_with_causes<E: Fail>(error: E) -> String {
    error
        .causes()
        .enumerate()
        .map(|(i, cause)| {
            #[cfg(debug_assertions)]
            let backtrace = if let Some(backtrace) = cause.backtrace() {
                format!("\n{:#?}\n", backtrace)
            } else {
                String::new()
            };

            #[cfg(not(debug_assertions))]
            let backtrace = String::new();

            if i == 0 {
                format!("Error: {}{}", cause, backtrace)
            } else {
                let indentation = 4 * i;
                format!(
                    "\n{0:1$}Caused by: {2}{3}",
                    "", indentation, cause, backtrace
                )
            }
        })
        .collect()
}

fn run(terminal: &mut TerminalBackend) -> Result<(), Error> {
    let slack_api_token = ::std::env::var("SLACK_API_TOKEN")
        .context("Could not read SLACK_API_TOKEN environment variable")?;
    let rtm = slack::RtmClient::login(&slack_api_token).context("Could not log in to Slack")?;

    let size = terminal.size()?;
    let mut app = App::new(&slack_api_token, size, &rtm)?;
    app.run(terminal, rtm)
}

impl App {
    fn new(slack_api_token: &str, size: Rect, rtm: &slack::RtmClient) -> Result<App, Error> {
        let response = rtm.start_response();

        let users: UserList = response
            .users
            .clone()
            .expect("Slack did not provide a user list on login")
            .iter()
            .flat_map(User::from_slack)
            .collect();

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
            loader: data::loader::Loader::create(slack_api_token)?,
            messages: messages::Buffer::new(),
            selected_channel_id,
            size,
            team_name,
            users,
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

        if let Some(channel_id) = self.selected_channel_id.clone() {
            self.async_load_channel_history(&channel_id)?;
        }

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

            if let Some(task_result) = self.loader.pending_result() {
                self.accept_task_result(task_result)?;
            }
        }

        terminal.show_cursor()?;
        terminal.clear()?;
        Ok(())
    }

    fn max_history_scroll(&self) -> usize {
        // NOTE: Scroll value is distance from bottom
        let chat_canvas_height = {
            match *self.chat_canvas.borrow() {
                Some(ref canvas) => canvas.height(),
                None => return 0,
            }
        };
        let chat_viewport_height = self.last_chat_height.get();

        // If the canvas is smaller than the viewport, lock to bottom.
        if chat_canvas_height <= chat_viewport_height {
            0
        } else {
            chat_canvas_height as usize - chat_viewport_height as usize
        }
    }

    fn current_history_scroll(&self) -> usize {
        self.history_scroll.min(self.max_history_scroll())
    }

    fn chat_height(&self) -> u16 {
        self.last_chat_height.get()
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

    fn clear_chat_canvas_cache(&self) {
        self.chat_canvas.replace(None);
    }

    fn enter_mode(&mut self, new_mode: Mode) {
        self.current_mode = new_mode;
    }

    fn scroll_down(&mut self, amount: usize) {
        // NOTE: Scroll value is distance from bottom
        self.history_scroll = self.current_history_scroll().saturating_sub(amount);
    }

    fn scroll_up(&mut self, amount: usize) {
        self.history_scroll =
            (self.history_scroll.saturating_add(amount)).min(self.max_history_scroll());
    }

    fn select_channel(&mut self, id: ChannelID) -> Result<(), Error> {
        self.async_load_channel_history(&id)?;
        self.selected_channel_id = Some(id);
        self.history_scroll = 0;
        self.clear_chat_canvas_cache();
        Ok(())
    }

    fn select_channel_from_selector(&mut self) -> Result<(), Error> {
        let id = self.channel_selector.select(&self.channels);
        self.select_channel(id)
    }

    fn toggle_loading_state(&mut self) {
        let new_state = !self.is_loading_more_messages;
        self.set_loading_state(new_state);
    }

    fn set_loading_state(&mut self, state: bool) {
        self.is_loading_more_messages = state;
        self.clear_chat_canvas_cache();
    }

    fn add_message(&mut self, message: Message) {
        self.messages.add(message);
        self.clear_chat_canvas_cache();
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
        self.clear_chat_canvas_cache();
    }

    fn add_error_message<E: Fail>(&mut self, error: E) {
        let channel_id = match self.selected_channel_id {
            Some(ref val) => val,
            None => return,
        };
        self.messages
            .add(messages::ErrorMessage::from_error(channel_id, error));
        self.clear_chat_canvas_cache();
    }

    fn async_load_channel_history(&mut self, channel_id: &ChannelID) -> Result<(), Error> {
        self.set_loading_state(true);
        self.loader.load_channel_history(channel_id, None)
    }

    fn accept_task_result(&mut self, result: data::loader::TaskResult) -> Result<(), Error> {
        use data::loader::TaskResult;
        match result {
            TaskResult::ChannelHistory(channel_id, response) => {
                self.set_loading_state(false);
                self.accept_channel_history(channel_id, response)
            }
        }
    }

    fn accept_channel_history(
        &mut self,
        channel_id: ChannelID,
        response: Result<
            slack::api::channels::HistoryResponse,
            slack::api::channels::HistoryError<slack::api::requests::Error>,
        >,
    ) -> Result<(), Error> {
        match response {
            Ok(response) => {
                if let Some(messages) = response.messages {
                    let side_channel = messages::MessageSideChannel {
                        channel_id: Some(channel_id),
                        ..Default::default()
                    };

                    for message in messages.into_iter() {
                        match Message::from_slack_message(&message, &side_channel) {
                            Ok(Some(message)) => self.add_message(message),
                            Ok(None) => {}
                            Err(error) => self.add_error_message(error.context(
                                "Could not convert Slack message to internal representation",
                            )),
                        }
                    }
                }
                Ok(())
            }
            Err(error) => {
                self.add_error_message(error.context("Could not load channel history"));
                Ok(())
            }
        }
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
