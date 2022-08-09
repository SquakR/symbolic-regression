//! Expression tree serializer module.
use crate::settings::Settings;

pub trait Serializable<'a> {
    /// Convert a node to string with function and operator simplification.
    fn to_string(&self, settings: &'a Settings) -> String;
    /// Convert a node to json without function and operator simplification.
    fn to_json(&self) -> String;
}
