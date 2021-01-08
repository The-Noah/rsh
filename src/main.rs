use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

const RESET: &str = "\x1b[0m";
const BRIGHT: &str = "\x1b[1m";

fn cwd() -> String {
  env::current_dir().unwrap().to_str().unwrap().to_owned()
}

fn main() {
  loop {
    print!("{}{}${} ", cwd(), BRIGHT, RESET);
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    let mut commands = input.trim().split(" | ").peekable();
    let mut previous_command = None;

    while let Some(command) = commands.next() {
      let mut args = command.trim().split_whitespace();
      let command = args.next().unwrap();

      match command {
        "exit" => return,
        "cd" => {
          let new_dir = args.peekable().peek().map_or(".", |x| *x);
          let root = Path::new(new_dir);
          if let Err(e) = env::set_current_dir(&root) {
            eprintln!("{}", e);
          }
        }
        "pwd" => {
          println!("{}", cwd());
        }
        command => {
          let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| Stdio::from(output.stdout.unwrap()));

          let stdout = if commands.peek().is_some() { Stdio::piped() } else { Stdio::inherit() };

          let output = Command::new(command).args(args).stdin(stdin).stdout(stdout).spawn();

          match output {
            Ok(output) => {
              previous_command = Some(output);
            }
            Err(e) => {
              previous_command = None;
              eprintln!("{}", e);
            }
          };
        }
      }
    }

    if let Some(mut final_command) = previous_command {
      final_command.wait().unwrap();
    }
  }
}
