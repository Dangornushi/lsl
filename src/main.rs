use crossterm::{
    cursor::{self, Hide, MoveTo, MoveToColumn, MoveToRow, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, SetBackgroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, size, window_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen, WindowSize,
    },
};
use std::env;
use std::fs;
use std::io::Result;
use std::path::Path;
use std::process::Command;

struct App {
    mostbig_size_filename: String,
    mostbig_size_length: u16,
    in_dir_files: Vec<String>,
    cursor: (u16, u16),
    window_width: u16,
    window_height: u16,
    focus_index: usize,
}

impl App {
    fn new(
        mostbig_size_filename: String,
        mostbig_size_length: u16,
        cursor: (u16, u16),
        in_dir_files: Vec<String>,
        mut window_width: u16,
        mut window_height: u16,
        focus_index: usize,
    ) -> Self {
        window_width = size().unwrap().0;
        window_height = size().unwrap().1;
        Self {
            mostbig_size_filename,
            mostbig_size_length,
            in_dir_files,
            cursor,
            window_width,
            window_height,
            focus_index,
        }
    }

    fn get_in_dir(&mut self) -> Result<()> {
        self.in_dir_files.clear();
        match fs::read_dir("./") {
            Ok(entries) => {
                for entry in entries {
                    let filename = entry.unwrap().file_name().to_string_lossy().to_string();
                    self.in_dir_files.push(filename.clone());
                    if self.mostbig_size_filename.len() < filename.len() {
                        self.mostbig_size_filename = filename;
                    }
                }
                return Ok(());
            }
            Err(err) => match err.kind() {
                _ => return Err(err),
            },
        };
    }

    fn render_dir_view(&mut self) {
        execute!(std::io::stderr(), MoveTo(0, 1));
        for i in 0..self.mostbig_size_filename.len() + 1 {
            execute!(
                std::io::stderr(),
                Clear(ClearType::CurrentLine),
                MoveTo(0, i.try_into().unwrap())
            );
        }
        self.focus_index = 0;
        self.get_in_dir().unwrap();
    }

    pub fn main(&mut self) -> Result<()> {
        enable_raw_mode()?;
        self.get_in_dir()?;
        execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
        loop {
            execute!(std::io::stderr(), MoveTo(0, 0))?;
            execute!(std::io::stderr(), Print("┌"))?;

            for i in 1..self.window_width - 1 {
                execute!(std::io::stderr(), Print("─"))?;
            }
            execute!(std::io::stderr(), Print("┐"))?;
            execute!(std::io::stderr(), Print(" "))?;
            self.cursor.1 = 1;
            execute!(std::io::stderr(), Print("\n"))?;

            for i in 0..self.in_dir_files.len() {
                execute!(std::io::stderr(), Print("│ "))?;

                if i == self.focus_index + 1 {
                    execute!(std::io::stderr(), Print("> "))?;
                } else {
                    execute!(std::io::stderr(), Print("  "))?;
                }

                execute!(std::io::stderr(), Print(self.in_dir_files[i].clone()),)?;

                execute!(
                    std::io::stderr(),
                    MoveToColumn((self.mostbig_size_filename.len() + 5).try_into().unwrap()),
                    Print(" "),
                    Print(
                        std::fs::metadata(self.in_dir_files[i].clone())
                            .unwrap()
                            .len()
                    ),
                    Print(" B"),
                )?;
                /*
                            x += 4 + std::fs::metadata(in_dir_data[i].clone())
                                .unwrap()
                                .len()
                                .to_string()
                                .len();
                */
                self.cursor.0 = self.window_width - 1;
                execute!(
                    std::io::stderr(),
                    MoveToColumn((self.cursor.0).try_into().unwrap()),
                    Print("│\n"),
                    MoveTo(0, (i as u16 + self.cursor.1).try_into().unwrap())
                )?;
            }
            self.cursor.1 += self.in_dir_files.len() as u16 - 1;
            execute!(std::io::stderr(), MoveToRow(self.cursor.1),)?;

            // 何も表示することがない場合の空欄
            for i in self.cursor.1..self.window_height - 1 {
                execute!(std::io::stderr(), Print("│"))?;
                for i in 1..self.window_width - 1 {
                    execute!(std::io::stderr(), Print(" "))?;
                }
                execute!(std::io::stderr(), Print("│"))?;
            }

            // 下の枠
            execute!(std::io::stderr(), Print("└"))?;
            for i in 1..self.window_width - 1 {
                execute!(std::io::stderr(), Print("─"))?;
            }
            execute!(std::io::stderr(), Print("┘"))?;
            // -------------------------

            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    let mv_to = format!("{}/../", env::current_dir().unwrap().to_str().unwrap());
                    let _ = env::set_current_dir(&Path::new(&mv_to));

                    // pathに指されているものがディレクトリである
                    // cd ../

                    self.render_dir_view();
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    let mv_to = format!(
                        "{}/{}",
                        env::current_dir().unwrap().to_str().unwrap(),
                        self.in_dir_files[self.focus_index + 1].clone()
                    );

                    let root = Path::new(&mv_to);

                    match env::set_current_dir(&root) {
                        Ok(_) => {
                            // pathに指されているものがディレクトリである
                            // in_dir_dataをpathの内容に上書き

                            self.render_dir_view();
                        }
                        Err(_) => {
                            // pathに指されているものがファイルである
                            // enter keyを押した時にファイルであればvimを起動
                            execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
                            let mut child = Command::new("nvim")
                                .arg(self.in_dir_files[self.focus_index + 1].clone())
                                .spawn()
                                .unwrap();
                            child.wait().unwrap();
                            execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
                        }
                    };
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                }) => {
                    if self.focus_index < self.in_dir_files.len() - 2 {
                        self.focus_index += 1
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                }) => {
                    if self.focus_index > 0 {
                        self.focus_index -= 1
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => {
                    break;
                }

                _ => continue,
            }
            self.cursor.0 = 0;
            self.cursor.0 = 1;
        }
        execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        return Ok(());
    }
}

fn main() -> Result<()> {
    let mut app = App::new(String::from(""), 0, (0, 0), Vec::new(), 0, 0, 0);
    app.main()
}
