use super::App;
use super::TerminalBackend;
use widgets::ChatHistory;

use tui::widgets::{Block, Borders, Paragraph, SelectableList, Widget};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};

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

fn render_sidebar(_app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    SelectableList::default()
        .block(Block::default().title("Channels").borders(Borders::RIGHT))
        .items(&["#env-production", "#random", "#api-v3", "#team-core"])
        .select(1)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .modifier(Modifier::Italic)
                .modifier(Modifier::Invert),
        )
        .render(terminal, rect)
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

fn render_breadcrumbs(_app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    Paragraph::default()
        .text("Hemnet > #random [Talk about anything]")
        .style(Style::default().bg(Color::Gray).fg(Color::White))
        .render(terminal, rect);
}

fn render_history(app: &App, terminal: &mut TerminalBackend, rect: &Rect) {
    if rect.width < 2 {
        return;
    }

    // Leave one width for scrollbar
    let canvas = app.messages.render_into_canvas((rect.width - 1) as usize);

    ChatHistory::default()
        .scroll(app.history_scroll)
        .canvas(&canvas)
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
