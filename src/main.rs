use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line},
    widgets::{Block, List, ListItem, ListState, Paragraph, Widget},
    DefaultTerminal, Frame,
};

#[derive(Debug)]
pub struct App {
    hosts: Vec<Host>,
    selected: usize,
    query: String,
    exit: bool,
}

#[derive(Debug, Clone)]
pub struct Host {
    name: String,
    target: String,
    description: String,
}

#[derive(Debug, Clone, Copy)]
struct AppLayout {
    search: Rect,
    hosts: Rect,
    details: Rect,
}

impl Default for App {
    fn default() -> Self {
        Self {
            hosts: vec![
                Host {
                    name: "Production".into(),
                    target: "root@prod.example.com".into(),
                    description: "Main production server".into(),
                },
                Host {
                    name: "Staging".into(),
                    target: "deploy@staging.example.com".into(),
                    description: "Pre-production deployment target".into(),
                },
                Host {
                    name: "Local VM".into(),
                    target: "user@192.168.56.10".into(),
                    description: "Local development machine".into(),
                },
            ],
            selected: 0,
            query: String::new(),
            exit: false,
        }
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
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

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(self, area);
        
        let layout = Self::layout(area);

        let cursor_x = layout.search.x + 1 + self.query.len() as u16;
        let cursor_y = layout.search.y + 1;

        frame.set_cursor_position((cursor_x, cursor_y));
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.previous_host(),
            KeyCode::Down => self.next_host(),
            
            KeyCode::Backspace => {
                self.query.pop();
            }

            KeyCode::Esc => {
                self.query.clear();
            }

            KeyCode::Char(c) => {
                self.query.push(c);
            }

            _ => {}
        }
    }
    
    fn exit(&mut self) {
        self.exit = true;
    }

    fn next_host(&mut self) {
        if self.hosts.is_empty() {
            return;
        }

        self.selected = (self.selected + 1) % self.hosts.len();
    }

    fn previous_host(&mut self) {
        if self.hosts.is_empty() {
            return;
        }

        self.selected = self.selected.checked_sub(1).unwrap_or(self.hosts.len() - 1);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let outer_block = Block::bordered()
            .title(Line::from(" KORA ".bold()).centered())
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

        let layout = App::layout(area);

        
        self.render_search(layout.search, buf);
        self.render_host_list(layout.hosts, buf);
        self.render_host_details(layout.details, buf);
    }
}

impl App {

    fn selected_host(&self) -> Option<&Host> {
        self.hosts.get(self.selected)
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
        let items: Vec<ListItem> = self
            .hosts
            .iter()
            .map(|host| ListItem::new(Line::from(host.name.clone()).centered()))
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

    let Some(host) = self.selected_host() else {
        Paragraph::new("No host selected")
            .red()
            .centered()
            .render(inner, buf);
        return;
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
        ])
        .split(inner);

    Paragraph::new(Line::from(vec![
        "Name\n".bold(),
        host.name.clone().into(),
    ]))
    .block(Block::bordered().title(" Name "))
    .render(rows[0], buf);

    Paragraph::new(Line::from(vec![
        host.target.clone().green(),
    ]))
    .block(Block::bordered().title(" Target "))
    .render(rows[1], buf);

    Paragraph::new(host.description.clone().black())
        .block(Block::bordered().title(" Description "))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(rows[2], buf);
}
}