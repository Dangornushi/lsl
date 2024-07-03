use chrono::prelude::{DateTime, Utc};
use crossterm::queue;
use crossterm::{
    cursor::{Hide, MoveTo, MoveToColumn, MoveToRow, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::fs;
use std::io::Result;
use std::io::Write;
use std::path::Path;
use std::process::Command;
extern crate chrono;
use std::{env, os::unix::fs::PermissionsExt};

mod window;

use crate::window::textbox::Style;
use crate::window::window::Mode;
use window::textbox::TextBox;
use window::textline::TextLine;
use window::window::Window;

const GRUVBOX_BACKGROUND: Color = Color::Rgb {
    r: 40,
    g: 40,
    b: 40,
};

const GRUVBOX_FOCUS_BACKGROUND: Color = Color::Rgb {
    r: 50,
    g: 50,
    b: 50,
};

struct Title {
    text_box: TextBox,
    width: usize,
}

impl Title {
    fn new(width: usize) -> Self {
        Self {
            text_box: TextBox::new(Color::White, width, 1),
            width,
        }
    }

    fn set(&mut self, title: String) -> Result<()> {
        self.text_box = TextBox::new(Color::Cyan, 10, 1).style(Style::Border);
        self.text_box.put(title)
    }
}

struct App {
    mostbig_size_filename: String,
    mostbig_size_length: usize,
    mostbig_permission: usize,
    in_dir_files: Vec<Vec<String>>,
    cursor: (u16, u16),
    window_width: u16,
    window_height: u16,
    focus_index: usize,
    start_w: u16,
    start_h: u16,
    exit_flag: bool,
    focus_page: usize,
    pwd: String,
    mode: Mode,
    input_buffer: String,
}

impl App {
    fn new(
        mostbig_size_filename: String,
        mostbig_size_length: usize,
        cursor: (u16, u16),
        in_dir_files: Vec<Vec<String>>,
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
            focus_page: 0,
            pwd: String::new(),
            mode: Mode::Nomal,
            input_buffer: String::new(),
        }
    }

    // self.in_dir_filesの内容を更新
    fn get_in_dir(&mut self) -> Result<()> {
        self.in_dir_files.clear();
        let mut tmp_vec: Vec<String> = Vec::new();
        let mut page_counter = 0;
        match fs::read_dir("./") {
            Ok(entries) => {
                self.pwd = env::current_dir().unwrap().display().to_string();
                for entry in entries {
                    if page_counter > self.window_height as usize - 3 {
                        self.in_dir_files.push(tmp_vec.clone());
                        tmp_vec.clear();
                        page_counter = 0;
                    }
                    let filename = entry.unwrap().file_name().to_string_lossy().to_string();
                    let filesize = std::fs::metadata(filename.clone())
                        .unwrap()
                        .len()
                        .to_string();
                    tmp_vec.push(filename.clone());
                    if self.mostbig_size_filename.len() < filename.len() {
                        self.mostbig_size_filename = filename.clone();
                    }

                    if self.mostbig_size_length < filesize.len() {
                        self.mostbig_size_length = filesize.len();
                    }
                    page_counter += 1;
                }
                if page_counter < self.window_height as usize - 2 {
                    self.in_dir_files.push(tmp_vec.clone());
                }
                return Ok(());
            }
            Err(err) => match err.kind() {
                _ => return Err(err),
            },
        };
    }

    fn render_dir_view(&mut self) -> Result<()> {
        queue!(
            std::io::stderr(),
            MoveTo(self.start_w + 0, self.start_h + 1)
        )?;
        for i in 0..self.mostbig_size_filename.len() + 1 {
            queue!(
                std::io::stderr(),
                Clear(ClearType::CurrentLine),
                MoveTo(0, (self.start_h + i as u16).try_into().unwrap())
            )?;
        }
        self.focus_index = 0;
        self.get_in_dir().unwrap();
        Ok(())
    }

    fn cd(&mut self, path: String) -> Result<()> {
        let mv_to = format!("{}/{}", env::current_dir().unwrap().to_str().unwrap(), path);
        match env::set_current_dir(Path::new(&mv_to)) {
            Ok(_) => {
                // pathに指されているものがディレクトリである
                // in_dir_dataをpathの内容に上書き

                self.focus_page = 0;
                self.render_dir_view()
            }
            Err(e) => {
                self.input_buffer.clear();
                Err(e)
            }
        }
    }

    fn nomal_key_read(&mut self, max_down: usize) -> Result<()> {
        match read()? {
            // ESC ----------------------------------------------------------------------------
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                let mv_to = format!("{}/../", env::current_dir().unwrap().to_str().unwrap());
                let _ = env::set_current_dir(&Path::new(&mv_to));

                // pathに指されているものがディレクトリである
                // cd ../

                let _ = self.render_dir_view();
            }
            // ESC ----------------------------------------------------------------------------
            // SPACE ---
            //
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                ..
            }) => {
                match read()? {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('f'),
                        ..
                    }) => {
                        //commandmode
                        self.mode = Mode::Command;
                    }
                    _ => {
                        return Ok(());
                    }
                }

                //self.draw_background()
            }
            // SPACE ---

            // Enter---------------------------------------------------------------------------
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                match self.cd(self.in_dir_files[self.focus_page][self.focus_index].clone()) {
                    Err(e) => {
                        // pathに指されているものがファイルである
                        // enter keyを押した時にファイルであればvimを起動
                        queue!(std::io::stderr(), Show, LeaveAlternateScreen)?;

                        let mut child = Command::new("nvim")
                            .arg(self.in_dir_files[self.focus_page][self.focus_index].clone())
                            .spawn()?;
                        child.wait().unwrap();
                        queue!(std::io::stderr(), Hide, EnterAlternateScreen)?;
                    }
                    _ => {}
                }
                let _ = std::io::stdout().flush();
            }
            // Enter --------------------------------------------------------------------------

            // WASD ---------------------------------------------------------------------------
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                if max_down > 1 && self.focus_index < max_down - 1 {
                    self.focus_index += 1
                } else if self.in_dir_files.len() > self.focus_page + 1 {
                    self.focus_page += 1;
                    self.focus_index = 0;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                if self.focus_index > 0 {
                    self.focus_index -= 1
                } else if 0 < self.focus_page {
                    self.focus_page -= 1;
                    self.focus_index = self.window_height as usize - 3;
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
        self.cursor.0 = 1;
        return Ok(());
    }

    fn command_key_read(&mut self) -> Result<()> {
        if let Event::Key(KeyEvent { code, .. }) = read()? {
            match code {
                KeyCode::Esc => {
                    self.input_buffer.clear();
                    self.mode = Mode::Nomal;
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Enter => {
                    self.cd(self.input_buffer.clone())?;
                    let _ = std::io::stdout().flush();
                    self.mode = Mode::Nomal;
                }

                _ => {}
            }
        }

        Ok(())
    }

    pub fn key_read(&mut self, max_down: usize) -> Result<()> {
        // ------------------------------------------------------------------------------------
        return match self.mode {
            Mode::Nomal => self.nomal_key_read(max_down),
            Mode::Command => self.command_key_read(),
            Mode::Edit => Ok(()),
        };
        // ------------------------------------------------------------------------------------
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

    fn format_utc_to_string(&mut self, utc_time: &DateTime<Utc>) -> String {
        utc_time.format("%Y/%m/%d %H:%M").to_string()
    }

    fn draw_line(&mut self, draw_data: String, counter: usize) -> Result<()> {
        // 一行分の内容を描写

        let mut text_line = TextLine::new(self.window_width as usize - 2);

        if counter == self.focus_index {
            text_line.focus()?;
        } else {
            text_line.unfocus()?;
        }

        // file permission
        let metadata = fs::symlink_metadata(draw_data.clone()).expect("Failed to get metadata");
        let hex_permission = self.shinsu(metadata.permissions().mode() as i64, 8);
        let permission = self.generate_permission_strings(hex_permission);

        text_line.create_text_box(Color::White, permission.len(), 1);
        text_line.put(permission.clone())?;

        text_line.separate()?;

        // file size
        let filesize = std::fs::metadata(draw_data.clone())
            .unwrap()
            .len()
            .to_string();

        text_line.create_text_box(Color::Blue, self.mostbig_size_length, 1);
        text_line.put(filesize)?;

        text_line.create_text_box(Color::White, 2, 1);
        text_line.put(String::from(" B"))?;

        text_line.separate()?;

        // file created time
        let created_time = self.format_utc_to_string(&(metadata.created().unwrap().into()));

        text_line.create_text_box(Color::Yellow, created_time.len(), 1);
        text_line.put(created_time.clone())?;

        text_line.separate()?;

        // file name

        let filename_color = match std::fs::metadata(draw_data.clone()).unwrap().is_dir() {
            true => {
                // directory
                Color::DarkBlue
            }
            false => {
                // file

                let index = match draw_data.find(".") {
                    Some(index) => index,
                    _ => 0,
                };

                let extracted_text = &draw_data[index..];

                match extracted_text {
                    ".rs" => Color::Rgb {
                        r: 255,
                        g: 158,
                        b: 101,
                    },
                    _ => Color::White,
                }
            }
        };

        text_line.create_text_box(filename_color, self.mostbig_size_filename.len(), 1);
        text_line.put(draw_data.clone())?;

        text_line.blank()?;

        /*
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
                    - 1
                {
                    queue!(std::io::stderr(), Print(" "));
                }
        */

        queue!(std::io::stderr(), SetBackgroundColor(GRUVBOX_BACKGROUND))?;
        Ok(())
    }

    fn find_dir(&mut self, serch_word: String, directory_vec: Vec<String>) -> Vec<String> {
        let mut return_vec = vec![];

        for i in directory_vec {
            if i.starts_with(&serch_word) {
                return_vec.push(i);
            }
        }

        return_vec
    }

    fn nomal_ui(&mut self, print_strings: Vec<String>) -> Result<()> {
        queue!(std::io::stderr(), MoveTo(self.start_w, self.start_h))?;
        queue!(
            std::io::stderr(),
            SetForegroundColor(Color::Blue),
            SetBackgroundColor(GRUVBOX_BACKGROUND),
            Print("┌")
        )?;

        let mut text_line = TextLine::new(self.window_width as usize - 2);
        text_line.set_beam_style(1);

        text_line
            .create_text_box(Color::Cyan, self.pwd.len(), 1)
            .put(self.pwd.clone())?;

        text_line
            .create_text_box(Color::Blue, 2, 1)
            .put("-[".to_string())?;

        let focus_page_char = (self.focus_page + 1).to_string();

        text_line
            .create_text_box(Color::Yellow, focus_page_char.clone().len(), 1)
            .put(focus_page_char)?;

        text_line
            .create_text_box(Color::Yellow, 1, 1)
            .put("/".to_string())?;

        let in_dir_files_char = self.in_dir_files.len();
        text_line
            .create_text_box(Color::Yellow, in_dir_files_char.clone(), 1)
            .put(in_dir_files_char.to_string())?;

        text_line
            .create_text_box(Color::Blue, 1, 1)
            .put("]".to_string())?;

        text_line.blank()?;

        queue!(std::io::stderr(), Print("┐"))?;
        self.cursor.1 = 1;
        queue!(std::io::stderr(), Print("\n"))?;
        queue!(
            std::io::stderr(),
            MoveTo(self.start_w, self.cursor.1 + self.start_h)
        )?;

        for i in 0..if print_strings.len() > self.window_height as usize - 2 {
            self.window_height as usize - 2
        } else {
            print_strings.len()
        } {
            queue!(
                std::io::stderr(),
                Print("│ "),
                SetForegroundColor(Color::Yellow),
                SetBackgroundColor(GRUVBOX_BACKGROUND),
            )?;

            // 枠の中身-------------------------------------------------------------------------------

            let _ = self.draw_line(print_strings[i].clone(), i);
            // --------------------------------------------------------------------------------------

            queue!(std::io::stderr(), SetForegroundColor(Color::Blue),)?;
            self.cursor.0 = self.window_width - 1;

            queue!(
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

        queue!(
            std::io::stderr(),
            MoveToRow(self.cursor.1 + self.start_h + 2),
        )?;

        // 何も表示することがない場合の空欄
        for i in if print_strings.len() > self.window_height as usize - 2 {
            0..0
        } else {
            self.cursor.1..self.window_height - 1
        } {
            queue!(std::io::stderr(), Print("│"))?;
            for _ in 1..self.window_width - 1 {
                queue!(std::io::stderr(), Print(" "))?;
            }
            queue!(
                std::io::stderr(),
                Print("│"),
                MoveTo(
                    self.start_w,
                    (i as u16 + self.start_h + 1).try_into().unwrap()
                )
            )?;
        }

        // 下の枠
        queue!(std::io::stderr(), Print("└"))?;

        for _ in 1..self.window_width - 1 {
            queue!(std::io::stderr(), Print("─"))?;
        }
        queue!(std::io::stderr(), Print("┘"))?;
        // -------------------------
        Ok(())
    }

    pub fn ui(&mut self, print_strings: Vec<String>) -> Result<()> {
        self.nomal_ui(print_strings)?;
        match self.mode {
            Mode::Nomal => {}
            Mode::Edit => {}
            Mode::Command => {
                let auto_correct = self.find_dir(
                    self.input_buffer.to_owned(),
                    self.in_dir_files.get(self.focus_page).unwrap().to_owned(),
                );

                self.draw_auto_correct(auto_correct)?;

                queue!(
                    std::io::stderr(),
                    SetBackgroundColor(GRUVBOX_BACKGROUND),
                    MoveTo(self.start_w, self.start_h + self.window_height)
                )?;

                let mut text_line = TextLine::new(self.window_width as usize);

                //text_line.set_frame_style(1);

                text_line
                    .create_text_box(Color::Blue, 4, 1)
                    .put("└[".to_string())?;

                text_line
                    .create_text_box(Color::Blue, self.input_buffer.len(), 1)
                    .put(self.input_buffer.clone())?;

                text_line.blank()?;

                text_line
                    .create_text_box(Color::Blue, 4, 1)
                    .put("]┘".to_string())?;
            }
        }

        Ok(())
    }

    fn auto_correct_line(&mut self, data: String, hight: u16) -> Result<()> {
        queue!(std::io::stderr(), MoveTo(self.start_w + 2, hight,))?;
        let mut border_line = TextLine::new(self.window_width as usize - 3);
        border_line
            .create_text_box(Color::Blue, data.len(), 1)
            .put(data)?;
        border_line.blank()?;

        Ok(())
    }

    fn draw_auto_correct_notfound(&mut self) -> Result<()> {
        // ---------------------------------------------------------
        queue!(
            std::io::stderr(),
            MoveTo(
                self.start_w + 1,
                self.start_h + self.window_height as u16 - 1 - 2
            )
        )?;
        let mut border_line = TextLine::new(self.window_width as usize - 2);
        border_line.set_beam_style(1);
        border_line.blank()?;

        queue!(
            std::io::stderr(),
            MoveTo(
                self.start_w + 2,
                self.start_h + self.window_height as u16 - 2
            )
        )?;
        let mut notfound_line = TextLine::new(self.window_width as usize - 4);
        let width = notfound_line.get_width();
        notfound_line
            .create_text_box(Color::Red, width, 1)
            .put("Not found".to_string())?;
        // ---------------------------------------------------------
        Ok(())
    }

    pub fn draw_auto_correct(&mut self, v: Vec<String>) -> Result<()> {
        if v.is_empty() {
            self.draw_auto_correct_notfound()
        } else {
            // 線を描き始める一番上（予測変換ウィンドウの上限ライン）------
            let mut auto_correct_window = Window::new(0, self.window_height as usize - v.len())
                .set_mode(Mode::Nomal)
                .set_width(self.window_width as usize - 2)
                .set_hight(v.len());
            auto_correct_window.top_line()?;
            // -----------------------------------------------------------

            // 予測変換たち v -> 予想されるファイル・ディレクトリの集合---------------------------------------------------
            for (_, item) in v.iter().enumerate() {
                auto_correct_window
                    .set_color(Color::Blue)
                    .put((*item).clone())?;
                /*
                // 予想されるファイル・ディレクトリ名を羅列---------------------------------------------------------------
                self.auto_correct_line(
                    item.to_owned(),
                    self.start_h + self.window_height - v.len() as u16 + /*<this>*/index.to_owned() as u16/*</this>*/ - 1,
                )?;
                //--------------------------------------------------------------------------------------------------------
                */
            }
            // -----------------------------------------------------------------------------------------------------------
            Ok(())
        }
    }

    pub fn main(&mut self) -> Result<()> {
        self.get_in_dir()?;
        execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;

        loop {
            // ui
            let _ = self.ui(self.in_dir_files[self.focus_page].clone());
            // Key Read
            let _ = self.key_read(
                if self.in_dir_files.len() > self.window_height as usize - 1 {
                    self.window_height as usize - 1
                } else {
                    self.in_dir_files[self.focus_page].len()
                },
            );

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
    let _ = disable_raw_mode();
    ret
}
