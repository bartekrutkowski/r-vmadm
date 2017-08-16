//! Errors for vmadm

use std::error::Error;
use std::fmt;
use uuid::Uuid;


/// Validation errors
#[derive(Debug)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}
impl ValidationErrors {
    /// Create a new error for validations
    pub fn new(errors: Vec<ValidationError>) -> Self {
        ValidationErrors { errors }
    }
   /// Create a new error in a box
   pub fn bx(errors: Vec<ValidationError>) -> Box<Error> {
       Box::new(ValidationErrors::new(errors))
   }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut r = write!(f, "{} validaiton errors encountered", self.errors.len());
        for e in self.errors.clone() {
            r = write!(f, "\n  {}", e)
        }
        r
    }
}
impl Error for ValidationErrors {
    fn description(&self) -> &str {
        "Validations Error"
    }
}


/// Validation error for input validation
#[derive(Debug, Clone)]
pub struct ValidationError {
    field: String,
    error: String,
}

impl ValidationError {
    /// Initialize a new generic error
    pub fn new(field: &str, error: &str) -> ValidationError {
        ValidationError {
            field: String::from(field),
            error: String::from(error),
        }
    }
    // /// Create a new error in a box
    // pub fn bx(field: &str, error: &str) -> Box<Error> {
    //     Box::new(ValidationError::new(field, error))
    // }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.error)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        "Validation Error"
    }
}


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
    uuid: Uuid,
}

impl ConflictError {
    /// Initialize a new conflict error
    pub fn new(uuid: &Uuid) -> ConflictError {
        ConflictError { uuid: uuid.clone() }
    }
    /// Initialize a new conflict error in side a box
    pub fn bx(uuid: &Uuid) -> Box<Error> {
        Box::new(ConflictError::new(uuid))
    }
}

impl fmt::Display for ConflictError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Duplicated UUID: {}", self.uuid.hyphenated())
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
    uuid: Uuid,
}
impl NotFoundError {
    /// Initialize a new conflict error
    pub fn new(uuid: &Uuid) -> NotFoundError {
        NotFoundError { uuid: uuid.clone() }
    }
    /// Initialize a new conflict error in side a box
    pub fn bx(uuid: &Uuid) -> Box<Error> {
        Box::new(NotFoundError::new(uuid))
    }
}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UUID not found: {}", self.uuid.hyphenated())
    }
}

impl Error for NotFoundError {
    fn description(&self) -> &str {
        "Not Found"
    }
}
