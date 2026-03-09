use std::io;

use derive_more::{Display, Error, From};

#[derive(Debug, Error, Display, From)]
pub enum Error {
    TransactionRead(io::Error),
    Database(redb::Error),
    Calculation,
    Programmer(ProgrammerError),
}

#[derive(Debug, Error, Display, From)]
pub enum ProgrammerError {
    WrongCallForConvertation,
}
