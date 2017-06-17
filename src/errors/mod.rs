use std::error::Error;
use std::fmt;
#[derive(Debug)]
pub struct ConflictError {
    uuid: String,
}

impl ConflictError {
    pub fn new(uuid: String) -> ConflictError {
        ConflictError { uuid: uuid }
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

#[derive(Debug)]
pub struct NotFoundError {
    uuid: String,
}
impl NotFoundError {
    pub fn new(uuid: String) -> NotFoundError {
        NotFoundError { uuid: uuid }
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
