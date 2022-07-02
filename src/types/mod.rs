mod bytes;
mod codes;
mod integer;
mod string;

pub use self::codes::ReasonCode;
pub use self::integer::VariableByteInteger;
pub use self::string::UTF8String;

/// Didn't know what else to call it :)
#[derive(Debug, PartialEq, PartialOrd)]
pub enum YesNoMaybe {
    None,
    Required,
    Optional,
}

/// Right now just a marker trait.
pub trait MqttDataType {}