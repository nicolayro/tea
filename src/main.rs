use std::{env, fs::File, io::{stdout, Read}, process::exit};

use editor::Editor;

mod editor;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("USAGE: tea <filename>");
        exit(1)
    }

    let filename = &args[1];
    let out = stdout();
    let content = read_file(filename);
    let mut editor = Editor::new(out, content);

    editor.run(filename).expect("Error while running editor");
}

fn read_file(filename: &str) -> String {
    let mut file = File::open(filename)
        .expect("Unable to open file.");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file into String");

    contents
}
