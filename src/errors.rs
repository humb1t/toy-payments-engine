use std::io;

use derive_more::{Display, Error, From};

#[derive(Debug, Error, Display, From)]
pub enum Error {
    TransactionRead(io::Error),
    Database(redb::Error),
    Programmer(ProgrammerError),
    #[display(
        "incorrect math calculation, like attempt to substract with overflow, code changes may be needed"
    )]
    Calculation,
    // TODO: This type should be breaken down into many subtypes, but didn't for the simplicity of current scope
    #[display("incorrect input data, like attempt to withdraw with insufficient balance")]
    Input,
    #[display(
        "incorrect state data, some inconsistent changes were made, database manipulations may be needed"
    )]
    State,
}

#[derive(Debug, Error, Display, From)]
pub enum ProgrammerError {
    WrongCallForConvertation,
}
