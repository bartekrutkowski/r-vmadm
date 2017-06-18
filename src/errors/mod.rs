use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct GenericError {
    msg: String,
}

impl GenericError {
    pub fn new(msg: &str) -> GenericError {
        GenericError { msg: String::from(msg) }
    }
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

#[derive(Debug)]
pub struct ConflictError {
    uuid: String,
}

impl ConflictError {
    pub fn new(uuid: &str) -> ConflictError {
        ConflictError { uuid: String::from(uuid) }
    }
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

#[derive(Debug)]
pub struct NotFoundError {
    uuid: String,
}
impl NotFoundError {
    pub fn new(uuid: &str) -> NotFoundError {
        NotFoundError { uuid: String::from(uuid) }
    }
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