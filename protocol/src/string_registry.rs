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
