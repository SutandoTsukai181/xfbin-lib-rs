use std::error;

use strum_macros::Display;

#[derive(Debug, Display)]
pub enum NuccError {
    GenericError,
}

impl error::Error for NuccError {}
