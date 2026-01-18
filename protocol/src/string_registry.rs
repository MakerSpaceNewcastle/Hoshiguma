use heapless::Vec;
use serde::{Deserialize, Serialize};

pub type String = heapless::String<32>;

#[derive(Default, Debug)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct StringRegistry {
    strings: Vec<String, 32>,
}

impl StringRegistry {
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn metadata(&self) -> Metadata {
        let crc = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
        let mut digest = crc.digest();
        for s in &self.strings {
            digest.update(s.as_bytes());
        }
        let checksum = digest.finalize();

        Metadata {
            count: self.strings.len(),
            checksum,
        }
    }

    pub fn push(&mut self, s: String) -> Result<(), String> {
        self.strings.push(s)
    }

    pub fn push_str(&mut self, s: &str) -> Result<(), String> {
        self.push(
            s.try_into()
                .expect("string is too long for string registry"),
        )
    }

    pub fn clear(&mut self) {
        self.strings.clear()
    }

    pub fn get_str(&self, idx: usize) -> Option<&str> {
        self.strings.get(idx).map(|v| &**v)
    }

    pub fn get_index(&self, s: &str) -> Option<usize> {
        self.strings.iter().position(|v| v == s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Metadata {
    count: usize,
    checksum: u16,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_use() {
        let mut sr = StringRegistry::default();
        assert_eq!(sr.len(), 0);

        sr.push_str("feck").unwrap();
        sr.push_str("arse").unwrap();
        sr.push_str("drink").unwrap();
        assert_eq!(sr.len(), 3);

        assert_eq!(sr.get_index("arse"), Some(1));

        assert_eq!(sr.get_str(2), Some("drink"));

        sr.clear();
        assert_eq!(sr.len(), 0);
    }

    #[test]
    fn push_str_just_right() {
        let mut sr = StringRegistry::default();

        sr.push_str("01234567890123456789012345678901").unwrap();
    }

    #[test]
    #[should_panic(expected = "string is too long for string registry")]
    fn push_str_too_long() {
        let mut sr = StringRegistry::default();

        sr.push_str("012345678901234567890123456789012").unwrap();
    }

    #[test]
    fn get_index_not_found() {
        let sr = StringRegistry::default();

        assert!(sr.get_index("feck").is_none());
    }

    #[test]
    fn get_str_not_found() {
        let sr = StringRegistry::default();

        assert!(sr.get_str(0).is_none());
    }

    #[test]
    fn metadata() {
        let mut sr = StringRegistry::default();
        let m = sr.metadata();
        assert_eq!(m.count, 0);
        assert_eq!(m.checksum, 0x0000);

        sr.push_str("feck").unwrap();
        let m = sr.metadata();
        assert_eq!(m.count, 1);
        assert_eq!(m.checksum, 32952);

        sr.push_str("arse").unwrap();
        let m = sr.metadata();
        assert_eq!(m.count, 2);
        assert_eq!(m.checksum, 10830);

        sr.push_str("drink").unwrap();
        let m = sr.metadata();
        assert_eq!(m.count, 3);
        assert_eq!(m.checksum, 26691);
    }
}
