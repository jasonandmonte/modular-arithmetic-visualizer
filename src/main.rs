use std::env;

fn main() {
    let (integer, modulus) = parse_args();
    let result =  integer % modulus;
    // TODO: Add --help flag

    println!("{} mod {} = {}", integer, modulus, result)
}

fn parse_args() -> (u32, u32) {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <integer> <modulus>", args[0]);
        std::process::exit(1);
    }

    let integer: u32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid integer (n >= 0).", args[1]);
            std::process::exit(1);
        }
    };

    let modulus: u32 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid integer (n >= 0).", args[2]);
            std::process::exit(1);
        }
    };

    (integer, modulus)
}

