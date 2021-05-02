use std::error;
use std::fmt;
use std::ops::Deref;

#[derive(Debug)]
pub enum Error {
    Simple(String),
    Cause(String, Box<dyn error::Error>),
}

impl From<Box<dyn error::Error>> for Error {
    fn from(error: Box<dyn error::Error>) -> Self {
        Error::Cause(String::from("an error occurred"), error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Simple(ref m) => writeln!(f, "{}", m),
            Error::Cause(ref m, ref e) => writeln!(f, "{}: {}", m, e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::Simple(_) => None,
            Error::Cause(_, ref e) => Some(e.deref()),
        }
    }
}
