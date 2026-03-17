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

    let parallel_mode = args.iter().any(|arg| arg == "--parallel");
    let input_path = if parallel_mode {
        args.iter()
            .find(|arg| !arg.starts_with("--") && *arg != &args[0])
            .ok_or(Error::Input)?
    } else {
        &args[1]
    };

    let file = File::open(input_path)?;
    let reader = ReaderBuilder::new().trim(Trim::All).from_reader(BufReader::new(file));

    let state = if parallel_mode {
        let transactions: Vec<_> = TransactionIterator::new(reader)?.collect::<Result<_, _>>()?;
        //TODO: replace with parallelism
        let shard_count = num_cpus::get().max(1);
        toy_payments_engine::process_transactions_parallel(transactions, shard_count)?
    } else {
        let mut state = State::default();
        toy_payments_engine::process_transactions(TransactionIterator::new(reader)?, &mut state)?;
        state
    };

    let accounts = state.accounts.values();
    let mut writer = Writer::from_writer(io::stdout());
    for account in accounts {
        writer.serialize(account)?;
    }
    writer.flush()?;
    Ok(())
}
