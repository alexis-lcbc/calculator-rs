use std::{env, io::{self, Write}};
use crate::calc::{run_calculation};
pub mod calc;

fn main() {
    // If used as command we don't use interactive mode
    let mut args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        args.remove(0);
        let input = args.join(" ");
        match run_calculation(input) {
            Ok(r) => {
                println!("{}", r)
            }
            Err(err) => {
                eprintln!("{}", err)
            },
        }
        return;
    }

    println!("Very cool calculator!");
    println!("type /q to exit, /c to clear");
    loop {
        let input = read_input().expect("Could not read input");
        if input.starts_with("/q") {
            println!("goodbye!");
            break;
        }
        if input.starts_with("/c") {
            print!("\x1b[2J");
            print!("\x1b[H");
            println!("Very cool calculator!");
            continue;
        }
        match run_calculation(input) {
            Ok(r) => {
                println!("= {}", r)
            }
            Err(err) => {
                eprintln!("! {}", err)
            },
        }
    }
}

fn read_input() -> Result<String, io::Error> {
    let mut input = String::new();
    io::stdout().write("> ".as_bytes())?;
    io::stdout().flush()?;
    io::stdin().read_line(&mut input).expect("failed to readline");
    return Ok(input);
}

