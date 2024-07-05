use std::io::{self, Write};

pub fn get_input() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn echo(input: &str) -> String {
    format!("You said: {:?}", input)
}

fn main() -> io::Result<()> {
    print!("Enter your input: ");
    io::stdout().flush()?; // flush the output buffer

    match get_input() {
        Ok(input) => println!("{}", echo(&input)),
        Err(e) => eprintln!("Failed to read input: {}", e)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo() {
        assert_eq!(echo("test"), "You said: \"test\"");
    }

    #[test]
    fn test_echo_should_fail() {
        assert_ne!(echo("test"), "You said: \"mail\"");
    }
}