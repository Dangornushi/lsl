// ANCHOR: imports
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use nix::{
    sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal},
    sys::wait::waitpid,
    unistd::{execvp, fork, getpgrp, pipe, read, setpgid, tcsetpgrp, ForkResult},
};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use std::process::exit;
use std::{
    ffi::CString,
    io::{stdin, stdout, Write},
};

use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
// ANCHOR_END: imports

// ANCHOR: modules
mod tui;
// ANCHOR_END: modules

// ANCHOR: main
fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);
    tui::restore()?;
    app_result
}
// ANCHOR_END: main

// ANCHOR: app
#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    in_dir_data: Vec<String>,
    path_log: Vec<String>,
    view_filedata: bool,
    exit: bool,
    screan_mode: usize,
}
// ANCHOR_END: app

// ANCHOR: impl App
impl App {
    // ANCHOR: run
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        // カレントディレクトリの内容を取得
        self.get_in_dir();
        // メンバ変数の初期化
        self.view_filedata = false;
        self.screan_mode = 0;

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;

            self.handle_events()?;
            self.screan_mode = 0;
        }
        Ok(())
    }
    // ANCHOR_END: run

    // ANCHOR: render_frame
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }
    // ANCHOR_END: render_frame

    // ANCHOR: handle_events
    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    // ANCHOR_END: handle_events

    // ANCHOR: handle_key_event
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('k') => self.decrement_counter(),
            KeyCode::Char('j') => self.increment_counter(),
            KeyCode::Enter => self.handle_enter_key(),
            KeyCode::Char(' ') => {
                self.view_filedata = if self.view_filedata == true {
                    false
                } else {
                    true
                }
            }
            KeyCode::Esc => self.handle_esc_key(),
            _ => {}
        }
    }
    // ANCHOR_END: handle_key_event

    fn handle_esc_key(&mut self) {
        // self.path_logの最後から二番目の要素に移動すれば良い
        // mv to PWD/../

        let mv_to = format!("{}/../", env::current_dir().unwrap().to_str().unwrap());

        let root = Path::new(&mv_to);

        match env::set_current_dir(&root) {
            Ok(_) => {
                // pathに指されているものがディレクトリである
                self.get_in_dir();
            }
            Err(_) => {
                // pathに指されているものがファイルである
                todo!();
            }
        };
    }

    fn handle_enter_key(&mut self) {
        let mv_to = format!(
            "{}/{}",
            env::current_dir().unwrap().to_str().unwrap(),
            self.in_dir_data[self.counter as usize].clone()
        );

        let root = Path::new(&mv_to);

        match env::set_current_dir(&root) {
            Ok(_) => {
                // pathに指されているものがディレクトリである
                self.get_in_dir();
            }
            Err(_) => {
                // pathに指されているものがファイルである

                self.screan_mode = 1;
                let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
                self.render(buf.area, &mut buf);
            }
        };
    }

    // ANCHOR: methods

    fn get_in_dir(&mut self) {
        // ターミナル表示用のディレクトリ一覧配列をリセット
        self.in_dir_data.clear();
        // カーソルキーのインデックスもリセット
        self.counter = 0;
        let entries = match fs::read_dir("./") {
            Ok(entries) => entries,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => return println!("ディレクトリが見つかりません"),
                _ => return println!("エラーが発生しました: {}", err),
            },
        };

        // 各エントリをループ処理
        for entry in entries {
            let file_name = entry.unwrap().file_name();
            self.in_dir_data
                .push(file_name.to_string_lossy().to_string());
        }
        /*
         */
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        if self.in_dir_data.len() - 1 > self.counter as usize {
            self.counter += 1;
        }
    }

    fn decrement_counter(&mut self) {
        if 0 < self.counter as usize {
            self.counter -= 1;
        }
    }
    // ANCHOR_END: methods
    // ANCHOR_END: impl App
}

// ANCHOR: impl Widget
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut dir_data_s = vec![];

        if self.view_filedata {
            // ファイルの内容を表示
            let path = format!(
                "{}/{}",
                env::current_dir().unwrap().to_str().unwrap(),
                self.in_dir_data[self.counter as usize].clone()
            );
            for result in BufReader::new(File::open(path).unwrap()).lines() {
                let l = result.unwrap();

                dir_data_s.push(Line::from(vec![l.white()]));
            }
        } else {
            if self.screan_mode == 1 {
                let mut child = Command::new("nvim").arg("./").spawn().unwrap();
                child.wait().unwrap();
            }

            // ディレクトリの一覧
            for i in 0..self.in_dir_data.len() {
                if i == self.counter as usize {
                    dir_data_s.push(Line::from(vec![
                        "> ".into(),
                        self.in_dir_data.get(i).unwrap().to_string().blue(),
                    ]));
                } else {
                    dir_data_s.push(Line::from(vec![
                        "  ".into(),
                        self.in_dir_data.get(i).unwrap().to_string().white(),
                    ]));
                }
            }
        }

        let title = Title::from(" lsl ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
            " counter ".into(),
            self.counter.to_string().blue(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        Paragraph::new(Text::from(dir_data_s))
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
// ANCHOR_END: impl Widget

// ANCHOR: tests
#[cfg(test)]
mod tests {
    // ANCHOR: render test
    use super::*;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }
    // ANCHOR_END: render test

    // ANCHOR: handle_key_event test
    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.exit, true);

        Ok(())
    }
    // ANCHOR_END: handle_key_event test
}
// ANCHOR_END: tests
