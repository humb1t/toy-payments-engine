use std::{
    fs::File,
    io::{self, BufReader},
};

use csv::{ReaderBuilder, Trim, Writer};
use toy_payments_engine::prelude::*;

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(Error::Input);
    }
    let input_path = &args[1];
    let file = File::open(input_path)?;
    let reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(BufReader::new(file));
    let mut state = State::new();
    toy_payments_engine::process_transactions(TransactionIterator::new(reader), &mut state)?;
    let accounts = state.accounts.values();
    let mut writer = Writer::from_writer(io::stdout());
    for account in accounts {
        writer.serialize(account)?;
    }
    writer.flush()?;
    Ok(())
}
