use std::{env, io::{self, Write}, time::Instant};
use crate::calc::{run_calculation};
pub mod calc;

fn main() {
    let mut flag_debug: bool = false;
    // If used as command we don't use interactive mode
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    if args.len() >= 1 {
        let mut expr_start = 0;
        for arg in args.iter() {
            match arg.trim() {
                "--debug" => {
                    expr_start += 1;
                    flag_debug = true;
                },
                _ => {

                }
            }
        }
        let input = args.split_off(expr_start).join(" ");
        if input.len() != 0 {
            let start = Instant::now();
            match run_calculation(input) {
                Ok(r) => {
                    println!("{}", r)
                }
                Err(err) => {
                    eprintln!("{}", err)
                },
            }
            if flag_debug {
                let stop = Instant::now();
                let calc_duration = stop - start;
                println!("in {} nanoseconds", calc_duration.as_nanos())
            }
            return;
        }
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

        let start = Instant::now();
        match run_calculation(input) {
            Ok(r) => {
                println!("= {}", r)
            }
            Err(err) => {
                eprintln!("! {}", err)
            },
        }
        if flag_debug {
            let stop = Instant::now();
            let calc_duration = stop - start;
            println!("in {} nanoseconds", calc_duration.as_nanos())
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

