use core::ops::Deref;
use defmt::Format;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Changed {
    Yes,
    No,
}

#[must_use = "checked_set is useless if the result of the check is not required"]
pub fn checked_set<T: PartialEq>(value: &mut T, new: T) -> Changed {
    if *value == new {
        Changed::No
    } else {
        *value = new;
        Changed::Yes
    }
}

pub struct ObservedValue<T: Clone + PartialEq> {
    value: Option<T>,
}

impl<T: Clone + PartialEq> Default for ObservedValue<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T: Clone + PartialEq> Deref for ObservedValue<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Clone + PartialEq> ObservedValue<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: Some(initial),
        }
    }

    pub fn update(&mut self, new_value: T) -> Changed {
        checked_set(&mut self.value, Some(new_value))
    }

    pub fn update_and<F: FnOnce(T)>(&mut self, new_value: T, on_change: F) {
        if self.update(new_value) == Changed::Yes {
            // Will always have a value when changed
            on_change(self.value.clone().unwrap());
        }
    }

    pub async fn update_and_async<F: AsyncFnOnce(T)>(&mut self, new_value: T, on_change: F) {
        if self.update(new_value) == Changed::Yes {
            // Will always have a value when changed
            let value = self.value.clone().unwrap();

            on_change(value).await;
        }
    }
}
