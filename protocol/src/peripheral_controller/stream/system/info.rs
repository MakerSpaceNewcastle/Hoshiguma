use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Info {
    pub git_revision: GitRevisionString,
}

pub type GitRevisionString = crate::String<20>;
