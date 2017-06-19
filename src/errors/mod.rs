//! Errors for vmadm

use std::error::Error;
use std::fmt;

/// Generic error that carries a message string
#[derive(Debug)]
pub struct GenericError {
    msg: String,
}

impl GenericError {
    /// Initialize a new generic error
    pub fn new(msg: &str) -> GenericError {
        GenericError { msg: String::from(msg) }
    }
    /// Create a new error in a box
    pub fn bx(msg: &str) -> Box<Error> {
        Box::new(GenericError::new(msg))
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for GenericError {
    fn description(&self) -> &str {
        "Generic Error"
    }
}

/// Conflict error when a uuid is re-used
#[derive(Debug)]
pub struct ConflictError {
    uuid: String,
}

impl ConflictError {
    /// Initialize a new conflict error
    pub fn new(uuid: &str) -> ConflictError {
        ConflictError { uuid: String::from(uuid) }
    }
    /// Initialize a new conflict error in side a box
    pub fn bx(uuid: &str) -> Box<Error> {
        Box::new(ConflictError::new(uuid))
    }
}

impl fmt::Display for ConflictError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Duplicated UUID: {}", self.uuid)
    }
}

impl Error for ConflictError {
    fn description(&self) -> &str {
        "Conflict"
    }
}

/// Conflict error when a uuid is not found
#[derive(Debug)]
pub struct NotFoundError {
    uuid: String,
}
impl NotFoundError {
    /// Initialize a new conflict error
    pub fn new(uuid: &str) -> NotFoundError {
        NotFoundError { uuid: String::from(uuid) }
    }
    /// Initialize a new conflict error in side a box
    pub fn bx(uuid: &str) -> Box<Error> {
        Box::new(NotFoundError::new(uuid))
    }
}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UUID not found: {}", self.uuid)
    }
}

impl Error for NotFoundError {
    fn description(&self) -> &str {
        "Not Found"
    }
}