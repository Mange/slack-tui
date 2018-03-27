use failure::Error;
use slack::api::rtm::StartResponse;
use std::cell::{Cell, RefCell};

use models::{AppState, Channel, ChannelList, MessageBuffer, Mode, User, UserList};

pub fn build_app_state(response: &StartResponse) -> Result<AppState, Error> {
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
    let selected_channel_id = match selected_channel_id {
        Some(val) => val,
        None => return Err(format_err!("Could not find any channels in the Team")),
    };

    let team_name = response
        .team
        .as_ref()
        .and_then(|team| team.name.as_ref())
        .cloned()
        .ok_or_else(|| format_err!("Slack did not provide a Team Name on login"))?;

    Ok(AppState {
        current_mode: Mode::History,

        chat_canvas: RefCell::new(None),
        history_scroll: 0,
        last_chat_height: Cell::new(0),

        selected_channel_id,
        channels,

        is_loading_more_messages: false,
        messages: MessageBuffer::new(),

        team_name,
        users,
    })
}
