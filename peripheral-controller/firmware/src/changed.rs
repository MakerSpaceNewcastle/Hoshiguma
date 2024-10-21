use defmt::Format;

#[derive(PartialEq, Eq, Format)]
pub(crate) enum Changed {
    Yes,
    No,
}

impl Changed {
    pub(crate) fn or(&self, other: Self) -> Self {
        if *self == Self::Yes || other == Self::Yes {
            Self::Yes
        } else {
            Self::No
        }
    }
}

#[must_use = "checked_set is useless if the result of the check is not required"]
pub(crate) fn checked_set<T: PartialEq>(value: &mut T, new: T) -> Changed {
    if *value == new {
        Changed::No
    } else {
        *value = new;
        Changed::Yes
    }
}
