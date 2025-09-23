pub mod error;
use std::result::Result;

pub type Res<T> = Result<T, error::Error>;
