use std::{env, fs::File, io::{self, stdout, Read, Write}};
use crossterm::{cursor, event::{read, Event, KeyCode}, style, terminal, ExecutableCommand, QueueableCommand};

enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

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
                        KeyCode::Char('q') => Some(Action::Quit),
                        KeyCode::Char('h') => Some(Action::MoveLeft),
                        KeyCode::Char('j') => Some(Action::MoveDown),
                        KeyCode::Char('k') => Some(Action::MoveUp),
                        KeyCode::Char('l') => Some(Action::MoveRight),
                        KeyCode::Esc => Some(Action::ChangeMode(Mode::Normal)),
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
        self.y = self.y.saturating_sub(1)
    }

    pub fn move_down(&mut self) {
        self.y += 1
    }

    pub fn move_left(&mut self) {
        self.x = self.x.saturating_sub(1)
    }

    pub fn move_right(&mut self) {
        self.x += 1
    }
}

fn run_editor<T: Write>(file: String, mut stdout: T, mut c: Cursor) -> io::Result<()>  {
    let mut mode = Mode::Normal;

    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    // Write file
    for (i, line) in file.lines().enumerate() {
        stdout.queue(cursor::MoveTo(0, i as u16))?;
        stdout.queue(style::Print(line))?;
    };
    stdout.flush()?;

    loop {
        stdout.queue(cursor::MoveTo(0, terminal::size()?.1))?;
        stdout.queue(style::Print(format!("-- {:?} --", mode)))?;

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
