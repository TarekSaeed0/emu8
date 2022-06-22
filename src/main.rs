use std::fs::File;
use std::io::Read;
use std::io;

fn main() {
    let mut input = String::new();
    let mut program = Vec::new();
    loop {
        println!("input a path to chip-8 rom");
        io::stdin().read_line(&mut input).expect("failed to read input");
        match File::open(&input.trim()) {
            Ok(mut file) => {
                file.read_to_end(&mut program).expect("failed to read program from file");
                pollster::block_on(emu8::run(&program));
                program.clear();
            },
            Err(error) => {
                eprintln!("failed to read error: {}", error);
            }
        }
        input.clear();
    }
}
