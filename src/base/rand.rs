//! Is version-4 based on Truly Random or Pseudo-Random Numbers.

use rand;

use crate::*;

impl Uuid {
    /// Generate a UUID from truly random numbers.
    pub fn v4() -> Layout {
        let rng = rand::random::<u128>();
        let rand = rng.to_be_bytes();

        Layout {
            time_low: ((rand[0] as u32) << 24)
                | (rand[1] as u32) << 16
                | (rand[2] as u32) << 8
                | rand[3] as u32,
            time_mid: (rand[4] as u16) << 8 | (rand[5] as u16),
            time_high_and_version: ((rand[6] as u16) << 8 | (rand[7] as u16)) & 0xfff
                | (Version::RAND as u16) << 12,
            clock_seq_high_and_reserved: (rand[8] & 0xf) | (Variant::RFC as u8) << 4,
            clock_seq_low: rand[9] as u8,
            node: [rand[10], rand[11], rand[12], rand[13], rand[14], rand[15]],
        }
    }
}

#[macro_export]
macro_rules! uuid_v4 {
    () => {
        format!("{:x}", $crate::Uuid::v4().as_bytes())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v4() {
        let uuid = Uuid::v4();

        assert_eq!(uuid.get_version(), Some(Version::RAND));
        assert_eq!(uuid.get_variant(), Some(Variant::RFC));

        assert!(Uuid::is_valid(&format!("{:x}", uuid.as_bytes())));
        assert!(Uuid::is_valid(&format!("{:X}", uuid.as_bytes())));
    }
}
