use std::{env, fs::File, io::{stdout, Read}};

use editor::Editor;

mod editor;

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = if args.len() >= 2 {
        Some(&args[1])
    } else {
        None
    };

    let out = stdout();
    let content = match filename {
        Some(filename) => read_file(filename),
        None => String::new(),
    };
    let mut editor = Editor::new(out, content);

    editor.run(filename.to_string()).expect("Error while running editor");
}

fn read_file(filename: &str) -> String {
    let mut file = File::open(filename)
        .expect("Unable to open file.");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file into String");

    contents
}
