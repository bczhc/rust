use console::Term;
use std::io;
use std::io::Write;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct LineProgress {
    pub length: u64,
    term: Term,
    term_width: u16,
    last_progress: u64,
    last_message: Option<String>,
}

impl LineProgress {
    pub fn new(len: u64) -> Self {
        let term = Term::stdout();
        let term_size = term.size();
        Self {
            length: len,
            term,
            term_width: term_size.1,
            last_progress: 0,
            last_message: None,
        }
    }

    fn progress_prefix(progress: u64, length: u64) -> String {
        format!("[{}/{}]", progress, length)
    }

    pub fn update(&mut self, progress: u64, message: String) -> io::Result<()> {
        self.last_progress = progress;
        self.last_message.replace(message);

        let message = self.last_message.as_ref();
        let message = message.unwrap();

        let progress_prefix = Self::progress_prefix(progress, self.length);
        let progress_prefix_width = progress_prefix.width();
        let message_width = message.width();
        let space_width = ' '.width().unwrap();
        let ellipse_width = "...".width();

        let line_string =
            if progress_prefix_width + space_width + message_width <= self.term_width as usize {
                format!("{} {}", progress_prefix, message)
            } else {
                let message_remain_width =
                    self.term_width as usize - progress_prefix_width - ellipse_width - space_width;
                let chars = message.chars().rev();
                let mut omitted_message_buf = String::new();
                let mut width_sum = 0_usize;
                for c in chars {
                    let width = c.width().unwrap_or(0);
                    width_sum += width;
                    if width_sum > message_remain_width {
                        break;
                    }
                    omitted_message_buf.push(c);
                }
                let omitted_message = omitted_message_buf.chars().rev().collect::<String>();
                format!("{} ...{}", progress_prefix, omitted_message)
            };

        self.term.clear_line()?;
        self.term.write_all(line_string.as_bytes())?;
        Ok(())
    }

    pub fn message(&mut self, msg: &str) -> io::Result<()> {
        self.term.clear_line()?;
        self.term.write_line(msg)?;
        match self.last_message {
            None => {}
            Some(ref msg) => {
                self.term.write_all(
                    format!(
                        "{} {}",
                        Self::progress_prefix(self.last_progress, self.length),
                        msg
                    )
                    .as_bytes(),
                )?;
            }
        }
        Ok(())
    }
}
