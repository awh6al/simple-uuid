#![cfg(feature = "mac")]

use mac_address as MAC;
use rand;

use crate::{ClockSeq, Domain, Layout, Node, Timestamp, Variant, Version, UUID};

impl UUID {
    /// New time-based UUID.
    pub fn new_v1() -> Option<Layout> {
        Self::from_mac(Version::TIME, Self::mac())
    }

    /// New DCE-security UUID.
    ///
    /// On a POSIX system the id should be the users UID for
    /// the Person domain and the users GID for the Group.
    pub fn new_v2(d: Domain) -> Layout {
        let utc = Timestamp::new();
        Layout {
            field_low: (utc & 0xffff_ffff) as u32,
            field_mid: ((utc >> 32 & 0xffff) as u16),
            field_high_and_version: (utc >> 48 & 0xfff) as u16 | (Version::DCE as u16) << 12,
            clock_seq_high_and_reserved: Self::clock_seq_high_and_reserved(Variant::RFC as u8).0,
            clock_seq_low: d as u8,
            node: Self::mac(),
        }
    }

    /// New UUID generated from a custom time.
    pub fn from_utc(v: Version, utc: u64) -> Option<Layout> {
        match v {
            Version::TIME | Version::DCE => {
                let clock_seq = Self::clock_seq_high_and_reserved(Variant::RFC as u8);
                Some(Layout {
                    field_low: ((utc & 0xffff_ffff) as u32),
                    field_mid: ((utc >> 32 & 0xffff) as u16),
                    field_high_and_version: (utc >> 48 & 0xfff) as u16 | (v as u16) << 12,
                    clock_seq_high_and_reserved: clock_seq.0,
                    clock_seq_low: clock_seq.1,
                    node: Self::mac(),
                })
            }
            _ => None,
        }
    }

    /// New UUID generated with a user defined MAC-address.
    pub fn from_mac(v: Version, mac: Node) -> Option<Layout> {
        match v {
            Version::TIME | Version::DCE => {
                let utc = Timestamp::new();
                let clock_seq = Self::clock_seq_high_and_reserved(Variant::RFC as u8);
                Some(Layout {
                    field_low: ((utc & 0xffff_ffff) as u32),
                    field_mid: ((utc >> 32 & 0xffff) as u16),
                    field_high_and_version: (utc >> 48 & 0xfff) as u16 | (v as u16) << 12,
                    clock_seq_high_and_reserved: clock_seq.0,
                    clock_seq_low: clock_seq.1,
                    node: mac,
                })
            }
            _ => None,
        }
    }

    fn clock_seq_high_and_reserved(s: u8) -> (u8, u8) {
        let clock_seq = ClockSeq::new(rand::random::<u16>());
        (
            ((clock_seq >> 8) & 0xf) as u8 | s << 4,
            (clock_seq & 0xff) as u8,
        )
    }

    fn mac() -> Node {
        Node(MAC::get_mac_address().unwrap().unwrap().bytes())
    }
}

/// Quick `UUID` version-1
#[macro_export]
macro_rules! v1 {
    () => {
        format!("{:x}", $crate::UUID::new_v1().unwrap().as_bytes())
    };
}

/// Quick `UUID` version-2
#[macro_export]
macro_rules! v2 {
    ($domain:expr) => {
        format!("{:x}", $crate::UUID::new_v2($domain).as_bytes())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_v1() {
        let uuid = UUID::new_v1();
        assert_eq!(uuid.unwrap().get_version(), Some(Version::TIME));

        let uuid = UUID::new_v1();
        assert_eq!(uuid.unwrap().get_variant(), Some(Variant::RFC));
    }

    #[test]
    fn new_v2() {
        let domains = [Domain::PERSON, Domain::GROUP, Domain::ORG];
        for d in domains.iter() {
            assert_eq!(UUID::new_v2(*d).get_version(), Some(Version::DCE));
            assert_eq!(UUID::new_v2(*d).get_variant(), Some(Variant::RFC));
        }
    }

    #[test]
    fn from_mac() {
        let uuid = UUID::from_mac(Version::TIME, Node([0x03, 0x2a, 0x35, 0x0d, 0x13, 0x80]));
        assert_eq!(uuid.unwrap().get_version(), Some(Version::TIME));

        let uuid = UUID::from_mac(Version::TIME, Node([0x03, 0x2a, 0x35, 0x0d, 0x13, 0x80]));
        assert_eq!(
            uuid.unwrap().get_mac().0,
            [0x03, 0x2a, 0x35, 0x0d, 0x13, 0x80]
        );
    }

    #[test]
    fn from_utc() {
        let uuid = UUID::from_utc(Version::TIME, 0x1234_u64);
        assert_eq!(uuid.unwrap().get_version(), Some(Version::TIME));

        let uuid = UUID::from_utc(Version::TIME, 0x1234_u64);
        assert_eq!(uuid.unwrap().get_time(), 0x1234_u64);
    }

    #[should_panic]
    #[test]
    fn from_utc_panic() {
        let uuid = UUID::from_utc(Version::RAND, 0x1234_u64);
        assert_eq!(uuid.unwrap().get_version(), Some(Version::RAND))
    }
}
