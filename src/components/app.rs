extern crate slack;
extern crate termion;

use tui::layout::Rect;
use failure::{Error, Fail};

use TerminalBackend;
use models::*;
use components::*;
use data::loader;
use data::loader::Loader;

#[derive(Debug)]
pub struct App {
    size: Rect,

    state: AppState,
    loader: Loader,

    // Components
    // TODO pub key_manager: KeyManager,
    pub channel_selector: ChannelSelector,
}

impl App {
    pub fn new(state: AppState, loader: Loader, size: Rect) -> App {
        App {
            channel_selector: ChannelSelector::new(),
            //TODO key_manager: KeyManager::new(),
            loader,
            size,
            state,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    pub fn size(&self) -> &Rect {
        &self.size
    }

    pub fn resize(&mut self, size: Rect) {
        self.size = size;
        self.state.chat_canvas.replace(None);
    }

    pub fn loader_mut(&mut self) -> &mut Loader {
        &mut self.loader
    }

    // pub fn handle_key(&mut self, input: termion::event::Key) -> input_manager::Outcome {
    //     self.key_manager.handle_key(self, input)
    // }

    pub fn select_channel_from_selector(&mut self) -> Result<(), Error> {
        if let Some(id) = self.channel_selector.select(&self.state.channels) {
            self.async_load_channel_history(&id)?;
            self.state.select_channel(id)
        } else {
            Ok(())
        }
    }

    pub fn async_load_channel_history(&mut self, channel_id: &ChannelID) -> Result<(), Error> {
        self.state.set_loading_state(true);
        self.loader.load_channel_history(channel_id, None)
    }

    pub fn accept_task_result(&mut self, result: loader::TaskResult) -> Result<(), Error> {
        use data::loader::TaskResult;
        match result {
            TaskResult::ChannelHistory(channel_id, response) => {
                self.state.set_loading_state(false);
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
                    let side_channel = MessageSideChannel {
                        channel_id: Some(channel_id),
                        ..Default::default()
                    };

                    for message in messages.into_iter() {
                        match Message::from_slack_message(&message, &side_channel) {
                            Ok(Some(message)) => self.state.add_message(message),
                            Ok(None) => {}
                            Err(error) => self.state.add_error_message(error.context(
                                "Could not convert Slack message to internal representation",
                            )),
                        }
                    }
                }
                Ok(())
            }
            Err(error) => {
                self.state
                    .add_error_message(error.context("Could not load channel history"));
                Ok(())
            }
        }
    }

    pub fn draw(&mut self, terminal: &mut TerminalBackend) -> Result<(), Error> {
        layout::render(&self, terminal, &self.size);
        terminal.draw().map_err(|e| e.into())
    }
}
