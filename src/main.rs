use std::io::{self, stdout, Write};
use crossterm::{cursor, event::{read, Event, KeyCode}, terminal, ExecutableCommand, QueueableCommand};

enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

fn handle_event(event: Event) -> Option<Action> {
    match event {
        Event::Key(event) => {
            match event.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('h') => Some(Action::MoveLeft),
                KeyCode::Char('j') => Some(Action::MoveDown),
                KeyCode::Char('k') => Some(Action::MoveUp),
                KeyCode::Char('l') => Some(Action::MoveRight),
                _ => None,
            }
        }
        _ => None,
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

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    let mut c = Cursor {
        x: 0,
        y: 0
    };

    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    loop {
        stdout.queue(cursor::MoveTo(c.x, c.y))?;
        stdout.flush()?;

        if let Some(action) = handle_event(read()?) {
            match action {
                Action::Quit => break,
                Action::MoveUp => c.move_up(),
                Action::MoveDown => c.move_down(),
                Action::MoveLeft => c.move_left(),
                Action::MoveRight => c.move_right()
            }
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode().expect("Unable to disable raw mode");

    Ok(())
}
