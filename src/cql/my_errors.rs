use std::fmt;
use std::io;
use std::error::Error;

#[derive(Debug)]
pub enum MyError<'b> {
    Error(&'b str),
    InvalidTable(&'b str),
    InvalidColumn(&'b str),
    InvalidSyntax(&'b str),
    TestError(&'b str),
}

impl<'b> fmt::Display for MyError<'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {   
        match *self {
            MyError::Error(ref msg) => write!(f, "[ERROR]: [{}]", msg),
            MyError::InvalidTable(ref msg) => write!(f, "[INVALID_TABLE]: [{}]", msg),
            MyError::InvalidColumn(ref msg) => write!(f, "[INVALID_COLUMN]: [{}]", msg),
            MyError::InvalidSyntax(ref msg) => write!(f, "[INVLAID_SYNTAX]: [{}]", msg),
            MyError::TestError(ref msg) => write!(f, "[TEST_ERROR]: [{}]", msg),
        }
    }
}

impl<'b> From<MyError<'b>> for io::Error {
    fn from(error: MyError<'b>) -> io::Error {
        io::Error::new(io::ErrorKind::Other, error.to_string())
    }
}

impl<'b> PartialEq for MyError<'b> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MyError::Error(a), MyError::Error(b)) => a == b,
            (MyError::InvalidTable(a), MyError::InvalidTable(b)) => a == b,
            (MyError::InvalidColumn(a), MyError::InvalidColumn(b)) => a == b,
            (MyError::InvalidSyntax(a), MyError::InvalidSyntax(b)) => a == b,
            (MyError::TestError(a), MyError::TestError(b)) => a == b,
            _ => false,
        }
    }
}

impl Error for MyError<'_> {}