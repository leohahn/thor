extern crate serde;

mod de;
mod error;
mod ser;

pub use de::from_bytes;
pub use error::{Error, Result};
pub use ser::to_bytes;
