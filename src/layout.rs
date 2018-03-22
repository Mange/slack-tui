use super::App;
use super::TerminalBackend;
use widgets::{self, ChatHistory};

use tui::widgets::{Block, Borders, Paragraph, Widget};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::*;

pub fn render(app: &App, terminal: &mut TerminalBackend) {
    let size = &app.size;
    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Percent(20), Size::Percent(80)])
        .render(terminal, size, |terminal, chunks| {
            render_sidebar(app, terminal, &chunks[0]);
            render_main(app, terminal, &chunks[1]);
        });
}

fn render_sidebar(app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    let mut block = Block::default().borders(Borders::RIGHT);
    block.render(terminal, rect);

    widgets::ChannelList::new(&app.channels, app.selected_channel_id.as_ref())
        .render(terminal, &block.inner(rect));
}

fn render_main(app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[
            Size::Fixed(1),
            Size::Min(10),
            Size::Fixed(1),
            Size::Fixed(1),
        ])
        .render(terminal, rect, |terminal, chunks| {
            render_breadcrumbs(app, terminal, &chunks[0]);
            render_history(app, terminal, &chunks[1]);
            render_statusbar(app, terminal, &chunks[2]);
            render_input(app, terminal, &chunks[3]);
        })
}

fn render_breadcrumbs(app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    let team_name = "My Team";
    match app.selected_channel() {
        Some(channel) => {
            let topic = match channel.topic_text() {
                Some(text) => format!("{{fg=white {}}}", text),
                None => String::from("{fg=gray No channel topic}"),
            };
            Paragraph::default()
                .text(&format!(
                    "{{mod=bold {team}}} > {{mod=bold #{channel}}} [{topic}]",
                    team = team_name,
                    channel = channel.name(),
                    topic = topic
                ))
                .style(Style::default().bg(Color::Gray).fg(Color::White))
                .render(terminal, rect);
        }
        None => {
            Paragraph::default()
                .text(&format!("{} > (No channel selected)", team_name))
                .style(Style::default().bg(Color::Gray).fg(Color::White))
                .render(terminal, rect);
        }
    }
}

fn render_history(app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    if rect.width < 2 {
        return;
    }

    // Leave one width for scrollbar
    let canvas = app.rendered_chat_canvas(rect.width - 1, rect.height);

    ChatHistory::with_canvas(&canvas)
        .scroll(app.current_history_scroll())
        .render(terminal, rect);
}

fn render_statusbar(_app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    Paragraph::default()
        .text("{mod=bold [NORMAL]} - {fg=dark_gray Peter is typing...}")
        .style(Style::default().bg(Color::Gray).fg(Color::White))
        .render(terminal, rect);
}

fn render_input(_app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    Paragraph::default()
        .text("{fg=dark_gray Enter a reply...}")
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .render(terminal, rect);
}
