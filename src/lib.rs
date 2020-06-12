//! This crate defines a uniform resource name namespace for UUIDs
//! (Universally Unique IDentifier), also known as GUIDs (Globally
//! Unique Identifier). A UUID is 128 bits long, and can guarantee
//! uniqueness across space and time.
//!
//! To activate various features, use syntax like:
//!
//! ```toml
//! [dependencies]
//! uuid = { version = "0.4.0", features = ["randy"] }
//! ```
//!
//! ```rust
//! use uuid_rs::uuid_v4;
//!
//! fn main() {
//!     println!("{}", uuid_v4!());
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/uuid-rs")]

mod name;
mod rand;
mod time;

use core::fmt;
use core::sync::atomic;
use std::time::SystemTime;

/// Is 100-ns ticks between UNIX and UTC epochs.
pub const UTC_EPOCH: u64 = 0x1B21_DD21_3814_000;

/// The UUID format is 16 octets.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Layout {
    /// The low field of the Timestamp.
    pub time_low: u32,
    /// The mid field of the Timestamp.
    pub time_mid: u16,
    /// The high field of the Timestamp multiplexed with the version number.
    pub time_high_and_version: u16,
    /// The high field of the ClockSeq multiplexed with the variant.
    pub clock_seq_high_and_reserved: u8,
    /// The low field of the ClockSeq.
    pub clock_seq_low: u8,
    /// IEEE 802 MAC address.
    pub node: [u8; 6],
}

impl Layout {
    /// Returns the four field values of the UUID in big-endian order.
    pub fn as_fields(&self) -> (u32, u16, u16, u16, u64) {
        (
            self.time_low,
            self.time_mid,
            self.time_high_and_version,
            ((self.clock_seq_high_and_reserved as u16) << 8) | self.clock_seq_low as u16,
            (self.node[0] as u64) << 40
                | (self.node[1] as u64) << 32
                | (self.node[2] as u64) << 24
                | (self.node[3] as u64) << 16
                | (self.node[4] as u64) << 8
                | (self.node[5] as u64),
        )
    }

    /// Returns a byte slice of this UUID content.
    pub fn as_bytes(&self) -> UUID {
        UUID([
            self.time_low.to_be_bytes()[0],
            self.time_low.to_be_bytes()[1],
            self.time_low.to_be_bytes()[2],
            self.time_low.to_be_bytes()[3],
            self.time_mid.to_be_bytes()[0],
            self.time_mid.to_be_bytes()[1],
            self.time_high_and_version.to_be_bytes()[0],
            self.time_high_and_version.to_be_bytes()[1],
            self.clock_seq_high_and_reserved,
            self.clock_seq_low,
            self.node[0],
            self.node[1],
            self.node[2],
            self.node[3],
            self.node[4],
            self.node[5],
        ])
    }

    /// Get the version of the current generated UUID.
    pub fn get_version(&self) -> Option<Version> {
        match (self.time_high_and_version >> 12) & 0xf {
            0x01 => Some(Version::TIME),
            0x02 => Some(Version::DCE),
            0x03 => Some(Version::MD5),
            0x04 => Some(Version::RAND),
            0x05 => Some(Version::SHA1),
            _ => None,
        }
    }

    /// Get the variant field of the current generated UUID.
    pub fn get_variant(&self) -> Option<Variant> {
        match (self.clock_seq_high_and_reserved >> 4) & 0xf {
            0x00 => Some(Variant::NCS),
            0x01 => Some(Variant::RFC),
            0x02 => Some(Variant::MS),
            0x03 => Some(Variant::FUT),
            _ => None,
        }
    }

    /// Get the time where the UUID generated in.
    pub fn get_time(&self) -> Timestamp {
        let time = (self.time_high_and_version as u64 & 0xfff) << 48
            | (self.time_mid as u64) << 32
            | self.time_low as u64;
        Timestamp(time)
    }
}

/// Domain is security-domain-relative name.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum Domain {
    PERSON = 0,
    GROUP,
    ORG,
}

/// Variant is a type field determines the layout of the UUID.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum Variant {
    /// Reserved, NCS backward compatibility.
    NCS = 0,
    /// The variant specified in rfc4122 document.
    RFC,
    /// Reserved, Microsoft Corporation backward compatibility.
    MS,
    /// Reserved for future definition.
    FUT,
}

