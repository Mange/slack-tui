use tui::widgets::*;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::*;

use TerminalBackend;
use models::{AppState, Mode};
use widgets::{self, ChatHistory};
use components::App;

pub fn render(app: &App, terminal: &mut TerminalBackend, size: &Rect) {
    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Percent(20), Size::Percent(80)])
        .render(terminal, size, |terminal, chunks| {
            render_sidebar(app.state(), terminal, &chunks[0]);
            render_main(app.state(), terminal, &chunks[1]);
        });

    if app.state().current_mode() == &Mode::SelectChannel {
        let mut selector_rect = size.clone();
        // Pick the largest out of 50% and X cells in both directions, but also cap it to display
        // size if it's smaller than the intended minimum.
        selector_rect.width = (size.width / 2).max(40).min(size.width);
        selector_rect.height = (size.height / 2).max(20).min(size.height);
        // Center selector in the middle of parent size
        selector_rect.x = (size.x + size.width / 2) - (selector_rect.width / 2);
        selector_rect.y = (size.y + size.height / 2) - (selector_rect.height / 2);

        render_channel_selector(&app, terminal, &selector_rect);
    }
}

fn render_sidebar(state: &AppState, terminal: &mut TerminalBackend, rect: &Rect) {
    let mut block = Block::default().borders(Borders::RIGHT);
    block.render(terminal, rect);

    widgets::ChannelList::new(&state.channels, state.selected_channel_id())
        .render(terminal, &block.inner(rect));
}

fn render_main(state: &AppState, terminal: &mut TerminalBackend, rect: &Rect) {
    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[
            Size::Fixed(1),
            Size::Min(10),
            Size::Fixed(1),
            Size::Fixed(1),
        ])
        .render(terminal, rect, |terminal, chunks| {
            render_breadcrumbs(state, terminal, &chunks[0]);
            render_history(state, terminal, &chunks[1]);
            render_statusbar(state, terminal, &chunks[2]);
            render_input(state, terminal, &chunks[3]);
        });
}

fn render_breadcrumbs(state: &AppState, terminal: &mut TerminalBackend, rect: &Rect) {
    match state.selected_channel() {
        Some(channel) => {
            let topic = match channel.topic_text() {
                Some(text) => format!("{{fg=white {}}}", text),
                None => String::from("{fg=gray No channel topic}"),
            };
            Paragraph::default()
                .text(&format!(
                    "{{mod=bold {team}}} > {{mod=bold #{channel}}} [{topic}]",
                    team = state.team_name,
                    channel = channel.name(),
                    topic = topic
                ))
                .style(Style::default().bg(Color::Gray).fg(Color::White))
                .render(terminal, rect);
        }
        None => {
            Paragraph::default()
                .text(&format!("{} > (No channel selected)", state.team_name))
                .style(Style::default().bg(Color::Gray).fg(Color::White))
                .render(terminal, rect);
        }
    }
}

fn render_history(state: &AppState, terminal: &mut TerminalBackend, rect: &Rect) {
    if rect.width < 2 {
        return;
    }

    // Leave one width for scrollbar
    let canvas = state.rendered_chat_canvas(rect.width - 1, rect.height);

    ChatHistory::with_canvas(&canvas)
        .scroll(state.current_history_scroll())
        .render(terminal, rect);
}

fn render_statusbar(state: &AppState, terminal: &mut TerminalBackend, rect: &Rect) {
    let (mode, mode_color) = match state.current_mode {
        Mode::History => ("HISTORY", "bg=cyan;fg=black"),
        Mode::SelectChannel => ("CHANNELS", "bg=black;fg=white"),
    };
    Paragraph::default()
        .text(&format!(
            "{{{mode_color} {mode}}} - [{offset}/{height}]",
            mode = mode,
            mode_color = mode_color,
            offset = state.history_scroll,
            height = state.max_history_scroll(),
        ))
        .style(Style::default().bg(Color::Gray).fg(Color::White))
        .render(terminal, rect);
}

fn render_input(_state: &AppState, terminal: &mut TerminalBackend, rect: &Rect) {
    Paragraph::default()
        .text("{fg=dark_gray Enter a reply...}")
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .render(terminal, rect);
}

fn render_channel_selector(app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    if rect.width <= 5 || rect.height <= 5 {
        return;
    }

    let black_on_gray = Style::default().bg(Color::Gray).fg(Color::Black);
    let white_on_black = Style::default().bg(Color::Black).fg(Color::White);

    Block::default()
        .title("Select channel")
        .borders(Borders::ALL)
        .style(black_on_gray)
        .border_style(black_on_gray)
        .title_style(black_on_gray)
        .render(terminal, rect);

    let input_rect = Rect::new(rect.left() + 1, rect.top() + 1, rect.width - 2, 1);
    widgets::LineEdit::default()
        .style(white_on_black)
        .text(app.channel_selector.text())
        .cursor_pos(app.channel_selector.cursor_pos())
        .render(terminal, &input_rect);

    let list_rect = Rect::new(
        rect.left() + 1,
        rect.top() + 3,
        rect.width - 2,
        rect.height - 4,
    );

    // SelectableList does not render background style.
    // https://github.com/fdehau/tui-rs/issues/42
    //
    // Pad all items with spaces to achieve the same effect.
    let matches: Vec<String> = app.channel_selector
        .top_matches(&app.state().channels, list_rect.height as usize)
        .into_iter()
        .map(|m| format!("#{:<1$}", m.channel.name(), list_rect.width as usize))
        .collect();

    SelectableList::default()
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(black_on_gray)
                .style(black_on_gray),
        )
        .style(black_on_gray)
        .highlight_style(white_on_black)
        .items(&matches)
        .select(app.channel_selector.selected_index(matches.len()))
        .render(terminal, &list_rect);
}
