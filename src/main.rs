extern crate chrono;
extern crate dotenv;
extern crate slack;
extern crate termion;
extern crate tui;

#[macro_use]
extern crate failure;

/// Contains stateful components - parts of the app.
///
/// Should depend on `models` as components usually need to contain state.
mod components;

/// Contains data conversion and loading classes. (Loading data from Slack, for example)
///
/// Should depend on `models` as they should be the results of loading data.
mod data;

/// Contains structs storing data to keep track of state.
///
/// Should not depend on other modules, unless some very special cases.
mod models;

/// Contains rendering code to render things to the terminal.
///
/// Usually depend on `components` and `models`.
mod widgets;

/// Helpful functions, mostly related to errors.
mod util;

use tui::Terminal;
use tui::backend::MouseBackend;
use failure::{Error, Fail, ResultExt};

pub type TerminalBackend = Terminal<MouseBackend>;

fn main() {
    dotenv::dotenv().ok();
    let mut terminal = match MouseBackend::new().and_then(|backend| Terminal::new(backend)) {
        Ok(val) => val,
        Err(error) => {
            util::print_error_and_exit(error.context("Cannot set up terminal backend").into())
        }
    };

    match main_with_result(&mut terminal) {
        Ok(_) => {}
        Err(error) => {
            let _ = terminal.show_cursor();
            let _ = terminal.clear();
            drop(terminal);
            util::print_error_and_exit(error.into());
        }
    }
}

fn main_with_result(terminal: &mut TerminalBackend) -> Result<(), Error> {
    let slack_api_token = ::std::env::var("SLACK_API_TOKEN")
        .context("Could not read SLACK_API_TOKEN environment variable")?;

    let rtm = slack::RtmClient::login(&slack_api_token).context("Could not log in to Slack")?;
    let app_state = data::build_app_state(rtm.start_response())?;
    let loader = data::loader::Loader::create(&slack_api_token)?;
    let selected_channel_id = app_state.selected_channel_id.clone();

    let mut app = components::App::new(app_state, loader, terminal.size()?);

    // Start to pre-load some history to get time-to-initial-render down.
    app.async_load_channel_history(&selected_channel_id)?;

    // Let app take over terminal and start main event loops.
    terminal.clear()?;
    terminal.hide_cursor()?;
    let result = components::event_loop::run(&mut app, rtm, terminal);

    terminal.show_cursor().ok();
    terminal.clear().ok();
    result
}
