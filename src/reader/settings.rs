use cli_clipboard::{ClipboardContext, ClipboardProvider};
use directories::BaseDirs;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    time::Duration,
};

/// 1-wpm 2-should-loop 3-words
const NUM_ELEMENTS: u8 = 3;

#[derive(Default, Clone)]
pub struct Settings {
    pub wpm: u32,
    pub word_index: usize,
    pub words: Vec<String>,
    pub should_loop: bool,
}

impl Settings {
    /// loads from {data-dir}/settings.json
    pub fn load() -> Option<Self> {
        let file = get_save_file()?;
        match std::fs::read_to_string(&file) {
            Ok(contents) => {
                let thing: serde_json::Result<FileSettings> = serde_json::from_str(&contents);
                match thing {
                    Ok(settings) => {
                        return Some(Self {
                            wpm: settings.wpm,
                            word_index: 0,
                            words: text_to_sv(&settings.words),
                            should_loop: settings.should_loop,
                        });
                    }
                    Err(err) => write_log(format!("{err}")),
                }
            }
            Err(err) => write_log(format!("{err}")),
        }
        None
    }

    pub fn save(&mut self) {
        if let Some(file) = get_save_file() {
            let filesettings = FileSettings {
                wpm: self.wpm,
                should_loop: self.should_loop,
                words: self.words.join(" "),
            };

            if let Ok(json) = serde_json::to_string(&filesettings) {
                let _ = fs::create_dir_all(get_dir().unwrap());
                let _ = std::fs::write(file, json);
            }
        }
    }

    pub fn get_text_cb(&mut self) {
        let mut ctx = ClipboardContext::new().unwrap();
        let content = ctx.get_contents();
        if let Ok(val) = content {
            self.words.clear();
            self.words = text_to_sv(&val);
            self.word_index = 0;
        }
    }
}

#[derive(Serialize, Deserialize)]
struct FileSettings {
    pub wpm: u32,
    pub should_loop: bool,
    pub words: String,
}

pub struct SettingsMenu {
    settings: Settings,
    index: u8,
    view_words: bool,
}

impl SettingsMenu {
    pub fn new(settings: &Settings) -> Self {
        Self {
            settings: settings.clone(),
            index: 0,
            view_words: false,
        }
    }

    pub fn run_menu(&mut self, terminal: &mut DefaultTerminal) -> io::Result<Settings> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if self.handle_input()? {
                break;
            }
        }

        Ok(self.settings.clone())
    }

    fn handle_input(&mut self) -> io::Result<bool> {
        if event::poll(Duration::from_millis(1))? {
            let event = event::read()?;
            if let event::Event::Key(key) = event {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        if self.view_words {
                            self.view_words = false
                        } else {
                            return Ok(true);
                        }
                    }
                    KeyCode::Up => self.up(),
                    KeyCode::Down => self.down(),
                    KeyCode::Right => self.increase(),
                    KeyCode::Left => self.decrease(),
                    KeyCode::Char(' ') => match self.index {
                        1 => self.settings.should_loop = !self.settings.should_loop,
                        2 => self.get_text_cb(),
                        _ => (),
                    },
                    KeyCode::Enter => {
                        if self.index == 2 {
                            self.view_words = true;
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(false)
    }

    fn get_text_cb(&mut self) {
        let mut ctx = ClipboardContext::new().unwrap();
        let content = ctx.get_contents();
        if let Ok(val) = content {
            self.settings.words.clear();
            self.settings.words = text_to_sv(&val);
            self.settings.word_index = 0;
        }
    }

    fn increase(&mut self) {
        if self.view_words {
            if self.settings.word_index < self.settings.words.len() - 1 {
                self.settings.word_index += 1;
            }
            return;
        }
        match self.index {
            0 => self.settings.wpm += 10,
            1 => self.settings.should_loop = !self.settings.should_loop,
            _ => (),
        }
    }

    fn decrease(&mut self) {
        if self.view_words {
            if self.settings.word_index > 0 {
                self.settings.word_index -= 1;
            }
            return;
        }
        match self.index {
            0 => {
                if self.settings.wpm > 10 {
                    self.settings.wpm -= 10;
                }
            }
            1 => self.settings.should_loop = !self.settings.should_loop,
            _ => (),
        }
    }

    fn up(&mut self) {
        if !self.view_words && self.index > 0 {
            self.index -= 1;
        }
    }

    fn down(&mut self) {
        if !self.view_words && self.index < NUM_ELEMENTS - 1 {
            self.index += 1;
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &SettingsMenu {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.view_words {
            let mut vec: Vec<Span> = vec![];

            for (i, x) in self.settings.words.iter().enumerate() {
                let string = x.to_owned() + " ";
                if i == self.settings.word_index {
                    vec.push(Span::styled(x, Style::default().bg(Color::DarkGray)));
                    vec.push(Span::raw(" "));
                } else {
                    vec.push(Span::raw(string));
                };
            }

            let line = Line::from(vec);

            Paragraph::new(line)
                .wrap(Wrap { trim: true })
                .block(
                    Block::bordered()
                        .border_set(border::DOUBLE)
                        .title("words")
                        .title_alignment(Alignment::Center)
                        .italic(),
                )
                .render(area, buf);
        } else {
            let hort = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30), // Space on the left
                    Constraint::Percentage(40), // Centered widget area
                    Constraint::Percentage(30), // Space on the right
                ])
                .split(area);
            let vert = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(hort[1]);

            Block::bordered()
                .border_set(border::DOUBLE)
                .title(" Settings ")
                .title_style(Color::White)
                .title_alignment(Alignment::Center)
                .italic()
                .border_style(Color::Gray)
                .render(area, buf);

            Paragraph::new(format!("Wpm: {}", self.settings.wpm))
                .block(Block::bordered().border_set(border::PLAIN).border_style(
                    if self.index == 0 {
                        Color::Green
                    } else {
                        Color::White
                    },
                ))
                .render(vert[1], buf);
            Paragraph::new(format!("Should loop: {}", self.settings.should_loop))
                .block(Block::bordered().border_set(border::PLAIN).border_style(
                    if self.index == 1 {
                        Color::Green
                    } else {
                        Color::White
                    },
                ))
                .render(vert[2], buf);
            Paragraph::new("Words (space to paste)")
                .block(Block::bordered().border_set(border::PLAIN).border_style(
                    if self.index == 2 {
                        Color::Green
                    } else {
                        Color::White
                    },
                ))
                .render(vert[3], buf);
        }
    }
}

pub fn text_to_sv(text: &str) -> Vec<String> {
    let mut vec = vec![];
    text.split(" ").for_each(|x| {
        if !x.is_empty() {
            vec.push(x.trim().to_owned())
        }
    });
    vec
}

fn get_save_file() -> Option<String> {
    let dirs = BaseDirs::new()?;
    let dir = dirs.data_dir();
    Some(dir.join("SpeedReader/settings.json").to_str()?.to_owned())
}

fn get_dir() -> Option<String> {
    let dirs = BaseDirs::new()?;
    let dir = dirs.data_dir();
    Some(dir.join("SpeedReader").to_str()?.to_owned())
}

fn get_log_file() -> Option<String> {
    let dirs = BaseDirs::new()?;
    let dir = dirs.data_dir();
    Some(dir.join("SpeedReader/logs.txt").to_str()?.to_owned())
}

fn write_log(log: String) {
    if let Some(file) = get_log_file() {
        if let Ok(data_file) = &mut OpenOptions::new().append(true).create(true).open(file) {
            let _ = data_file.write((log + "\n").as_bytes());
        }
    }
}
