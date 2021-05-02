use core::result;

use crate::error;

pub type Result<R> = result::Result<R, error::Error>;
