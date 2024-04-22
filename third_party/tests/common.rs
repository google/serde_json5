use serde_json5::{Error, Location};

#[allow(unused)]
pub fn deserializes_to<'a, T>(s: &'a str, v: T)
where
    T: ::std::fmt::Debug + ::std::cmp::PartialEq + serde::de::Deserialize<'a>,
{
    assert_eq!(serde_json5::from_str::<T>(s), Ok(v));
}

#[allow(unused)]
pub fn deserializes_to_nan_f32(s: &str) {
    assert!(serde_json5::from_str::<f32>(s).unwrap().is_nan());
}

#[allow(unused)]
pub fn deserializes_to_nan_f64(s: &str) {
    assert!(serde_json5::from_str::<f64>(s).unwrap().is_nan());
}

#[allow(unused)]
pub fn deserializes_with_error<'a, T>(s: &'a str, error_expected: Error)
where
    T: ::std::fmt::Debug + ::std::cmp::PartialEq + serde::de::Deserialize<'a>,
{
    assert_eq!(serde_json5::from_str::<T>(s), Err(error_expected));
}

#[allow(unused)]
pub fn make_error(msg: impl Into<String>, line: usize, column: usize) -> Error {
    Error::Message {
        msg: msg.into(),
        location: Some(Location { line, column }),
    }
}
