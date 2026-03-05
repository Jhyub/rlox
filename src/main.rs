use rlox::vm::{InterpretResult, VM};

use std::io::Write;

fn main() {
    let mut vm = VM::new();

    let args = std::env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        repl(&mut vm);
    } else if args.len() == 2 {
        run_file(&mut vm, &args[1]);
    } else {
        eprintln!("Usage: rlox [path]");
        std::process::exit(64);
    }
}

fn repl(vm: &mut VM) {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            println!();
            break;
        }
        let _ = vm.interpret(input);
    }
}

fn run_file(vm: &mut VM,path: &str) {
    let contents = std::fs::read_to_string(path).unwrap();
    let result = vm.interpret(&contents);
    if result == InterpretResult::CompileError {
        std::process::exit(65);
    }
    if result == InterpretResult::RuntimeError {
        std::process::exit(70);
    }
}
