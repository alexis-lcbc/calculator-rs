use std::{env, io::{self, Write}, time::Instant};
use crate::calc::{CalcError, lexer, parse, run_calculation};
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
            match run_calc_debug(input) {
                Ok(r) => {
                    println!("{}", r)
                }
                Err(err) => {
                    eprintln!("{}", err)
                },
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

        match run_calc_debug(input) {
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


fn run_calc_debug(input: String) -> Result<f64, CalcError> {
    let lexer_start = Instant::now();
    let tokens = lexer(input)?;
    let lexer_stop = Instant::now();
    println!("Lexing took {} ns", (lexer_stop - lexer_start).as_nanos());
    let parser_start = Instant::now();
    let optree = parse(&tokens.as_slice())?;
    let parser_stop = Instant::now();
    println!("parsing took {} ns", (parser_stop - parser_start).as_nanos());
    let eval_start = Instant::now();
    let result = optree.evaluate();
    let eval_stop = Instant::now();
    println!("eval took {} ns", (eval_stop - eval_start).as_nanos());
    println!("total took {} ns", (eval_stop - lexer_start).as_nanos());

    return result;
}