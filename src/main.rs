mod parse;
mod pda;
mod tm;

use clap::Parser;
use core::{arch, panic};
use pda::ArchState;
use std::io::Read;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "show step by step execution trace")]
    verbose: bool,
    #[arg(id = "machine", help = "pda (*.pda) or tm (*.tm) description")]
    program: String,
    #[arg(id = "input")]
    input: String,
}

fn main() {
    let args = Arguments::parse();

    let mut program = String::new();
    std::fs::File::open(&args.program)
        .expect("failed to open machine description")
        .read_to_string(&mut program)
        .expect("failed to read machine description");

    fn banner(s: &str) -> String {
        "=".repeat(20) + " " + s + " " + &("=".repeat(20))
    }
    let banner_run = banner("RUN");
    let banner_err = banner("ERR");
    let banner_end = banner("END");
    let banner_split = "-".repeat(45);

    if args.program.ends_with(".pda") {
        let machine = match program.parse::<pda::PushDownAutomata>() {
            Ok(m) => m,
            Err(e) => {
                if args.verbose {
                    eprintln!("{}", banner_err);
                    eprintln!("{:?}", e);
                    eprintln!("{}", banner_end);
                } else {
                    eprintln!("syntax error");
                }
                std::process::exit(1);
            }
        };
        if args.verbose {
            println!("Input: {}", args.input);
        }

        let mut arch_state = match pda::ArchState::new(machine, &args.input) {
            Ok(m) => m,
            Err(e) => match e {
                pda::Exception::InvalidInput { col } => {
                    eprintln!("{}", banner_err);
                    eprintln!(
                        "error: '{}' was not declared in the set of input symbols",
                        args.input.chars().nth(col).unwrap()
                    );
                    eprintln!("Input: {}", args.input);
                    eprintln!("{}^", " ".repeat(7 + col));
                    eprintln!("{}", banner_end);
                    std::process::exit(1);
                }
                _ => panic!(),
            },
        };

        if args.verbose {
            println!("{}", banner_run);
        }
        loop {
            if args.verbose {
                print!("{}", arch_state);
                println!("{}", banner_split);
            }

            use pda::Exception::*;
            match arch_state.step() {
                Ok(_) => {}
                Err(e) => {
                    match e {
                        Accept => println!("true"),
                        Reject => println!("false"),
                        EmptyStack => eprintln!("error: empty stack!"),
                        Halt => eprintln!("error: halt!"),
                        _ => panic!()
                    }
                    match e {
                        EmptyStack | Halt => std::process::exit(1),
                        _ => ()
                    }
                    println!("{}", banner_end);
                    break;
                }
            }
        }
    } else if args.program.ends_with(".tm") {
        let machine = match program.parse::<tm::TuringMachine>() {
            Ok(m) => m,
            Err(e) => {
                if args.verbose {
                    eprintln!("{}", banner_err);
                    use tm::ParseErrorType::*;
                    match e.error {
                        Spec(s) => eprintln!("spec error: {:?}", s),
                        FieldNotFound => eprintln!("field '{}' not found", e.msg),
                        _ => {
                            eprintln!("syntax error at line {} offset {}:", e.pc, e.offset);
                            eprintln!("{}", e.inst);
                            eprintln!("{}^", " ".repeat(e.offset));
                        }
                    }
                    eprintln!("{}", banner_end);
                } else {
                    eprintln!("syntax error");
                }
                std::process::exit(1);
            }
        };

        if args.verbose {
            println!("Input: {}", args.input);
        }

        let mut arch_state = tm::ArchState::new(machine);
        match arch_state.input(&args.input) {
            Ok(_) => (),
            Err(e) => match e {
                tm::Exception::InvalidInput { input, offset } => {
                    eprintln!("{}", banner_err);
                    eprintln!(
                        "error: '{}' was not declared in the set of input symbols",
                        input.chars().nth(offset).unwrap()
                    );
                    eprintln!("Input: {}", input);
                    eprintln!("{}^", " ".repeat(7 + offset));
                    eprintln!("{}", banner_end);
                    std::process::exit(1);
                }
                _ => panic!(),
            },
        }

        if args.verbose {
            println!("{}", banner_run);
            loop {
                print!("{}", arch_state);
                println!("{}", banner_split);

                use tm::Exception::*;
                match arch_state.step() {
                    Ok(_) => {}
                    Err(e) => {
                        match e {
                            Halt { accept } => {
                                // match accept {
                                //     true => {
                                //         println!("Result: {}", arch_state.result().unwrap());
                                //     }
                                //     false => {
                                //         println!("error: halt!");
                                //     }
                                // }
                                println!("Result: {}", arch_state.result().unwrap());
                            }
                            _ => panic!(),
                        }
                        println!("{}", banner_end);
                        break;
                    }
                }
            }
        } else {
            while let Ok(_) = arch_state.step() {}
            println!("{}", arch_state.result().unwrap())
        }
    } else {
        log::error!("machine is not pda or tm");
        std::process::exit(1);
    }
}
