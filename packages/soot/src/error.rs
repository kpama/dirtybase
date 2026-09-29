use std::{collections::BTreeMap, fmt};

use dirtybase_db::field_values::FieldValue;

/// The kind of failure that occurred.
///
/// Mirrors the taxonomy Ash uses to decide whether a failure is the caller's
/// fault, the developer's fault, or an infrastructure problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorClass {
    /// Something went wrong that the caller cannot fix. A bug.
    Framework,
    /// The changeset as a whole could not be processed.
    Changeset,
    /// A value failed a constraint or comparison.
    Invalid,
    /// A required value was not supplied.
    Required,
    /// Input referenced an attribute the resource does not declare.
    UnknownField,
    /// A required action argument was not supplied.
    ActionInputRequired,
    /// An optimistic-lock version did not match the stored row.
    StaleRecord,
    /// An operation requiring a primary key ran against a resource without one.
    NoPrimaryKey,
}

impl ErrorClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Framework => "framework",
            Self::Changeset => "changeset",
            Self::Invalid => "invalid",
            Self::Required => "required",
            Self::UnknownField => "unknown_field",
            Self::ActionInputRequired => "action_input_required",
            Self::StaleRecord => "stale_record",
            Self::NoPrimaryKey => "no_primary_key",
        }
    }
}

impl fmt::Display for ErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single error produced by an action.
///
/// `field_path` locates the offending value. For a top level attribute it is
/// `["title"]`; for a value nested inside a map attribute it is
/// `["profile", "email"]`. An empty path means the error is about the action
/// as a whole.
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    class: ErrorClass,
    field_path: Vec<String>,
    message: String,
    code: Option<String>,
    vars: BTreeMap<String, FieldValue>,
}

impl Error {
    pub fn new<M: Into<String>>(class: ErrorClass, message: M) -> Self {
        Self {
            class,
            field_path: Vec::new(),
            message: message.into(),
            code: None,
            vars: BTreeMap::new(),
        }
    }

    /// An error attached to a specific attribute, e.g. "is invalid".
    pub fn invalid<M: Into<String>>(field: &str, message: M) -> Self {
        Self::new(ErrorClass::Invalid, message).at(field)
    }

    /// A required attribute was not supplied.
    pub fn required<M: Into<String>>(field: &str, message: M) -> Self {
        Self::new(ErrorClass::Required, message).at(field)
    }

    pub fn unknown_field<M: Into<String>>(field: &str, message: M) -> Self {
        Self::new(ErrorClass::UnknownField, message).at(field)
    }

    pub fn action_input_required<M: Into<String>>(field: &str, message: M) -> Self {
        Self::new(ErrorClass::ActionInputRequired, message).at(field)
    }

    pub fn stale_record<M: Into<String>>(field: &str, message: M) -> Self {
        Self::new(ErrorClass::StaleRecord, message).at(field)
    }

    pub fn framework<M: Into<String>>(message: M) -> Self {
        Self::new(ErrorClass::Framework, message)
    }

    pub fn changeset<M: Into<String>>(message: M) -> Self {
        Self::new(ErrorClass::Changeset, message)
    }

    pub fn no_primary_key<M: Into<String>>(message: M) -> Self {
        Self::new(ErrorClass::NoPrimaryKey, message)
    }

    /// Anchor the error at a field, replacing any existing path.
    pub fn at(mut self, field: &str) -> Self {
        self.field_path = vec![field.to_string()];
        self
    }

    /// Append a segment to the field path, for nested values.
    pub fn path(mut self, segment: &str) -> Self {
        self.field_path.push(segment.to_string());
        self
    }

    pub fn with_code(mut self, code: &str) -> Self {
        self.code = Some(code.to_string());
        self
    }

    pub fn with_var<V: Into<FieldValue>>(mut self, key: &str, value: V) -> Self {
        self.vars.insert(key.to_string(), value.into());
        self
    }

    pub fn class(&self) -> ErrorClass {
        self.class
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    pub fn field_path(&self) -> &[String] {
        &self.field_path
    }

    /// The first path segment, which is the attribute name for attribute errors.
    pub fn field(&self) -> Option<&str> {
        self.field_path.first().map(|s| s.as_str())
    }

    pub fn vars(&self) -> &BTreeMap<String, FieldValue> {
        &self.vars
    }

    /// Whether this error should make the whole action fail.
    pub fn is_invalid(&self) -> bool {
        matches!(
            self.class,
            ErrorClass::Invalid
                | ErrorClass::Required
                | ErrorClass::UnknownField
                | ErrorClass::ActionInputRequired
                | ErrorClass::StaleRecord
                | ErrorClass::Changeset
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.field_path.is_empty() {
            write!(f, "{}: {}", self.class, self.message)
        } else {
            write!(
                f,
                "{}: {} ({})",
                self.class,
                self.message,
                self.field_path.join(".")
            )
        }
    }
}

impl std::error::Error for Error {}

/// Collects errors so an action can report every problem it found rather than
/// stopping at the first.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ErrorList {
    errors: Vec<Error>,
}

impl ErrorList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Absorb another batch of errors, so a validation that collects several
    /// problems can be merged into a caller's list. Accepts either shape, since
    /// a single error can arrive as `Error` or wrapped in `Errors`.
    pub fn add_errors(&mut self, other: impl Into<Errors>) {
        self.errors.extend(Into::<Errors>::into(other).0);
    }

    /// Absorb the errors out of a failed result, leaving its success value
    /// alone.
    pub fn add_result<T>(&mut self, result: std::result::Result<T, Errors>) {
        if let Err(errors) = result {
            self.errors.extend(errors.0);
        }
    }

    pub fn from_vec(errors: Vec<Error>) -> Self {
        Self { errors }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// The first error, which is the one to report in a summary log line.
    pub fn first(&self) -> Option<&Error> {
        self.errors.first()
    }

    pub fn has_class(&self, class: ErrorClass) -> bool {
        self.errors.iter().any(|e| e.class() == class)
    }

    pub fn into_result(self) -> std::result::Result<(), Errors> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(Errors(self.errors))
        }
    }
}

impl From<Error> for ErrorList {
    fn from(error: Error) -> Self {
        Self {
            errors: vec![error],
        }
    }
}

impl From<ErrorList> for Errors {
    fn from(list: ErrorList) -> Self {
        Self(list.errors)
    }
}

/// The error an action returns when it accumulated one or more [`Error`]s.
#[derive(Debug, Clone, PartialEq)]
pub struct Errors(pub Vec<Error>);

impl Errors {
    /// The errors a failed action collected.
    pub fn new(errors: Vec<Error>) -> Self {
        Self(errors)
    }

    pub fn first(&self) -> Option<&Error> {
        self.0.first()
    }

    pub fn errors(&self) -> &[Error] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn has_class(&self, class: ErrorClass) -> bool {
        self.0.iter().any(|error| error.class() == class)
    }
}

impl From<Error> for Errors {
    fn from(error: Error) -> Self {
        Self(vec![error])
    }
}

impl From<Vec<Error>> for Errors {
    fn from(errors: Vec<Error>) -> Self {
        Self(errors)
    }
}

impl From<Errors> for Vec<Error> {
    fn from(errors: Errors) -> Self {
        errors.0
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Errors {}

pub type Result<T> = std::result::Result<T, Errors>;

/// Bridges an infrastructure failure into the action error taxonomy so that a
/// database error surfaces as a framework error rather than an unhandled panic.
pub fn from_db(context: &str, error: impl std::fmt::Display) -> Errors {
    Errors(vec![Error::framework(format!("{context}: {error}"))])
}
