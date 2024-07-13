use std::{fs, io::{self, Stdout, Write}};

use crossterm::{cursor, event, style, terminal, ExecutableCommand, QueueableCommand};

enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    InsertChar(char),
    RemoveChar,
    NewLine,

    ChangeMode(Mode)
}

#[derive(Debug)]
enum Mode {
    Normal,
    Insert
}

fn handle_event(e: event::Event, mode: &Mode) -> Option<Action> {
    match mode {
        Mode::Normal => {
            match e {
                event::Event::Key(event) => {
                    match event.code {
                        event::KeyCode::Char('q') => Some(Action::Quit),
                        event::KeyCode::Char('h') => Some(Action::MoveLeft),
                        event::KeyCode::Char('j') => Some(Action::MoveDown),
                        event::KeyCode::Char('k') => Some(Action::MoveUp),
                        event::KeyCode::Char('l') => Some(Action::MoveRight),
                        event::KeyCode::Char('i') => Some(Action::ChangeMode(Mode::Insert)),
                        _ => None,
                    }
                }
                _ => None,
            }
        },
        Mode::Insert => {
            match e {
                event::Event::Key(event) => {
                    match event.code {
                        event::KeyCode::Char(c) => Some(Action::InsertChar(c)),
                        event::KeyCode::Esc => Some(Action::ChangeMode(Mode::Normal)),
                        event::KeyCode::Backspace => Some(Action::RemoveChar),
                        event::KeyCode::Enter => Some(Action::NewLine),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
    }
}

struct Cursor {
    x: u16,
    y: u16,
}

impl Cursor {
    pub fn new(x: u16, y: u16) -> Cursor {
        Cursor { x, y }
    }

    pub fn move_up(&mut self) {
        self.y = self.y.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        self.y += 1;
    }

    pub fn move_left(&mut self) {
        self.x = self.x.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.x += 1;
    }

    pub fn pos(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

pub struct Editor {
    out: Stdout,
    cursor: Cursor,
    lines: Vec<String>,  // TODO: Improve data structure
    mode: Mode,
}

impl Editor {
    pub fn new(out: Stdout, content: String) -> Self {
        Self {
            out,
            cursor: Cursor::new(0, 0),
            lines: content.lines().map(|x| x.to_string()).collect(),
            mode: Mode::Normal,
        }
    }

    fn render(&mut self) -> io::Result<()> {
        let size = terminal::size()?;
        let (x, y) = self.cursor.pos();

        // Clean
        self.out.execute(terminal::Clear(terminal::ClearType::All))?;

        // Display file
        for (i, line) in self.lines.iter().enumerate() {
            self.out.queue(cursor::MoveTo(0, i as u16))?;
            self.out.queue(style::Print(line))?;
        }

        // Display mode
        self.out.queue(cursor::MoveTo(0, size.1))?;
        self.out.queue(style::Print(format!("-- {:?} --", self.mode)))?;

        // Display pos
        self.out.queue(cursor::MoveTo(size.0 - 10, size.1))?;
        self.out.queue(style::Print(format!("{}, {}", x, y)))?;

        // Set cursor
        self.out.queue(cursor::MoveTo(x, y))?;

        self.out.flush()?;
        Ok(())
    }

    pub fn run(&mut self, filename: String) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        self.out.execute(terminal::EnterAlternateScreen)?;

        loop {
            self.render()?;

            if let Some(action) = handle_event(event::read()?, &self.mode) {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => self.cursor.move_up(),
                    Action::MoveDown => self.cursor.move_down(),
                    Action::MoveLeft => self.cursor.move_left(),
                    Action::MoveRight => self.cursor.move_right(),
                    Action::ChangeMode(new_mode) => self.mode = new_mode,
                    Action::InsertChar(ch) => {
                        if let Some(line) = self.lines.get_mut(self.cursor.y as usize) {
                            if self.cursor.x <= line.len() as u16 {
                                line.insert(self.cursor.x as usize, ch);
                                self.cursor.move_right();
                            }
                        }
                    },
                    Action::RemoveChar => {
                        if self.cursor.x == 0 {
                            if self.cursor.y != 0 && self.cursor.y < self.lines.len() as u16 {
                                let old_line = self.lines.remove(self.cursor.y as usize);
                                if let Some(line) = self.lines.get_mut((self.cursor.y-1) as usize) {
                                    self.cursor.move_up();
                                    self.cursor.x = line.len() as u16;
                                    line.push_str(&old_line);
                                }
                            }
                        } else if let Some(line) = self.lines.get_mut(self.cursor.y as usize) {
                            if self.cursor.x <= line.len() as u16 {
                                self.cursor.move_left();
                                line.remove(self.cursor.x as usize);
                            }
                        }
                    },
                    Action::NewLine => {
                        if self.cursor.y <= self.lines.len() as u16 {
                            let line = self.lines.remove(self.cursor.y as usize);
                            let (left, right) = line.split_at(self.cursor.x as usize);
                            self.lines.insert(self.cursor.y as usize, right.to_string());
                            self.lines.insert(self.cursor.y as usize, left.to_string());
                            self.cursor.x = 0;
                            self.cursor.move_down();
                        }
                    }
                }
            }
        }

        self.out.execute(terminal::Clear(terminal::ClearType::All))?;

        self.out.queue(cursor::MoveTo(2, 2))?;
        self.out.queue(style::Print(format!("Write to file?")))?;
        self.out.flush()?;

        let save = match event::read()? {
            event::Event::Key(event) => {
                match event.code {
                    event::KeyCode::Char('w') => true,
                    _ => false,
                }
            }
            _ => false,
        };

        if save  {
            fs::write(filename, self.lines.join("\n"))?;
        }

        self.out.flush()?;

        self.out.execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

}


