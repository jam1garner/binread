//! An enum to represent what endianness to read as

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// An enum to represent what endianness to read as
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Endian {
    Big,
    Little,
    Native,
}

pub use Endian::{Big as BE, Little as LE, Native as NE};

impl From<&Endian> for String {
    fn from(endian: &Endian) -> String {
        String::from(match endian {
            Endian::Big => "Big",
            Endian::Little => "Little",
            Endian::Native => "Native",
        })
    }
}

impl Default for Endian {
    fn default() -> Endian {
        Endian::Native
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endian_to_string() {
        for &(ref endian, string) in [
            (Endian::Big, "Big"),
            (Endian::Little, "Little"),
            (Endian::Native, "Native"),
        ]
        .iter()
        {
            let converted: String = endian.into();
            assert_eq!(converted, string)
        }
    }
}
