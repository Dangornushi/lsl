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

use crate::TextBox;

pub struct TextLine {
    width: usize,
    now_width: usize,
    text_box: TextBox,
    beam_style: usize,
    frame_style: usize,
}

impl TextLine {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            now_width: 0,
            text_box: TextBox::new(Color::White, width, 1),
            beam_style: 0,
            frame_style: 0,
        }
    }

    pub fn get_width(&mut self) -> usize {
        return self.width.clone();
    }

    pub fn set_beam_style(&mut self, style: usize) {
        self.beam_style = style;
    }

    fn set_frame_style(&mut self, style: usize) {
        self.frame_style = style;
    }

    pub fn create_text_box(&mut self, color: Color, width: usize, height: usize) -> TextBox {
        self.now_width += width;
        self.text_box = TextBox::new(color, width, height);
        return self.text_box.clone();
    }

    pub fn focus(&mut self) -> Result<()> {
        self.now_width += 2;

        queue!(
            std::io::stderr(),
            SetBackgroundColor(crate::GRUVBOX_FOCUS_BACKGROUND),
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

    pub fn put(&mut self, data: String) -> Result<()> {
        self.text_box.put(data)
    }
}
