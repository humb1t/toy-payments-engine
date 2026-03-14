use std::io;

use derive_more::{Display, Error, From};

#[derive(Debug, Error, Display, From)]
pub enum Error {
    #[from]
    Io(io::Error),
    #[from]
    Csv(csv::Error),
    Programmer(ProgrammerError),
    #[display(
        "incorrect math calculation, like attempt to substract with overflow, code changes may be needed"
    )]
    Calculation,
    #[display("incorrect input data, like attempt to withdraw with insufficient balance")]
    Input,
    #[display("incorrect state data, some inconsistent changes were made")]
    State,
}

#[derive(Debug, Error, Display, From)]
pub enum ProgrammerError {
    WrongCallForConvertation,
}
