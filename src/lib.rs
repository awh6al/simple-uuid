use core::{fmt, str, sync::atomic};
use std::time::{self, SystemTime};

use mac_address::get_mac_address;
use rand;
use regex::Regex;

pub const NANO_UTC_EPOCH: u64 = 0x1B21_DD21_3814_000;

#[derive(Debug)]
pub enum Format {
    Layout,
    Variant,
    Version,
    TimeStamp,
    ClockSeq,
    Node,
}

#[derive(Debug)]
pub struct Layout {
    pub time_low: u32,
    pub time_mid: u16,
    pub time_high_and_version: u16,
    pub clock_seq_high_and_reserved: u8,
    pub clock_seq_low: u8,
    pub node: [u8; 6],
}

impl Layout {
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

    pub fn as_bytes(&self) -> Uuid {
        Uuid([
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

    pub fn get_variant(&self) -> Option<Variant> {
        match (self.clock_seq_high_and_reserved >> 4) & 0xf {
            0x00 => Some(Variant::NCS),
            0x01 => Some(Variant::RFC),
            0x02 => Some(Variant::MS),
            0x03 => Some(Variant::FUT),
            _ => None,
        }
    }

    pub fn get_time(&self) -> Timestamp {
        let time = (self.time_high_and_version as u64 & 0xfff) << 48
            | (self.time_mid as u64) << 32
            | self.time_low as u64;
        Timestamp(time)
    }
}

#[derive(Debug)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn new() -> u64 {
        let nano = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .checked_add(std::time::Duration::from_nanos(NANO_UTC_EPOCH))
            .unwrap()
            .as_nanos();

        (nano & 0xffff_ffff_ffff_ffff) as u64
    }

    pub fn duration(&self) -> time::Duration {
        time::Duration::from_nanos(self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Variant {
    NCS = 0,
    RFC,
    MS,
    FUT,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Version {
    TIME = 1,
    DCE,
    MD5,
    RAND,
    SHA1,
}

pub enum Domain {
    PERSON = 0,
    GROUP,
    ORG,
}

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

#[derive(Clone, Copy)]
pub struct ClockSeq(u16);

impl ClockSeq {
    pub fn new(r: u16) -> u16 {
        atomic::AtomicU16::new(r).fetch_add(1, atomic::Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub struct Uuid([u8; 16]);

impl Uuid {
    pub fn v1() -> Layout {
        let utc = Timestamp::new();
        let clock_seq = ClockSeq::new(rand::random::<u16>());

        Layout {
            time_low: ((utc & 0xffff_ffff) as u32),
            time_mid: ((utc >> 32 & 0xffff) as u16),
            time_high_and_version: (utc >> 48 & 0xfff) as u16 | (Version::TIME as u16) << 12,
            clock_seq_high_and_reserved: ((clock_seq >> 8) & 0xf) as u8 | (Variant::RFC as u8) << 4,
            clock_seq_low: (clock_seq & 0xff) as u8,
            node: get_mac_address().unwrap().unwrap().bytes(),
        }
    }

    pub fn v2(domain: Domain) -> Layout {
        let utc = Timestamp::new();
        let clock_seq = ClockSeq::new(rand::random::<u16>());
        let local_id = ((utc & 0xffff_ffff) as u32) | 1000;

        Layout {
            time_low: local_id,
            time_mid: ((utc >> 32 & 0xffff) as u16),
            time_high_and_version: (utc >> 48 & 0xfff) as u16 | (Version::DCE as u16) << 12,
            clock_seq_high_and_reserved: ((clock_seq >> 8) & 0xf) as u8 | (Variant::RFC as u8) << 4,
            clock_seq_low: domain as u8,
            node: get_mac_address().unwrap().unwrap().bytes(),
        }
    }

    pub fn is_valid(s: &str) -> bool {
        let regex = Regex::new(
            r"^(?i)(urn:uuid:)?[0-9a-f]{8}\-[0-9a-f]{4}\-[0-5]{1}[0-9a-f]{3}\-[0-9a-f]{4}\-[0-9a-f]{12}$",
        );
        regex.unwrap().is_match(s)
    }
}

impl fmt::LowerHex for Uuid {
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

impl fmt::UpperHex for Uuid {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v1() {
        let uuid = Uuid::v1();

        assert_eq!(uuid.get_version(), Some(Version::TIME));
        assert_eq!(uuid.get_variant(), Some(Variant::RFC));

        assert!(Uuid::is_valid(&format!("{:x}", uuid.as_bytes())));
        assert!(Uuid::is_valid(&format!("{:X}", uuid.as_bytes())));
    }

    #[test]
    fn test_v2() {
        let uuid = Uuid::v2(Domain::PERSON);

        assert_eq!(uuid.get_version(), Some(Version::DCE));
        assert_eq!(uuid.get_variant(), Some(Variant::RFC));

        assert!(Uuid::is_valid(&format!("{:x}", uuid.as_bytes())));
        assert!(Uuid::is_valid(&format!("{:X}", uuid.as_bytes())));
    }

    #[test]
    fn test_valid_uuid() {
        let uuid = [
            "urn:uuid:7370554e-8a0c-11ea-bc55-0242ac130003",
            "0c0bf838-9388-11ea-bb37-0242ac130002",
        ];

        for id in uuid.iter() {
            assert!(Uuid::is_valid(id))
        }
    }

    #[test]
    fn test_valid_upper_uuid() {
        let uuid = [
            "urn:uuid:7370554e-8a0c-11ea-bc55-0242ac130003",
            "0c0bf838-9388-11ea-bb37-0242ac130002",
        ];

        for id in uuid.iter() {
            assert!(Uuid::is_valid(&id.to_ascii_uppercase()))
        }
    }

    #[test]
    #[should_panic]
    fn test_invalid_uuid() {
        let uuid = [
            "urn:uuid:7370554e-8a0c-11ea-bc55-0242ac130003_invalid",
            "0c0bf838-9388-11ea-bb37-0242ac130002_invalid",
            "0c0bf838-9388-61ea-bb37-0242ac130002", // invalid version
        ];

        for id in uuid.iter() {
            assert!(Uuid::is_valid(id))
        }
    }

    #[test]
    fn test_node() {
        let node = Node([00, 42, 53, 13, 19, 128]);
        assert_eq!(format!("{:x}", node), "00-2a-35-0d-13-80");
        assert_eq!(format!("{:X}", node), "00-2A-35-0D-13-80")
    }
}
