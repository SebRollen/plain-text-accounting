use plain_text_accounting::transaction;
use std::fs::File;
use std::io::prelude::*;
fn main() -> std::io::Result<()> {
    let mut file = File::open("journal.ledger")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let (_, t) = transaction(&contents).unwrap();
    println!("{:#?}", t);
    Ok(())
}
