use std::io;

mod app;
mod host;
mod ui;

use app::App;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}