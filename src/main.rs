use chrono::prelude::{DateTime, Datelike, Utc};
use crossterm::queue;
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
use std::io::Write;
use std::path::Path;
use std::process::Command;
extern crate chrono;
use std::path::PathBuf;
use std::{env, os::unix::fs::PermissionsExt};

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

#[derive(Debug, Copy, Clone)]
enum Style {
    Default,
    Border,
}
#[derive(Debug, Copy, Clone)]
struct TextBox {
    color: Color,
    width: usize,
    height: usize,
    style: Style,
}
impl TextBox {
    fn new(color: Color, width: usize, height: usize) -> Self {
        Self {
            color,
            width,
            height,
            style: Style::Default,
        }
    }

    pub fn style(&mut self, style: Style) -> TextBox {
        self.style = style;
        let r = self;
        return *r;
    }

    pub fn blank(&mut self, put_data: String) -> Result<()> {
        for _ in put_data.len()..self.width as usize {
            execute!(std::io::stderr(), Print(" "))?;
        }
        Ok(())
    }

    fn put_over_border(&mut self) -> Result<()> {
        // 上の枠
        queue!(std::io::stderr(), Print("┌"))?;
        for _ in 1..self.width - 1 {
            queue!(std::io::stderr(), Print("─"))?;
        }
        queue!(std::io::stderr(), Print("┐\n"))?;
        // -------------------------
        Ok(())
    }

    fn put_under_border(&mut self) -> Result<()> {
        // 下の枠
        queue!(std::io::stderr(), Print("└"))?;
        for _ in 1..self.width - 1 {
            queue!(std::io::stderr(), Print("─"))?;
        }
        queue!(std::io::stderr(), Print("┘"))?;
        // -------------------------
        Ok(())
    }

    pub fn put(&mut self, data: String) -> Result<()> {
        match self.style {
            Style::Default => {
                queue!(
                    std::io::stderr(),
                    SetForegroundColor(self.color),
                    Print(data.clone()),
                )?;

                if self.width == data.len() {
                    return Ok(());
                }

                for _ in 0..self.width - data.len() {
                    queue!(std::io::stderr(), Print(" "))?;
                }
            }
            Style::Border => {
                self.put_over_border()?;
                self.put_under_border()?;
            }
        }
        Ok(())
    }

    pub fn change_width(&mut self, new_width: usize) {
        self.width = new_width
    }
}

struct TextLine {
    width: usize,
    now_width: usize,
    text_box: TextBox,
    beam_style: usize,
}

impl TextLine {
    fn new(width: usize) -> Self {
        Self {
            width,
            now_width: 0,
            text_box: TextBox {
                color: Color::White,
                width: (width),
                height: (1),
                style: Style::Default,
            },
            beam_style: 0,
        }
    }

    fn set_beam_style(&mut self, style: usize) {
        self.beam_style = style;
    }

    fn create_text_box(&mut self, color: Color, width: usize, height: usize) -> TextBox {
        self.now_width += width;
        self.text_box = TextBox::new(color, width, height);
        return self.text_box.clone();
    }

    pub fn focus(&mut self) -> Result<()> {
        self.now_width += 2;

        queue!(
            std::io::stderr(),
            SetBackgroundColor(GRUVBOX_FOCUS_BACKGROUND),
        )?;
        execute!(std::io::stderr(), Print("> "))
    }

    pub fn unfocus(&mut self) -> Result<()> {
        self.now_width += 2;
        execute!(std::io::stderr(), Print("  "))
    }

    pub fn blank(&mut self) -> Result<()> {
        match self.beam_style {
            0 => {
                for _ in 0..self.width as usize - self.now_width {
                    queue!(std::io::stderr(), Print(" "))?;
                }
            }
            1 => {
                for _ in 0..self.width as usize - self.now_width {
                    queue!(std::io::stderr(), Print("─"))?;
                }
            }
            _ => {
                for _ in 0..self.width as usize - self.now_width {
                    queue!(std::io::stderr(), Print(" "))?;
                }
            }
        }
        Ok(())
    }

    pub fn separate(&mut self) -> Result<()> {
        self.now_width += 3;
        queue!(
            std::io::stderr(),
            SetForegroundColor(Color::Blue),
            Print(" │ "),
            SetForegroundColor(Color::White),
        )
    }
}

struct Title {
    text_box: TextBox,
    width: usize,
}

impl Title {
    fn new(width: usize) -> Self {
        Self {
            text_box: TextBox {
                color: Color::White,
                width: (width),
                height: (1),
                style: Style::Default,
            },
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
        }
    }

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

                    //--------------------------------------------------------

                    //--------------------------------------------------------
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

                let _ = self.render_dir_view();
            }
            // ESC ----------------------------------------------------------------------------
            // SPACE ---
            //
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                ..
            }) => {
                queue!(std::io::stderr(), Show, LeaveAlternateScreen)?;
                let mut sub_window =
                    App::new(String::from(""), 0, (0, 0), Vec::new(), 20, 10, 0, 100, 10);
                let _ = sub_window.main();
                queue!(std::io::stderr(), Hide, EnterAlternateScreen)?;

                //self.draw_background()
            }
            // SPACE ---

            // Enter---------------------------------------------------------------------------
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                let mv_to = format!(
                    "{}/{}",
                    env::current_dir().unwrap().to_str().unwrap(),
                    self.in_dir_files[self.focus_page][self.focus_index].clone()
                );

                let root = Path::new(&mv_to);

                match env::set_current_dir(&root) {
                    Ok(_) => {
                        // pathに指されているものがディレクトリである
                        // in_dir_dataをpathの内容に上書き

                        self.focus_page = 0;
                        let _ = self.render_dir_view();
                    }
                    Err(_) => {
                        // pathに指されているものがファイルである
                        // enter keyを押した時にファイルであればvimを起動
                        queue!(std::io::stderr(), Show, LeaveAlternateScreen)?;
                        let mut child = Command::new("nvim")
                            .arg(self.in_dir_files[self.focus_page][self.focus_index].clone())
                            .spawn()
                            .unwrap();
                        child.wait().unwrap();
                        queue!(std::io::stderr(), Hide, EnterAlternateScreen)?;

                        //self.draw_background();
                    }
                };
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
        text_line.text_box.put(permission.clone())?;

        text_line.separate()?;

        // file size
        let filesize = std::fs::metadata(draw_data.clone())
            .unwrap()
            .len()
            .to_string();

        text_line.create_text_box(Color::Blue, self.mostbig_size_length, 1);
        text_line.text_box.put(filesize)?;

        text_line.create_text_box(Color::White, 2, 1);
        text_line.text_box.put(String::from(" B"))?;

        text_line.separate()?;

        // file created time
        let created_time = self.format_utc_to_string(&(metadata.created().unwrap().into()));

        text_line.create_text_box(Color::Yellow, created_time.len(), 1);
        text_line.text_box.put(created_time.clone())?;

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
        text_line.text_box.put(draw_data.clone())?;

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

    pub fn ui(&mut self, print_strings: Vec<String>) -> Result<()> {
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

    pub fn main(&mut self) -> Result<()> {
        self.get_in_dir()?;
        execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
        //self.draw_background();

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
