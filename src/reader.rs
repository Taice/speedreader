mod settings;

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use settings::{Settings, SettingsMenu, text_to_sv};
use std::{io, thread::sleep, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Stylize},
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};

pub struct Reader {
    exit: bool,
    paused: bool,

    wait: Duration,
    settings: Settings,
}

impl Default for Reader {
    fn default() -> Self {
        if let Some(val) = Settings::load() {
            return Self {
                exit: false,
                paused: true,

                wait: Duration::from_millis(calc_wait(val.wpm)),
                settings: val,
            };
        }
        let mut ctx = ClipboardContext::new().unwrap();
        let content = ctx.get_contents();
        if let Ok(cb) = &content {
            Reader::new(400, cb, false)
        } else {
            Reader::new(
                400,
                "the skibidi toilets attacked the quick brown foxes and they lived to regret it after thre foxes attacked them in a fierce battle of wits where the skibidi rizlas from da hood with tonya from gta v got defeated like the australians in the great emu war",
                false,
            )
        }
    }
}

impl Reader {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            if self.handle_input()? {
                self.paused = true;
                self.open_settings(terminal)?;
                self.update_wpm();
                continue;
            }
            if !self.paused {
                self.update_word();
                sleep(self.wait)
            }
        }

        Ok(())
    }

    pub fn new(wpm: u32, text: &str, should_loop: bool) -> Self {
        if wpm == 0 {
            panic!("Wpm is 0");
        }
        let wait = calc_wait(wpm);
        if wait < 5 {
            panic!("idiotic speed {}", wpm);
        }

        let words = text_to_sv(text);
        Self {
            paused: true,
            wait: Duration::from_millis(wait - 1),
            settings: Settings {
                wpm,
                word_index: 0,
                words,
                should_loop,
            },
            exit: false,
        }
    }

    fn open_settings(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut menu = SettingsMenu::new(&self.settings);
        self.settings = menu.run_menu(terminal)?;
        Ok(())
    }

    fn handle_input(&mut self) -> io::Result<bool> {
        if event::poll(Duration::from_millis(1))? {
            let event = event::read()?;
            if let event::Event::Key(key) = event {
                match key.code {
                    KeyCode::Esc => return Ok(true),
                    KeyCode::Char(' ') => {
                        if self.paused && self.settings.word_index == self.settings.words.len() - 1
                        {
                            self.settings.word_index = 0;
                        }
                        self.paused = !self.paused;
                    }
                    KeyCode::Char('q') => self.quit(),
                    KeyCode::Char('s') => self.settings.save(),
                    KeyCode::Char('b') => self.back(),
                    KeyCode::Char('B') => self.back_sentence(),
                    KeyCode::Enter => self.words_from_cb(),
                    _ => (),
                }
            }
        }
        Ok(false)
    }

    fn back(&mut self) {
        self.settings.word_index = self.settings.word_index.saturating_sub(1);
        self.paused = true;
    }

    fn back_sentence(&mut self) {
        let index = self.settings.word_index;
        self.settings.word_index = 0;
        for x in (0..index.saturating_sub(1)).rev() {
            if self.settings.words[x].contains(".") {
                self.settings.word_index = x + 1;
            }
        }
        self.paused = true;
    }

    fn curr_word(&self) -> &str {
        &self.settings.words[self.settings.word_index]
    }

    fn update_word(&mut self) {
        if self.settings.word_index + 1 < self.settings.words.len() {
            self.settings.word_index += 1;
        } else {
            if !self.settings.should_loop {
                self.paused = true;
            }
            self.settings.word_index = 0;
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn update_wpm(&mut self) {
        self.wait = Duration::from_millis(calc_wait(self.settings.wpm));
    }

    fn quit(&mut self) {
        self.exit = true;
        self.settings.save();
    }

    fn words_from_cb(&mut self) {
        self.settings.get_text_cb();
    }
}

impl Widget for &Reader {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let hort = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(0),   // Space on the left
                Constraint::Percentage(100), // Centered widget area
                Constraint::Min(0),          // Space on the right
            ])
            .split(area);
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(49),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(hort[1]);

        let current = Block::bordered()
            .border_set(border::DOUBLE)
            .title(" Speed reader ")
            .title_style(Color::White)
            .title_alignment(Alignment::Center)
            .border_style(Color::White);
        current.render(hort[1], buf);

        let word = self.curr_word();
        Paragraph::new(format!("wpm: {}", self.settings.wpm))
            .italic()
            .centered()
            .render(vert[3], buf);
        Paragraph::new(word).centered().render(vert[1], buf);
        if self.paused {
            Paragraph::new("Paused")
                .italic()
                .centered()
                .render(vert[4], buf);
        }
    }
}

fn calc_wait(wpm: u32) -> u64 {
    1000u64 / (wpm as f32 / 60.0).ceil() as u64
}
