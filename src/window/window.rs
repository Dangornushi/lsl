use crate::Color;
use crate::TextLine;
use crossterm::cursor::MoveTo;
use crossterm::cursor::MoveToColumn;
use crossterm::cursor::MoveToRow;
use crossterm::queue;
use crossterm::style::Print;
use std::io::Result;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Nomal,
    Command,
    Edit,
}

#[derive(Debug, Copy, Clone)]
pub struct Window {
    now_mode: Mode,
    now_cursor_x: usize,
    now_cursor_y: usize,
    now_color: Color,
    start_width: usize,
    start_height: usize,
    window_width: usize,
    window_height: usize,
}

impl Window {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            now_mode: Mode::Nomal,
            now_cursor_x: 0,
            now_cursor_y: 0,
            now_color: Color::White,
            start_width: w,
            start_height: h,
            window_width: 1,
            window_height: 1,
        }
    }

    pub fn set_mode(&mut self, mode: Mode) -> Self {
        self.now_mode = mode;
        self.to_owned()
    }

    pub fn set_width(&mut self, width: usize) -> Self {
        self.window_width = width;
        self.to_owned()
    }

    pub fn set_hight(&mut self, hight: usize) -> Self {
        self.window_height = hight;
        self.to_owned()
    }

    pub fn set_color(&mut self, color: Color) -> Self {
        self.now_color = color;
        self.to_owned()
    }

    pub fn top_line(&mut self) -> Result<()> {
        queue!(
            std::io::stderr(),
            MoveTo(
                self.start_width as u16 + 1,
                (self.start_height + self.now_cursor_y) as u16 - 2
            )
        )?;

        self.now_cursor_y += 1;

        // 上限ライン-----------------------------------
        let mut border_line = TextLine::new(self.window_width as usize);
        border_line.set_beam_style(1);
        border_line.blank()?;
        // -----------------------------------------------------------------
        /*
                queue!(
                    std::io::stderr(),
                    MoveTo(
                        self.start_width as u16 + 1,
                        (self.start_height + self.now_cursor_y + 1) as u16 - 2
                    )
                )?;
        */
        queue!(
            std::io::stderr(),
            MoveToColumn(self.start_width as u16 + 2,),
            Print("\n")
        )?;
        Ok(())
    }

    pub fn put(&mut self, data: String) -> Result<()> {
        let mut put_line = TextLine::new(self.window_width as usize - 3);
        put_line
            .create_text_box(self.now_color, data.len(), 1)
            .put(data)?;
        put_line.blank()?;
        queue!(
            std::io::stderr(),
            MoveToColumn(self.start_width as u16 + 2,),
            Print("\n")
        )?;
        Ok(())
    }
}
