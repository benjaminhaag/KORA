use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

use crate::host::Host;

#[derive(Debug)]
pub struct App {
    pub hosts: Vec<Host>,
    pub selected: usize,
    pub query: String,
    pub exit: bool,
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

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
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

    pub fn selected_host(&self) -> Option<&Host> {
        self.hosts.get(self.selected)
    }
}