use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

use crate::host::Entry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Finder,
    Details,
}

#[derive(Debug)]
pub struct App {
    pub entries: Vec<Entry>,
    pub selected: usize,
    pub query: String,
    pub connect_target: Option<String>,
    pub mode: Mode,
    pub exit: bool,
}


#[derive(Debug)]
pub struct VisibleEntry<'a> {
    pub depth: usize,
    pub entry: &'a Entry,
}

impl Default for App {
    fn default() -> Self {
        Self {
            entries: crate::host::collect_hosts().unwrap(),
            selected: 0,
            connect_target: None,
            query: String::new(),
            mode: Mode::Normal,
            exit: false,
        }
    }
}

fn fuzzy_match(query: &str, text: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let mut query_chars = query.chars().map(|c| c.to_ascii_lowercase());

    let Some(mut current) = query_chars.next() else {
        return true;
    };

    for c in text.chars().map(|c| c.to_ascii_lowercase()) {
        if c == current {
            match query_chars.next() {
                Some(next) => current = next,
                None => return true,
            }
        }
    }
    false
}

fn entry_macthes_query(entry: &Entry, query: &str) -> bool {
    match entry {
        Entry::Host(host) => {
            fuzzy_match(query, &host.name)
                || fuzzy_match(query, &host.target)
                || fuzzy_match(query, &host.description)
        }
        Entry::Folder(folder) => fuzzy_match(query, &folder.name),
    }
}

impl App {

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<Option<String>> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(self.connect_target.clone())
    }

    fn connect_selected(&mut self) {
        let Some(entry) = self.selected_entry() else {
            return;
        };

        let Entry::Host(host) = entry else {
            return;
        };

        self.connect_target = Some(host.name.clone());
        self.exit = true;
    }

    fn draw(&self, frame: &mut Frame) {
        crate::ui::draw(self, frame);
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
        match self.mode {
            Mode::Normal => self.handle_normal_key_event(key_event),
            Mode::Finder => self.handle_finder_key_event(key_event),
            Mode::Details => self.handle_details_key_event(key_event),
        }
    }

    fn handle_normal_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_entry(),
            KeyCode::Down | KeyCode::Char('j') => self.next_entry(),
            KeyCode::Enter => self.mode = Mode::Details,
            
            KeyCode::Char(' ') => {}
            KeyCode::Char('f') => self.mode = Mode::Finder,
            KeyCode::Char(c) => {
                self.mode = Mode::Finder;
                self.query.push(c);
            },
            _ => {}
        }
    }

    fn handle_details_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Enter => {
                self.connect_selected();
            },
            KeyCode::Esc => {
                self.query.clear();
                self.mode = Mode::Normal;
            }
            
            KeyCode::Char(' ') => {}
            KeyCode::Char('f') => self.mode = Mode::Finder,
            KeyCode::Char(c) => {
                self.mode = Mode::Finder;
                self.query.push(c);
            },
            _ => {}
        }
    }

    fn handle_finder_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Backspace => {
                self.query.pop();
            }
            KeyCode::Enter => self.mode = Mode::Normal,

            KeyCode::Esc => {
                self.query.clear();
                self.mode = Mode::Normal;
            }
            KeyCode::Char(' ') => {}
            KeyCode::Char(c) => self.query.push(c),
            _ => {}
        }
    }
    
    fn exit(&mut self) {
        self.exit = true;
    }

    fn next_entry(&mut self) {
        let visible_len = self.visible_entries().len();

        if visible_len == 0 {
            return;
        }

        self.selected = (self.selected + 1) % visible_len;
    }

    fn previous_entry(&mut self) {
        let visible_len = self.visible_entries().len();

        if visible_len == 0 {
            return;
        }

        self.selected = self.selected.checked_sub(1).unwrap_or(visible_len - 1);
    }

    pub fn visible_entries(&self) -> Vec<VisibleEntry<'_>> {
        fn walk<'a>(
            entries: &'a [Entry],
            depth: usize,
            query: &str,
            out: &mut Vec<VisibleEntry<'a>>,
        ) {
            for entry in entries {
                if query.is_empty() || entry_macthes_query(entry, query) {
                    out.push(VisibleEntry { depth, entry });
                }

                if let Entry::Folder(folder) = entry {
                    walk(&folder.children, depth + 1, query, out);
                }
            }
        }

        let mut out = Vec::new();
        walk(&self.entries, 0, &self.query, &mut out);
        out
    }

    pub fn selected_entry(&self) -> Option<&Entry> {
        let visible = self.visible_entries();
        visible.get(self.selected).map(|entry| entry.entry)
    }
}