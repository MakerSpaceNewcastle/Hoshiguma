use heapless::Vec;

pub type RegisteredString = heapless::String<32>;

#[derive(Default, Debug)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct StringRegistry {
    strings: Vec<RegisteredString, 32>,
}

impl StringRegistry {
    pub fn push(&mut self, s: RegisteredString) -> Result<(), RegisteredString> {
        self.strings.push(s)
    }

    pub fn get_str(&self, idx: usize) -> Option<&str> {
        self.strings.get(idx).map(|v| &**v)
    }

    pub fn get_index(&self, s: &str) -> Option<usize> {
        self.strings.iter().position(|v| v == s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_use() {
        let mut sr = StringRegistry::default();

        sr.push("feck".try_into().unwrap()).unwrap();
        sr.push("arse".try_into().unwrap()).unwrap();
        sr.push("drink".try_into().unwrap()).unwrap();

        assert_eq!(sr.get_index("arse"), Some(1));

        assert_eq!(sr.get_str(2), Some("drink"));
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
}
