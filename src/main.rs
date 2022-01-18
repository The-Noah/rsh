use std::env;
use std::io::{stdin, stdout, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

mod color;

fn cwd() -> String {
  env::current_dir().unwrap().to_str().unwrap().to_owned()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let home_dir = &env::var("USERPROFILE")?;
  let home_path = Path::new(home_dir);

  'main_loop: loop {
    let current_dir = env::current_dir()?;
    let print_path = current_dir.strip_prefix(home_path).unwrap_or(current_dir.as_path());

    let current_branch = {
      let command = Command::new("git").arg("branch").arg("--no-color").arg("--show-current").stdout(Stdio::piped()).output()?;
      let text = String::from_utf8(command.stdout)?.replace("\n", "").replace("\r", "");

      text
    };

    print!(
      "{}{}{}{}",
      color::BLUE,
      if print_path != current_dir {
        if print_path != home_path {
          "~/"
        } else {
          "~"
        }
      } else {
        ""
      },
      print_path.to_str().unwrap().replace("\\", "/"),
      color::RESET,
    );

    if current_branch.len() > 0 {
      print!(" {}[{}]{}", color::YELLOW, current_branch, color::RESET);
    }

    print!("$ ");

    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;

    let mut commands = input.trim().split(" | ").peekable();
    let mut previous_command = None;

    while let Some(command) = commands.next() {
      let mut args = command.trim().split_whitespace();
      let command = match args.next() {
        Some(val) => val,
        None => continue,
      };

      match command {
        "exit" => break 'main_loop,
        "cd" => {
          let mut new_dir = args.peekable().peek().map_or(".", |x| *x);
          if new_dir == "~" {
            new_dir = home_dir;
          }
          let path = Path::new(new_dir);

          if let Err(e) = env::set_current_dir(&path) {
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
      final_command.wait()?;
    }
  }

  Ok(())
}
