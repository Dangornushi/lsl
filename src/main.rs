use crossterm::{
    cursor::{self, Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, SetBackgroundColor},
    terminal::window_size,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, WindowSize,
    },
};
use rand::Error;
use std::env;
use std::fs;
use std::io::Result;
use std::path::Path;
use std::process::Command;

fn get_in_dir() -> Result<Vec<String>> {
    // ターミナル表示用のディレクトリ一覧配列をリセット
    //    self.in_dir_data.clear();
    // カーソルキーのインデックスもリセット
    //   self.counter = 0;
    /**/
    let mut in_dir_data = vec![];

    match fs::read_dir("./") {
        Ok(entries) => {
            // 各エントリをループ処理
            for entry in entries {
                in_dir_data.push(entry.unwrap().file_name().to_string_lossy().to_string());
            }
            return Ok(in_dir_data);
        }
        Err(err) => match err.kind() {
            _ => return Err(err),
        },
    };
}

fn render_dir_view(v: Vec<String>, dir_index: &mut usize) -> Vec<String> {
    execute!(std::io::stderr(), MoveTo(0, 1));
    for i in 0..v.len() + 1 {
        execute!(
            std::io::stderr(),
            Clear(ClearType::CurrentLine),
            MoveTo(0, i.try_into().unwrap())
        );
    }
    *dir_index = 0;
    get_in_dir().unwrap()
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut cursor = (0, 0);
    let mut dir_index = 0;
    let mut in_dir_data = vec![];
    let mut y = 0;
    in_dir_data = get_in_dir()?;
    execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
    loop {
        execute!(std::io::stderr(), MoveTo(0, 0),)?;
        execute!(std::io::stderr(), MoveTo(1, 0),)?;
        for i in 1..window_size().unwrap().columns {
            execute!(std::io::stderr(), Print("_"))?;
        }
        y = 1;
        execute!(std::io::stderr(), Print("\n"))?;

        for i in 0..in_dir_data.len() {
            if i == dir_index + 1 {
                execute!(std::io::stderr(), Print(" > "))?;
            } else {
                execute!(std::io::stderr(), Print("   "))?;
            }
            execute!(std::io::stderr(), Print(in_dir_data[i].clone()))?;
            execute!(std::io::stderr(), Print("\n"))?;
            execute!(std::io::stderr(), MoveTo(0, (i + y).try_into().unwrap()),)?;
        }
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                let mv_to = format!("{}/../", env::current_dir().unwrap().to_str().unwrap());
                let _ = env::set_current_dir(&Path::new(&mv_to));

                // pathに指されているものがディレクトリである
                // cd ../

                in_dir_data = render_dir_view(in_dir_data.clone(), &mut dir_index);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                let mv_to = format!(
                    "{}/{}",
                    env::current_dir().unwrap().to_str().unwrap(),
                    in_dir_data[dir_index + 1].clone()
                );

                let root = Path::new(&mv_to);

                match env::set_current_dir(&root) {
                    Ok(_) => {
                        // pathに指されているものがディレクトリである
                        // in_dir_dataをpathの内容に上書き

                        in_dir_data = render_dir_view(in_dir_data.clone(), &mut dir_index);
                    }
                    Err(_) => {
                        // pathに指されているものがファイルである
                        // enter keyを押した時にファイルであればvimを起動
                        execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
                        let mut child = Command::new("nvim")
                            .arg(in_dir_data[dir_index + 1].clone())
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
                if dir_index < in_dir_data.len() - 2 {
                    dir_index += 1
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                if dir_index > 0 {
                    dir_index -= 1
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
        cursor.0 = 0;
        cursor.0 = 1;
    }
    execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    println!("{:?}", in_dir_data);
    return Ok(());
}
