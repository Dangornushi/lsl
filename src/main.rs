use chrono::prelude::{DateTime, Datelike, Utc};
use crossterm::{
    cursor::{self, Hide, MoveTo, MoveToColumn, MoveToRow, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, size, window_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen, WindowSize,
    },
};
use std::fs;
use std::io::Result;
use std::path::Path;
use std::process::Command;
extern crate chrono;
use std::{env, os::unix::fs::PermissionsExt};

const GRUVBOX_BACKGROUND: Color = Color::Rgb {
    r: 40,
    g: 40,
    b: 40,
};

struct App {
    mostbig_size_filename: String,
    mostbig_size_length: usize,
    mostbig_permission: usize,
    in_dir_files: Vec<String>,
    cursor: (u16, u16),
    window_width: u16,
    window_height: u16,
    focus_index: usize,
    start_w: u16,
    start_h: u16,
    exit_flag: bool,
}

impl App {
    fn new(
        mostbig_size_filename: String,
        mostbig_size_length: usize,
        cursor: (u16, u16),
        in_dir_files: Vec<String>,
        window_width: u16,
        window_height: u16,
        focus_index: usize,
        start_w: u16,
        start_h: u16,
    ) -> Self {
        Self {
            mostbig_size_filename,
            mostbig_size_length,
            mostbig_permission: 0,
            in_dir_files,
            cursor,
            window_width,
            window_height,
            focus_index,
            start_w,
            start_h,
            exit_flag: false,
        }
    }

    fn get_in_dir(&mut self) -> Result<()> {
        self.in_dir_files.clear();
        match fs::read_dir("./") {
            Ok(entries) => {
                for entry in entries {
                    let filename = entry.unwrap().file_name().to_string_lossy().to_string();
                    let filesize = std::fs::metadata(filename.clone())
                        .unwrap()
                        .len()
                        .to_string();
                    self.in_dir_files.push(filename.clone());
                    if self.mostbig_size_filename.len() < filename.len() {
                        self.mostbig_size_filename = filename.clone();
                    }

                    if self.mostbig_size_length < filesize.len() {
                        self.mostbig_size_length = filesize.len();
                    }

                    //--------------------------------------------------------

                    //--------------------------------------------------------
                }
                return Ok(());
            }
            Err(err) => match err.kind() {
                _ => return Err(err),
            },
        };
    }

    fn render_dir_view(&mut self) {
        execute!(
            std::io::stderr(),
            MoveTo(self.start_w + 0, self.start_h + 1)
        );
        for i in 0..self.mostbig_size_filename.len() + 1 {
            execute!(
                std::io::stderr(),
                Clear(ClearType::CurrentLine),
                MoveTo(0, (self.start_h + i as u16).try_into().unwrap())
            );
        }
        self.focus_index = 0;
        self.get_in_dir().unwrap();
    }

    fn draw_background(&mut self) {
        for i in 0..self.window_width {
            for j in 0..self.window_height {
                execute!(
                    std::io::stderr(),
                    MoveTo(self.start_w + i, self.start_h + j),
                    SetBackgroundColor(GRUVBOX_BACKGROUND),
                    Print(" "),
                );
            }
        }
    }

