extern crate slack;

use failure::{Error, Fail};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;

use TerminalBackend;
use components::{input_manager, App, KeyManager};
use models::Message;

#[derive(Debug)]
pub enum Event {
    Error(Box<Error>),
    Tick,
    Input(Key),
    Connected,
    Disconnected,
    Message(Box<Message>),
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

pub fn run(
    app: &mut App,
    rtm: slack::RtmClient,
    terminal: &mut TerminalBackend,
) -> Result<(), Error> {
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
        thread::sleep(Duration::from_millis(200));
    });

    let slack_tx = tx.clone();
    thread::spawn(move || {
        rtm.run(&mut SlackEventHandler { tx: slack_tx });
    });

    // TODO: Move to App; but then KeyManager cannot take &mut of App anymore. Instead, give an
    // action enum back to the app so it can act on its own(?).
    let mut key_manager = KeyManager::new();

    loop {
        // Deal with new size, if resized
        let size = terminal.size()?;
        if app.size() != &size {
            terminal.resize(size)?;
            app.resize(size);
        }

        // Draw the App component to the terminal
        app.draw(terminal)?;

        // Handle any pending data (not blocking)
        if let Some(task_result) = app.loader_mut().pending_result() {
            app.accept_task_result(task_result)?;
        }

        // Handle any events (blocking)
        let evt = rx.recv()?;
        match evt {
            Event::Error(error) => return Err(*error),
            Event::Input(input) => match key_manager.handle_key(app, input) {
                input_manager::Outcome::Continue => {}
                input_manager::Outcome::Quit => break Ok(()),
            },
            Event::Connected => {}
            Event::Disconnected => {
                // TODO: Show disonnected status, try to reconnect, etc.
                break Err(format_err!(
                    "Slack disconnected. Offline mode is not yet implemented"
                ));
            }
            Event::Message(message) => app.state_mut().add_message(*message),
            Event::Tick => {}
        }
    }
}
