use std::{env, fs::File, io::{self, stdout, Read, Write}};
use crossterm::{cursor, event::{read, Event, KeyCode}, style, terminal, ExecutableCommand, QueueableCommand};

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

fn handle_event(event: Event, mode: &Mode) -> Option<Action> {
    match mode {
        Mode::Normal => {
            match event {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char('q') => Some(Action::Quit),
                        KeyCode::Char('h') => Some(Action::MoveLeft),
                        KeyCode::Char('j') => Some(Action::MoveDown),
                        KeyCode::Char('k') => Some(Action::MoveUp),
                        KeyCode::Char('l') => Some(Action::MoveRight),
                        KeyCode::Char('i') => Some(Action::ChangeMode(Mode::Insert)),
                        _ => None,
                    }
                }
                _ => None,
            }
        },
        Mode::Insert => {
            match event {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char(c) => Some(Action::InsertChar(c)),
                        KeyCode::Esc => Some(Action::ChangeMode(Mode::Normal)),
                        KeyCode::Backspace => Some(Action::RemoveChar),
                        KeyCode::Enter => Some(Action::NewLine),
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
}

fn run_editor<T: Write>(file: String, mut stdout: T, mut c: Cursor) -> io::Result<()>  {
    let mut mode = Mode::Normal;
    let mut lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();

    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    loop {
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        let size = terminal::size()?;

        // Display file
        for (i, line) in lines.iter().enumerate() {
            stdout.queue(cursor::MoveTo(0, i as u16))?;
            stdout.queue(style::Print(line))?;
        }

        // Display mode
        stdout.queue(cursor::MoveTo(0, size.1))?;
        stdout.queue(style::Print(format!("-- {:?} --", mode)))?;

        // Display pos
        stdout.queue(cursor::MoveTo(size.0 - 10, size.1))?;
        stdout.queue(style::Print(format!("{}, {}", c.x, c.y)))?;

        stdout.queue(cursor::MoveTo(c.x, c.y))?;
        stdout.flush()?;

        if let Some(action) = handle_event(read()?, &mode) {
            match action {
                Action::Quit => break,
                Action::MoveUp => c.move_up(),
                Action::MoveDown => c.move_down(),
                Action::MoveLeft => c.move_left(),
                Action::MoveRight => c.move_right(),
                Action::ChangeMode(new_mode) => mode = new_mode,
                Action::InsertChar(ch) => {
                    if let Some(line) = lines.get_mut(c.y as usize) {
                        if c.x < line.len() as u16 {
                            line.insert(c.x as usize, ch);
                            c.move_right();
                        }
                    }
                },
                Action::RemoveChar => {
                    if c.x == 0 {
                        if c.y != 0 && c.y < lines.len() as u16 {
                            let old_line = lines.remove(c.y as usize);
                            if let Some(line) = lines.get_mut((c.y-1) as usize) {
                                c.move_up();
                                c.x = line.len() as u16;
                                line.push_str(&old_line);
                            }
                        }
                    } else if let Some(line) = lines.get_mut(c.y as usize) {
                        if c.x < line.len() as u16 {
                            c.move_left();
                            line.remove(c.x as usize);
                        }
                    }
                },
                Action::NewLine => {
                    if c.y < lines.len() as u16 {
                        let line = lines.remove(c.y as usize);
                        let (left, right) = line.split_at(c.x as usize);
                        lines.insert(c.y as usize, right.to_string());
                        lines.insert(c.y as usize, left.to_string());
                        c.x = 0;
                        c.move_down();
                    }
                }
            }
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode().expect("Unable to disable raw mode");
    Ok(())
}

fn read_file(filename: &str) -> String {
    let mut file = File::open(filename)
        .expect("Unable to open file.");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file into String");

    contents
}

fn main() {
    let stdout = stdout();
    let cursor = Cursor::new(0, 0);

    let args: Vec<String> = env::args().collect();
    let filename = if args.len() >= 2 {
        &args[1]
    } else {
        "main.c"
    };

    let file = read_file(filename);

    run_editor(file, stdout, cursor).unwrap();
}
