use std::io;
use std::io::Write;
use anyhow::Result;

fn main() -> Result<()> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    io::stdout().write_all(buffer.as_bytes())?;
    io::stderr().write_all(format!("error: {buffer}").as_bytes())?;
    Ok(())
}
