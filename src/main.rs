use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut program = Vec::new();
    if let [_, arg] = &args[..] {
        let mut file = File::open(arg).expect("failed to open file");
        file.read_to_end(&mut program).expect("failed to read program from file");
    } else {
        panic!("expected a single arg but found: {:?}", args);
    }
    pollster::block_on(emu8::run(&program));
}
