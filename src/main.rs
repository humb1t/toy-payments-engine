use std::{
    fs::File,
    io::{self, BufReader},
};

use csv::{Reader, Writer};
use derive_more::{Display, Error, From};

use redb::{Database, backends::InMemoryBackend};
use toy_payments_engine::prelude::*;

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err(Error::Arguments);
    }
    let input_path = &args[1];
    let file = File::open(input_path)?;
    let mut reader = Reader::from_reader(BufReader::new(file));
    let database = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .map_err(redb::Error::from)?;
    let mut state = State::new(database);
    toy_payments_engine::process_transactions(reader.deserialize(), &mut state)?;
    let accounts = state.accounts.values();
    let mut writer = Writer::from_writer(io::stdout());
    for account in accounts {
        writer.serialize(account)?;
    }
    writer.flush()?;
    Ok(())
}

/// Possible errors of executable CLI.
/// Categories based, please add new variants based on category of errors.
#[derive(Debug, Error, Display, From)]
enum Error {
    #[display("Usage: cargo run -- sample.csv")]
    Arguments,
    Io(io::Error),
    Csv(csv::Error),
    Database(redb::Error),
    Engine(ToyPaymentsEngineError),
}
