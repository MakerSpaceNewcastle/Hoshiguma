#[cfg(feature = "std")]
pub type String<const N: usize> = std::string::String;
#[cfg(feature = "no-std")]
pub type String<const N: usize> = heapless::String<N>;

#[cfg(feature = "std")]
pub type Vec<T, const N: usize> = std::vec::Vec<T>;
#[cfg(feature = "no-std")]
pub type Vec<T, const N: usize> = heapless::Vec<T, N>;

pub type GitVersionString = String<20>;
