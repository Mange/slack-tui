extern crate slack;
use std::sync::mpsc;
use std::thread;
use slack::api;

use failure::Error;

use models::{ChannelID, MessageID};

#[derive(Debug)]
enum Task {
    ChannelHistory(ChannelID, Option<MessageID>),
}

#[derive(Debug)]
pub enum TaskResult {
    ChannelHistory(
        ChannelID,
        Result<api::channels::HistoryResponse, api::channels::HistoryError<api::requests::Error>>,
    ),
}

struct BackgroundLoader {
    requests: mpsc::Receiver<Task>,
    results: mpsc::Sender<TaskResult>,
    client: api::requests::Client,
    slack_api_key: String,
}

#[derive(Debug)]
pub struct Loader {
    requests: mpsc::Sender<Task>,
    results: mpsc::Receiver<TaskResult>,
}

impl Loader {
    pub fn create(slack_api_key: &str) -> Result<Loader, Error> {
        let (requests_tx, requests_rx) = mpsc::channel();
        let (results_tx, results_rx) = mpsc::channel();
        let api_key = slack_api_key.to_owned();

        let mut bl = BackgroundLoader::new(api_key, requests_rx, results_tx)?;
        thread::spawn(move || {
            bl.run();
        });

        Ok(Loader {
            requests: requests_tx,
            results: results_rx,
        })
    }

    pub fn pending_result(&mut self) -> Option<TaskResult> {
        self.results.try_recv().ok()
    }

    pub fn load_channel_history(
        &mut self,
        channel_id: &ChannelID,
        before_message_id: Option<&MessageID>,
    ) -> Result<(), Error> {
        self.requests
            .send(Task::ChannelHistory(
                channel_id.clone(),
                before_message_id.cloned(),
            ))
            .map_err(|e| e.into())
    }
}

impl BackgroundLoader {
    fn new(
        slack_api_key: String,
        requests: mpsc::Receiver<Task>,
        results: mpsc::Sender<TaskResult>,
    ) -> Result<BackgroundLoader, Error> {
        Ok(BackgroundLoader {
            requests,
            results,
            slack_api_key,
            client: api::requests::default_client()?,
        })
    }

    fn run(&mut self) {
        loop {
            let task = match self.requests.recv() {
                Ok(val) => val,
                Err(_) => break,
            };

            match task {
                Task::ChannelHistory(channel_id, before_message_id) => {
                    self.load_channel_history(channel_id, before_message_id)
                }
            }
        }
    }

    fn load_channel_history(
        &mut self,
        channel_id: ChannelID,
        before_message_id: Option<MessageID>,
    ) {
        let latest = before_message_id.map(|id| id.as_string());
        let response = slack::api::channels::history(
            &self.client,
            &self.slack_api_key,
            &slack::api::channels::HistoryRequest {
                channel: channel_id.as_str(),
                latest: latest.as_ref().map(String::as_ref),

                ..Default::default()
            },
        );
        self.results
            .send(TaskResult::ChannelHistory(channel_id, response))
            .ok();
    }
}
