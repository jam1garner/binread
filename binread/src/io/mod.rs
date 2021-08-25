//! A swappable version of [std::io](std::io) that works in `no_std + alloc` environments.
//! If the feature flag `std` is enabled (as it is by default), this will just re-export types from `std::io`.

pub mod error;
pub mod prelude;

#[cfg(any(not(feature = "std"), test))]
pub mod cursor;

#[cfg(any(not(feature = "std"), test))]
mod no_std;

#[cfg(not(feature = "std"))]
pub use no_std::*;

#[cfg(feature = "std")]
pub use std::io::{Bytes, Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};

pub trait StreamPosition {
    fn stream_pos(&mut self) -> Result<u64>;
}

impl<T: Seek> StreamPosition for T {
    #[rustversion::before(1.51)]
    #[cfg(feature = "std")]
    fn stream_pos(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }

    // would prefer any(since(1.51), not(feature = "std"))
    // but i don't know how to compose rustversion and cfg like that
    #[rustversion::since(1.51)]
    #[cfg(feature = "std")]
    fn stream_pos(&mut self) -> Result<u64> {
        self.stream_position()
    }

    #[cfg(not(feature = "std"))]
    fn stream_pos(&mut self) -> Result<u64> {
        self.stream_position()
    }
}
