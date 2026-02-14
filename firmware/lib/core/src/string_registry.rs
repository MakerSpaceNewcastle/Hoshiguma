use core::slice::Iter;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct StringRegistry<const N: usize, const L: usize> {
    strings: Vec<String<L>, N>,
}

impl<const N: usize, const L: usize> StringRegistry<N, L> {
    pub fn from_slice(a: &[&str]) -> Result<Self> {
        let mut sr = Self::default();
        for s in a {
            sr.push_str(s)?;
        }
        Ok(sr)
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn iter(&self) -> Iter<'_, String<L>> {
        self.strings.iter()
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

    pub fn push(&mut self, s: String<L>) -> Result<()> {
        if self.strings.contains(&s) {
            Ok(())
        } else {
            self.strings.push(s).map_err(|_| Error::RegistryFull)
        }
    }

    pub fn push_str(&mut self, s: &str) -> Result<()> {
        self.push(s.try_into().map_err(|_| Error::StringTooLong)?)
    }

    pub fn extend_from_str_slice(&mut self, ss: &[&str]) -> Result<()> {
        for s in ss {
            self.push_str(s)?;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.strings.clear()
    }

    pub fn get_str(&self, idx: usize) -> Option<&str> {
        self.strings.get(idx).map(|v| &**v)
    }

    pub fn get_string(&self, idx: usize) -> Option<String<L>> {
        self.strings.get(idx).cloned()
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

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Error {
    StringTooLong,
    RegistryFull,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_use() {
        let mut sr = StringRegistry::<8, 32>::default();
        assert!(sr.is_empty());
        assert_eq!(sr.len(), 0);

        sr.push_str("feck").unwrap();
        sr.push_str("arse").unwrap();
        sr.push_str("drink").unwrap();
        assert!(!sr.is_empty());
        assert_eq!(sr.len(), 3);

        assert_eq!(sr.get_index("arse"), Some(1));

        assert_eq!(sr.get_str(2), Some("drink"));

        sr.clear();
        assert_eq!(sr.len(), 0);
    }

    #[test]
    fn registry_full() {
        let mut sr = StringRegistry::<3, 32>::default();

        sr.push_str("feck").unwrap();
        sr.push_str("arse").unwrap();
        sr.push_str("drink").unwrap();

        assert_eq!(sr.push_str("nope"), Err(Error::RegistryFull));
    }

    #[test]
    fn push_str_just_right() {
        let mut sr = StringRegistry::<8, 32>::default();

        sr.push_str("01234567890123456789012345678901").unwrap();
    }

    #[test]
    fn push_str_too_long() {
        let mut sr = StringRegistry::<8, 32>::default();

        assert_eq!(
            sr.push_str("012345678901234567890123456789012"),
            Err(Error::StringTooLong)
        );
    }

    #[test]
    fn get_index_not_found() {
        let sr = StringRegistry::<8, 32>::default();

        assert!(sr.get_index("feck").is_none());
    }

    #[test]
    fn get_str_not_found() {
        let sr = StringRegistry::<8, 32>::default();

        assert!(sr.get_str(0).is_none());
    }

    #[test]
    fn metadata() {
        let mut sr = StringRegistry::<8, 32>::default();
        let m = sr.metadata();
        assert_eq!(m.count, 0);
        assert_eq!(m.checksum, 0x0000);
        let m = sr.metadata();
        assert_eq!(m.count, 0);
        assert_eq!(m.checksum, 0x0000);

        sr.push_str("feck").unwrap();
        let m = sr.metadata();
        assert_eq!(m.count, 1);
        assert_eq!(m.checksum, 32952);
        let m = sr.metadata();
        assert_eq!(m.count, 1);
        assert_eq!(m.checksum, 32952);

        sr.push_str("arse").unwrap();
        let m = sr.metadata();
        assert_eq!(m.count, 2);
        assert_eq!(m.checksum, 10830);
        let m = sr.metadata();
        assert_eq!(m.count, 2);
        assert_eq!(m.checksum, 10830);

        sr.push_str("drink").unwrap();
        let m = sr.metadata();
        assert_eq!(m.count, 3);
        assert_eq!(m.checksum, 26691);
        let m = sr.metadata();
        assert_eq!(m.count, 3);
        assert_eq!(m.checksum, 26691);
    }

    #[test]
    fn from_slice() {
        let sr = StringRegistry::<8, 32>::from_slice(&["feck", "arse", "drink"]).unwrap();
        assert_eq!(sr.len(), 3);
        assert_eq!(sr.get_index("arse"), Some(1));
        assert_eq!(sr.get_str(2), Some("drink"));
    }

    #[test]
    fn from_slice_duplicates() {
        let sr = StringRegistry::<8, 32>::from_slice(&["feck", "arse", "drink", "arse"]).unwrap();
        assert_eq!(sr.len(), 3);
        assert_eq!(sr.get_index("arse"), Some(1));
        assert_eq!(sr.get_str(2), Some("drink"));
    }

    #[test]
    fn extend_from_slice() {
        let mut sr = StringRegistry::<8, 32>::default();
        sr.extend_from_str_slice(&["feck", "arse"]).unwrap();
        sr.extend_from_str_slice(&["feck", "drink"]).unwrap();
        assert_eq!(sr.len(), 3);
        assert_eq!(sr.get_str(0), Some("feck"));
        assert_eq!(sr.get_str(1), Some("arse"));
        assert_eq!(sr.get_str(2), Some("drink"));
    }
}
