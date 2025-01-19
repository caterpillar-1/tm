#[allow(non_snake_case, non_camel_case_types)]
mod automata;
mod parse;

use clap::Parser;
use std::io::Read;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "show step by step execution trace")]
    verbose: bool,
    #[arg(
        id = "machine",
        help = "dfa (*.dfa), pda (*.pda) or tm (*.tm) description"
    )]
    program: String,
    #[arg(id = "input")]
    input: String,
}

enum Mode {
    Dfa,
    Pda,
    Tm,
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

    if args.verbose {
        println!("Input: {}", args.input);
    }

    let mode: Mode;

    let mut arch_state: Box<dyn automata::ArchState>;

    if args.program.ends_with(".dfa") {
        mode = Mode::Dfa;
        unimplemented!()
    } else if args.program.ends_with(".pda") {
        mode = Mode::Pda;
        let machine: automata::PushDownAutomata = match program.parse() {
            Ok(m) => m,
            Err((pos, err)) => {
                eprintln!("{}", banner_err);
                eprint!("{}", pos);
                eprintln!("{:?}", err);
                eprintln!("{}", banner_end);
                std::process::exit(1);
            }
        };
        arch_state = Box::new(automata::PdaArchState::new(machine));
    } else if args.program.ends_with(".tm") {
        mode = Mode::Tm;
        let machine: automata::TuringMachine = match program.parse() {
            Ok(m) => m,
            Err((pos, err)) => {
                eprintln!("{}", banner_err);
                eprint!("{}", pos);
                eprintln!("{:?}", err);
                eprintln!("{}", banner_end);
                std::process::exit(1);
            }
        };
        arch_state = Box::new(automata::TmArchState::new(machine));
    } else {
        panic!("Unknown machine type!");
    }

    let verbose_input_err = |col: usize| {
        eprintln!("{}", banner_err);
        eprintln!(
            "error: '{}' was not declared in the ser of input symbols",
            args.input.chars().nth(col).unwrap()
        );
        eprintln!("Input: {}", args.input);
        eprintln!("       {}^", " ".repeat(col));
        eprintln!("{}", banner_end);
    };

    match arch_state.input(&args.input) {
        Ok(_) => (),
        Err(e) => match e {
            automata::Exception::Dfa(e) => {}
            automata::Exception::Pda(e) => match e {
                automata::pda::Exception::InvalidInput { col } => {
                    if args.verbose {
                        verbose_input_err(col);
                    } else {
                        eprintln!("Illegal Input");
                    }
                    std::process::exit(1);
                }
                _ => panic!(),
            },
            automata::Exception::Tm(e) => match e {
                automata::tm::Exception::InvalidInput { input: _, offset } => {
                    verbose_input_err(offset);
                    std::process::exit(1);
                }
                _ => panic!(),
            },
        },
    }

    if args.verbose {
        println!("{}", banner_run);
    }

    loop {
        if args.verbose {
            print!("{}", arch_state);
            println!("{}", banner_split);
        }
        match arch_state.step() {
            Ok(_) => (),
            Err(e) => {
                match e {
                    automata::Exception::Dfa(_e) => panic!(),
                    automata::Exception::Pda(e) => match e {
                        automata::pda::Exception::Accept => {
                            println!("true");
                        }
                        automata::pda::Exception::Reject => {
                            println!("false");
                        }
                        _ => panic!(),
                    },
                    automata::Exception::Tm(e) => match e {
                        automata::tm::Exception::Reject(s) | automata::tm::Exception::Accept(s) => {
                            if args.verbose {
                                println!("Result: {}", s);
                            } else {
                                println!("{}", s);
                            }
                        }
                        _ => panic!(),
                    },
                }
                if args.verbose {
                    println!("{}", banner_end);
                }
                std::process::exit(0);
            }
        }
    }
}
