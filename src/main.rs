use std::io::{self, Write};

pub fn get_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

pub fn echo(input: &str) -> String {
    format!("You said: '{}'", input)
}

fn main() {
    // Ask the user for input
    print!("Enter your input: ");
    io::stdout().flush().unwrap(); // flush the output buffer

    let input = get_input();

    // Print the result of echo
    println!("{}", echo(&input));
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo() {
        assert_eq!(echo("test"), "You said: 'test'");
    }
    #[test]
    fn test_echo_should_fail() {
        assert_ne!(echo("test"), "You said: 'mail'");
    }
}