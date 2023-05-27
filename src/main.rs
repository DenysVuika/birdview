use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let query = &args[1];
    let input_file = &args[2];

    println!("Searching for {}", query);
    println!("In file {}", input_file);

    let contents = fs::read_to_string(input_file)
        .expect("Should have been able to read file");

    println!("With text:\n{contents}");
}
