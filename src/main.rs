mod exec;
mod tm;

use clap::Parser;
use std::io::Read;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    verbose: bool,
    #[arg(id = "tm")]
    program: String,
    #[arg(id = "input")]
    input: String,
}

fn main() {
    let args = Arguments::parse();
    eprintln!("{:?}", args);

    let mut program = String::new();
    std::fs::File::open(&args.program)
        .expect("failed to open <tm> description")
        .read_to_string(&mut program)
        .expect("failed to read <tm> description");

    fn banner(s: &str) -> String {
        "=".repeat(20) + " " + s + " " + &("=".repeat(20))
    }
    let banner_run = banner("RUN");
    let banner_err = banner("ERR");
    let banner_end = banner("END");
    let banner_split = "-".repeat(45);

    let machine = match program.parse::<tm::TuringMachine>() {
        Ok(m) => m,
        Err(e) => {
            if args.verbose {
                eprintln!("{}", banner_err);
                use tm::ParseErrorType::*;
                match e.error {
                    SpecError => eprintln!("spec error"),
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
            std::process::exit(exitcode::DATAERR);
        }
    };

    println!("Input: {}", args.input);

    let mut arch_state = exec::ArchState::new(machine);
    match arch_state.input(&args.input) {
        Ok(_) => (),
        Err(e) => match e {
            exec::Exception::InvalidInput { input, offset } => {
                eprintln!("{}", banner_err);
                eprintln!(
                    "error: '{}' was not declared in the set of input symbols",
                    input.chars().nth(offset).unwrap()
                );
                eprintln!("Input: {}", input);
                eprintln!("{}^", " ".repeat(7 + offset));
                eprintln!("{}", banner_end);
                std::process::exit(exitcode::DATAERR);
            }
            _ => panic!(),
        },
    }

    println!("{}", banner_run);
    loop {
        print!("{}", arch_state);
        println!("{}", banner_split);

        use exec::Exception::*;
        match arch_state.step() {
            Ok(_) => {
            }
            Err(e) => {
                match e {
                    Halt => {
                        println!("error: halt!");
                    }
                    Accept => {
                        println!("Result: {}", arch_state.result().unwrap());
                    }
                    _ => panic!()
                }
                println!("{}", banner_end);
            }
        }
    }
}
