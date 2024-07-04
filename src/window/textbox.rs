use crossterm::queue;
use crossterm::{
    execute,
    style::{Color, Print, SetForegroundColor},
};
use std::io::Result;
extern crate chrono;

#[derive(Debug, Copy, Clone)]
pub enum Style {
    Default,
    Border,
}
#[derive(Debug, Copy, Clone)]
pub struct TextBox {
    color: Color,
    width: usize,
    height: usize,
    style: Style,
}
impl TextBox {
    pub fn new(color: Color, width: usize, height: usize) -> Self {
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