/// Version represents the type of UUID, and is in the most significant 4 bits of the Timestamp.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum Version {
    /// The time-based version specified in this document.
    TIME = 1,
    /// DCE Security version, with embedded POSIX UIDs.
    DCE,
    /// The name-based version specified in rfc4122 document that uses MD5 hashing.
    MD5,
    /// The randomly or pseudo-randomly generated version specified in rfc4122 document.
    RAND,
    /// The name-based version specified in rfc4122 document that uses SHA-1 hashing.
    SHA1,
}

/// Timestamp represented by Coordinated Universal Time (UTC)
/// as a count of 100-ns intervals from the system-time.
#[derive(Debug)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Generate new 60-bit value from the system-time.
    pub fn new() -> u64 {
        let nano = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .checked_add(std::time::Duration::from_nanos(UTC_EPOCH))
            .unwrap()
            .as_nanos();
        (nano & 0xffff_ffff_ffff_fff) as u64
    }

    pub fn duration(&self) -> std::time::Duration {
        std::time::Duration::from_nanos(self.0)
    }
}

/// Is a 128-bit number used to identify information in computer systems.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct UUID([u8; 16]);

impl UUID {
    /// UUID namespace for domain name system (DNS).
    pub const NAMESPACE_DNS: Self = UUID([
        0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
        0xc8,
    ]);

    /// UUID namespace for ISO object identifiers (OIDs).
    pub const NAMESPACE_OID: Self = UUID([
        0x6b, 0xa7, 0xb8, 0x12, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
        0xc8,
    ]);

    /// UUID namespace for uniform resource locators (URLs).
    pub const NAMESPACE_URL: Self = UUID([
        0x6b, 0xa7, 0xb8, 0x11, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
        0xc8,
    ]);

    /// UUID namespace for X.500 distinguished names (DNs).
    pub const NAMESPACE_X500: Self = UUID([
        0x6b, 0xa7, 0xb8, 0x14, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
        0xc8,
    ]);
}

impl fmt::LowerHex for UUID {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5],
            self.0[6],
            self.0[7],
            self.0[8],
            self.0[9],
            self.0[10],
            self.0[11],
            self.0[12],
            self.0[13],
            self.0[14],
            self.0[15],
        )
    }
}

impl fmt::UpperHex for UUID {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5],
            self.0[6],
            self.0[7],
            self.0[8],
            self.0[9],
            self.0[10],
            self.0[11],
            self.0[12],
            self.0[13],
            self.0[14],
            self.0[15],
        )
    }
}

/// ClockSeq is used to avoid duplicates that could arise when the clock
/// is set backwards in time.
pub struct ClockSeq(u16);

impl ClockSeq {
    pub fn new(r: u16) -> u16 {
        atomic::AtomicU16::new(r).fetch_add(1, atomic::Ordering::AcqRel)
    }
}

/// the clock sequence is used to help avoid duplicates that could arise
/// when the clock is set backwards in time or if the node ID changes.
pub struct Node([u8; 6]);

impl fmt::LowerHex for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        )
    }
}

impl fmt::UpperHex for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    fn is_valid(s: &str) -> bool {
        let regex = Regex::new(
            r"^(?i)(urn:uuid:)?[0-9a-f]{8}\-[0-9a-f]{4}\-[0-5]{1}[0-9a-f]{3}\-[0-9a-f]{4}\-[0-9a-f]{12}$",
        );
        regex.unwrap().is_match(s)
    }

    #[test]
    fn test_node() {
        let node = Node([00, 42, 53, 13, 19, 128]);
        assert_eq!(format!("{:x}", node), "00-2a-35-0d-13-80");
        assert_eq!(format!("{:X}", node), "00-2A-35-0D-13-80")
    }

    #[test]
    fn test_valid_uuid() {
        let uuid = [
            uuid_v1!(),
            uuid_v2!(Domain::PERSON),
            uuid_v3!("any", UUID::NAMESPACE_URL),
            uuid_v4!(),
            uuid_v5!("any", UUID::NAMESPACE_DNS),
        ];

        for id in uuid.iter() {
            assert!(is_valid(id))
        }

        for id in uuid.iter() {
            assert!(is_valid(&id.to_ascii_uppercase()))
        }
    }

    #[test]
    fn test_get_time() {
        let uuid = UUID::v1();
        assert!(uuid.get_time().0 > 0);
    }
}
