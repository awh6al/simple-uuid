//! Is version-3 and version-5 UUIDs generated by hashing a namespace
//! identifier and name.

use md5;
use sha1::Sha1;

use crate::*;

impl UUID {
    /// Generate a UUID by hashing a namespace identifier and name uses MD5.
    pub fn v3(any: &str, ns: UUID) -> Layout {
        let data = format!("{:x}", ns) + any;
        let hash = md5::compute(&data).0;

        Layout {
            time_low: ((hash[0] as u32) << 24)
                | (hash[1] as u32) << 16
                | (hash[2] as u32) << 8
                | hash[3] as u32,
            time_mid: (hash[4] as u16) << 8 | (hash[5] as u16),
            time_high_and_version: ((hash[6] as u16) << 8 | (hash[7] as u16)) & 0xfff
                | (Version::MD5 as u16) << 12,
            clock_seq_high_and_reserved: (hash[8] & 0xf) | (Variant::RFC as u8) << 4,
            clock_seq_low: hash[9] as u8,
            node: [hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]],
        }
    }

    /// Generate a UUID by hashing a namespace identifier and name uses SHA1.
    pub fn v5(any: &str, nspace: UUID) -> Layout {
        let data = format!("{:x}", nspace) + any;
        let hash = Sha1::from(&data).digest().bytes();

        Layout {
            time_low: ((hash[0] as u32) << 24)
                | (hash[1] as u32) << 16
                | (hash[2] as u32) << 8
                | hash[3] as u32,
            time_mid: (hash[4] as u16) << 8 | (hash[5] as u16),
            time_high_and_version: ((hash[6] as u16) << 8 | (hash[7] as u16)) & 0xfff
                | (Version::SHA1 as u16) << 12,
            clock_seq_high_and_reserved: (hash[8] & 0xf) | (Variant::RFC as u8) << 4,
            clock_seq_low: hash[9] as u8,
            node: [hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]],
        }
    }
}

#[macro_export]
macro_rules! uuid_v3 {
    ($any:expr, $namespace:expr) => {
        format!("{:x}", $crate::UUID::v3($any, $namespace).as_bytes())
    };
}

#[macro_export]
macro_rules! uuid_v5 {
    ($any:expr, $namespace:expr) => {
        format!("{:x}", $crate::UUID::v5($any, $namespace).as_bytes())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v3() {
        let uuid = UUID::v3("any", UUID::NAMESPACE_X500);

        assert_eq!(uuid.get_version(), Some(Version::MD5));
        assert_eq!(uuid.get_variant(), Some(Variant::RFC));

        assert!(UUID::is_valid(&format!("{:x}", uuid.as_bytes())));
        assert!(UUID::is_valid(&format!("{:X}", uuid.as_bytes())));
    }

    #[test]
    fn test_v5() {
        let uuid = UUID::v5("any", UUID::NAMESPACE_X500);

        assert_eq!(uuid.get_version(), Some(Version::SHA1));
        assert_eq!(uuid.get_variant(), Some(Variant::RFC));

        assert!(UUID::is_valid(&format!("{:x}", uuid.as_bytes())));
        assert!(UUID::is_valid(&format!("{:X}", uuid.as_bytes())));
    }

    #[test]
    fn test_from_macro() {
        assert!(UUID::is_valid(&uuid_v3!("any", UUID::NAMESPACE_DNS)));
        assert!(UUID::is_valid(&uuid_v5!("any", UUID::NAMESPACE_OID)));
    }
}
