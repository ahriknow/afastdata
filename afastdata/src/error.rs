/// afastdata 的核心错误类型。
///
/// Core error type for afastdata.
///
/// Encapsulates serialization, deserialization, and validation failures.
pub struct Error {
    inner: ErrorKind,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            ErrorKind::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            ErrorKind::DeserializeError(msg) => write!(f, "Deserialize error: {}", msg),
            ErrorKind::ValidateError(code, msg) => {
                write!(f, "Validation error: code {} is invalid: {}", code, msg)
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            ErrorKind::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            ErrorKind::DeserializeError(msg) => write!(f, "Deserialize error: {}", msg),
            ErrorKind::ValidateError(code, msg) => {
                write!(f, "Validation error: code {} is invalid: {}", code, msg)
            }
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    /// 创建一个序列化错误。
    ///
    /// Creates a serialization error.
    pub fn serialize(msg: String) -> Self {
        Error {
            inner: ErrorKind::SerializeError(msg),
        }
    }

    /// 创建一个反序列化错误。
    ///
    /// Creates a deserialization error.
    pub fn deserialize(msg: String) -> Self {
        Error {
            inner: ErrorKind::DeserializeError(msg),
        }
    }

    /// 创建一个验证错误，包含无效值和错误信息。
    ///
    /// Creates a validation error with the invalid value and a message.
    pub fn validate(value: i64, msg: String) -> Self {
        Error {
            inner: ErrorKind::ValidateError(value, msg),
        }
    }
}

/// 列举 afastdata 可能产生的错误类型。
///
/// Enumerates the kinds of errors produced by afastdata.
pub enum ErrorKind {
    /// 序列化失败，携带文本消息。
    ///
    /// Serialization failure with a textual message.
    SerializeError(String),

    /// 反序列化失败，携带文本消息。
    ///
    /// Deserialization failure with a textual message.
    DeserializeError(String),

    /// 验证失败，包含无效值和错误信息。
    ///
    /// Validation failure, carrying the invalid value and a message.
    ValidateError(i64, String),
}

/// 用于更丰富错误构造的验证错误类型。
///
/// Validation-specific error type used for richer error construction.
pub struct ValidateError {
    code: i64,
    message: String,
}

impl std::fmt::Display for ValidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Validation error: code {}, message {}",
            self.code, self.message
        )
    }
}

impl std::fmt::Debug for ValidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Validation error: code {}, message {}",
            self.code, self.message
        )
    }
}

impl std::error::Error for ValidateError {}

impl ValidateError {
    /// 创建一个新的 `ValidateError`。
    ///
    /// Creates a new `ValidateError`.
    pub fn new(code: i64, message: String) -> Self {
        ValidateError { code, message }
    }

    /// 将此验证错误转换为通用 `Error` 类型。
    ///
    /// Converts this validation error into the generic `Error` type.
    pub fn to_afastdata_error(&self) -> Error {
        Error::validate(self.code, self.message.clone())
    }
}
