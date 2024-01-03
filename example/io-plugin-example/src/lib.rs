use io_plugin::{handle_doc, io_plugin};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

#[io_plugin]
#[derive(Clone)]
#[handle_doc("async `ExamplePlugin` handle")]
pub enum ExamplePlugin<T: DeserializeOwned + Serialize, U: DeserializeOwned + Serialize> {
    ///Get the name of this plugin
    GetName(U),
    SetRounding(bool, ()),
    Op(f64, f64, T),
    /// Get `usize` random bytes from the plugin - used to simulate large data transfer
    RandomBytes(usize, Vec<u8>),
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum Error {
    #[error("The result is mathematically invalid")]
    MathError,
    #[error("{0}")]
    Generic(String),
}
