mod reader;

use std::io;

use reader::Reader;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = Reader::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
