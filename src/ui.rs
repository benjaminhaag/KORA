use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Widget},
    Frame,
};

use crate::app::App;

use crate::host::Entry;

#[derive(Debug, Clone, Copy)]
struct AppLayout {
    search: Rect,
    hosts: Rect,
    details: Rect,
}

pub fn draw(app: &App, frame: &mut Frame) {
    let area = frame.area();
    frame.render_widget(app, area);
    
    let layout = layout(area);

    let cursor_x = layout.search.x + 1 + app.query.len() as u16;
    let cursor_y = layout.search.y + 1;

    frame.set_cursor_position((cursor_x, cursor_y));
}

fn layout(area: Rect) -> AppLayout {
    let outer_block = Block::bordered();
    let inner = outer_block.inner(area);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(inner);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_chunks[1]);

    AppLayout {
        search: main_chunks[0],
        hosts: content_chunks[0],
        details: content_chunks[1],
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let outer_block = Block::bordered()
            .title(Line::from(" KORA - KORA Opinionated Remote Access ".bold()).centered())
            .title_bottom(
                Line::from(vec![
                    " Navigate ".into(),
                    "<↑/↓>".blue().bold(),
                    " Connect ".into(),
                    "<Enter>".blue().bold(),
                    " Quit ".into(),
                    "<q> ".blue().bold(),
                ])
                .centered(),
            )
            .border_set(border::THICK);

        outer_block.render(area, buf);

        let layout = layout(area);

        
        self.render_search(layout.search, buf);
        self.render_host_list(layout.hosts, buf);
        self.render_host_details(layout.details, buf);
    }
}

impl App {

    fn highlight_matches(&self, text: &str) -> Line<'_> {
        if self.query.is_empty() {
            return Line::from(text.to_string());
        }

        let mut spans = Vec::new();
        let mut query_chars = self.query.chars().map(|c| c.to_ascii_lowercase());

        let mut current = query_chars.next();
        
        for c in text.chars() {
            if let Some(q) = current {
                if c.to_ascii_lowercase() == q {
                    spans.push(Span::styled(
                        c.to_string(),
                        Style::default().fg(Color::Yellow),
                    ));
                    current = query_chars.next();
                    continue;
                }
            }

            spans.push(Span::raw(c.to_string()));
        }

        Line::from(spans)
    }

    fn render_search(&self, area: Rect, buf: &mut Buffer) {
        let search_text = if self.query.is_empty() {
            "Search hosts...".dim()
        } else {
            self.query.clone().into()
        };
        Paragraph::new(search_text)
            .block(Block::bordered().title(" Search "))
            .render(area, buf);
    }

    fn render_host_list(&self, area: Rect, buf: &mut Buffer) {

        let visible = self.visible_entries();

        let items: Vec<ListItem> = visible
            .iter()
            .map(|visible_entry| {
                let indent = "  ".repeat(visible_entry.depth);

                let label = match visible_entry.entry {
                    Entry::Host(host) => format!("{indent} {}", host.name),
                    Entry::Folder(folder) => format!("{indent} {}", folder.name),
                };

                ListItem::new(self.highlight_matches(&label))
            })
            .collect();

        let list = List::new(items)
            .block(Block::bordered().title(" Hosts "))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White),
            );

        let mut state = ListState::default();
        state.select(Some(self.selected));

        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut state);
    }

    fn render_host_details(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Details ");
        let inner = block.inner(area);
        block.render(area, buf);

        match self.selected_entry() {
            Some(Entry::Host(host)) => {
                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(3),
                    ])
                    .split(inner);

                Paragraph::new(Line::from(vec![
                    host.name.clone().into(),
                ]))
                .block(Block::bordered().title(" Name "))
                .render(rows[0], buf);

                Paragraph::new(Line::from(vec![
                    host.target.clone().green(),
                ]))
                .block(Block::bordered().title(" Target "))
                .render(rows[1], buf);

                
                let config_lines: Vec<Line> = host
                    .config
                    .iter()
                    .map(|line| Line::from(line.clone().black()))
                    .collect();
                Paragraph::new(config_lines)
                    .block(Block::bordered().title(" Config "))
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .render(rows[2], buf);
            }
            _ => {
                Paragraph::new("No host selected")
                .red()
                .centered()
                .render(inner, buf);
                return;
            }
        };

        
    }
}