    pub fn key_read(&mut self, max_down: usize) -> Result<()> {
        // ------------------------------------------------------------------------------------
        match read()? {
            // ESC ----------------------------------------------------------------------------
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                let mv_to = format!("{}/../", env::current_dir().unwrap().to_str().unwrap());
                let _ = env::set_current_dir(&Path::new(&mv_to));

                // pathに指されているものがディレクトリである
                // cd ../

                self.render_dir_view();
            }
            // ESC ----------------------------------------------------------------------------
            // SPACE ---
            //
            /*
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                ..
            }) => {
                execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
                let mut sub_window =
                    App::new(String::from(""), 0, (0, 0), Vec::new(), 20, 10, 0, 100, 10);
                let _ = sub_window.sw();
                execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;

                //self.draw_background()
            }
             */
            // SPACE ---

            // Enter---------------------------------------------------------------------------
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                let mv_to = format!(
                    "{}/{}",
                    env::current_dir().unwrap().to_str().unwrap(),
                    self.in_dir_files[self.focus_index].clone()
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
                            .arg(self.in_dir_files[self.focus_index].clone())
                            .spawn()
                            .unwrap();
                        child.wait().unwrap();
                        execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;

                        //self.draw_background();
                    }
                };
            }
            // Enter --------------------------------------------------------------------------

            // WASD ---------------------------------------------------------------------------
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                if max_down > 1 && self.focus_index < max_down - 1 {
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
                self.exit_flag = true;
            }

            _ => {} // WASD ---------------------------------------------------------------------------
        }

        // ------------------------------------------------------------------------------------
        self.cursor.0 = 0;
        self.cursor.0 = 1;
        return Ok(());
    }

    fn shinsu(&mut self, mut x: i64, b: i64) -> i64 {
        let mut amari: Vec<i64> = Vec::new();
        while x != 0 {
            amari.push(x % b);
            x /= b;
        }
        let mut n: i64 = 0;
        for i in 0..amari.len() {
            n += 10i64.pow(i as u32) * amari.get(i).unwrap()
        }
        return n;
    }

    fn generate_permission_strings(&mut self, permission_num: i64) -> String {
        let hex_permission_num;
        let mut permission = String::new();
        if permission_num > 100000 {
            hex_permission_num = permission_num - 100000;
        } else {
            hex_permission_num = permission_num - 40000;
        }

        for hex_permission_char in hex_permission_num.to_string().chars() {
            permission = format!(
                "{}{}",
                permission,
                match hex_permission_char {
                    '0' => "---",
                    '1' => "--x",
                    '2' => "-w-",
                    '3' => "-wx",
                    '4' => "r--",
                    '5' => "r-x",
                    '6' => "rw-",
                    '7' => "rwx",
                    _ => "",
                }
            );
        }

        permission
    }

    fn generate_permission_strings_in_japanese(&mut self, permission_num: i64) -> String {
        let hex_permission_num;
        let mut permission = String::new();
        let mut counter = 0;
        if permission_num > 100000 {
            hex_permission_num = permission_num - 100000;
        } else {
            hex_permission_num = permission_num - 40000;
        }

        for hex_permission_char in hex_permission_num.to_string().chars() {
            if counter == 0 {
                permission = "所有者 > ".to_string();
                counter += 1;
            } else if counter == 1 {
                permission = format!("{}グループ > ", permission);
                counter += 1;
            } else if counter == 2 {
                permission = format!("{}その他 > ", permission);
            }
            permission = format!(
                "{}{} │ ",
                permission,
                match hex_permission_char {
                    '0' => "不可  ",
                    '1' => "実    ",
                    '2' => "書    ",
                    '3' => "実書  ",
                    '4' => "読    ",
                    '5' => "読実  ",
                    '6' => "読書  ",
                    '7' => "読書実",
                    _ => {
                        break;
                    }
                }
            );
        }

        if self.mostbig_permission > permission.len() {
            self.mostbig_permission = permission.len()
        }

        permission
    }

    fn separate(&mut self) -> Result<()> {
        execute!(
            std::io::stderr(),
            SetForegroundColor(Color::Blue),
            Print(" │ "),
            SetForegroundColor(Color::White),
        )
    }
    fn format_utc_to_string(&mut self, utc_time: &DateTime<Utc>) -> String {
        utc_time.format("%Y年%m月%d日 %H時%M分%S秒 %Z").to_string()
    }

    fn draw_line(&mut self, draw_data: String) -> Result<()> {
        // 一行分の内容を描写

        let filesize = std::fs::metadata(draw_data.clone())
            .unwrap()
            .len()
            .to_string();

        // self.mostbig_size_filename -----------------------
        execute!(
            std::io::stderr(),
            SetForegroundColor(Color::White),
            Print(draw_data.clone()),
        )?;

        for _ in 0..self.mostbig_size_filename.len() - draw_data.len() {
            execute!(std::io::stderr(), Print(" "),)?;
        }
        // self.mostbig_size_filename -----------------------

        self.separate()?;

        // file size
        // self.mostbig_size_filelength -----------------------
        execute!(
            std::io::stderr(),
            SetForegroundColor(Color::Blue),
            Print(filesize.clone()),
        )?;

        for _ in 0..self.mostbig_size_length - filesize.len() {
            execute!(std::io::stderr(), Print(" "))?;
        }
        // self.mostbig_size_filelength -----------------------

        execute!(
            std::io::stderr(),
            SetForegroundColor(Color::White),
            Print(" B"),
        )?;
        // file size

        self.separate()?;

        // file permission
        let metadata = fs::symlink_metadata(draw_data.clone()).expect("Failed to get metadata");
        let hex_permission = self.shinsu(metadata.permissions().mode() as i64, 8);
        let permission = self.generate_permission_strings(hex_permission);

        execute!(std::io::stderr(), Print(permission.clone()))?;
        // file permission

        self.separate()?;

        let created_time = metadata.created().unwrap();
        let created_time: DateTime<Utc> = created_time.into();
        let created_time = self.format_utc_to_string(&created_time);

        execute!(std::io::stderr(), Print(created_time.clone()))?;

        self.separate()?;

        for _ in 0..(self.window_width - self.start_w) as usize
            - 4
            - self.mostbig_size_filename.len()
            - 3
            - self.mostbig_size_length
            - 2
            - 3
            - permission.len()
            - 3
            - created_time.len()
            - 1
        {
            execute!(std::io::stderr(), Print(" "));
        }

        Ok(())
    }

    pub fn ui(&mut self, print_strings: Vec<String>) -> Result<()> {
        execute!(std::io::stderr(), MoveTo(self.start_w, self.start_h))?;
        execute!(
            std::io::stderr(),
            SetForegroundColor(Color::Blue),
            Print("┌")
        )?;

        for _ in 1..self.window_width - 1 {
            execute!(std::io::stderr(), Print("─"))?;
        }
        execute!(std::io::stderr(), Print("┐"))?;
        self.cursor.1 = 1;
        execute!(std::io::stderr(), Print("\n"))?;
        execute!(
            std::io::stderr(),
            MoveTo(self.start_w, self.cursor.1 + self.start_h)
        )?;

        for i in 0..print_strings.len() {
            execute!(
                std::io::stderr(),
                Print("│ "),
                SetForegroundColor(Color::Yellow),
                SetBackgroundColor(GRUVBOX_BACKGROUND),
            )?;

            // 枠の中身-------------------------------------------------------------------------------

            if i == self.focus_index {
                execute!(std::io::stderr(), Print("> "))?;
            } else {
                execute!(std::io::stderr(), Print("  "))?;
            }

            self.draw_line(print_strings[i].clone());
            // --------------------------------------------------------------------------------------

            execute!(std::io::stderr(), SetForegroundColor(Color::Blue),)?;
            self.cursor.0 = self.window_width - 1;

            execute!(
                std::io::stderr(),
                MoveToColumn((self.cursor.0 + self.start_w).try_into().unwrap()),
                Print("│"),
                MoveTo(
                    self.start_w,
                    (i as u16 + self.cursor.1 + self.start_h + 1)
                        .try_into()
                        .unwrap()
                )
            )?;
        }
        self.cursor.1 += print_strings.len() as u16 - 1;

        execute!(
            std::io::stderr(),
            MoveToRow(self.cursor.1 + self.start_h + 2),
        )?;

        // 何も表示することがない場合の空欄
        for i in self.cursor.1..self.window_height - 2 {
            execute!(std::io::stderr(), Print("│"))?;
            for i in 1..self.window_width - 1 {
                execute!(std::io::stderr(), Print(" "))?;
            }
            execute!(
                std::io::stderr(),
                Print("│"),
                MoveTo(
                    self.start_w,
                    (i as u16 + self.start_h + 1).try_into().unwrap()
                )
            )?;
        }

        // 下の枠
        execute!(std::io::stderr(), Print("└"))?;
        for _ in 1..self.window_width - 1 {
            execute!(std::io::stderr(), Print("─"))?;
        }
        execute!(std::io::stderr(), Print("┘"))?;
        // -------------------------

        Ok(())
    }

    pub fn main(&mut self) -> Result<()> {
        self.get_in_dir()?;
        execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
        //self.draw_background();
        loop {
            // ui
            let _ = self.ui(self.in_dir_files.clone());

            // Key Read
            let _ = self.key_read(self.in_dir_files.len());

            if self.exit_flag {
                break;
            }
        }
        execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
        return Ok(());
    }

    pub fn sw(&mut self) -> Result<()> {
        let v: Vec<String> = vec![String::from(""), String::from("Hello, World2!!")];
        execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
        self.draw_background();
        loop {
            // ui
            let _ = self.ui(v.clone());

            // Key Read
            let _ = self.key_read(1);
            if self.exit_flag {
                break;
            }
        }
        execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
        return Ok(());
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let window_width = size().unwrap().0;
    let window_height = size().unwrap().1;
    let mut app = App::new(
        String::from(""),
        0,
        (0, 0),
        Vec::new(),
        window_width,
        window_height,
        0,
        0,
        0,
    );

    let ret = app.main();
    disable_raw_mode();
    ret
}
