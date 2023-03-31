use std::fs::OpenOptions;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    // Open the file in append mode
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("./test.txt")?;

    // Write the string to the file
    writeln!(file, "\nHello, World!")?;

    Ok(())
}