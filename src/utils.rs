use std::io::{self, Write};
use text_colorizer::*;

pub fn get_input(prompt: &str) -> String {
    print!("{}", prompt.bright_blue());
    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim().to_owned()
}
