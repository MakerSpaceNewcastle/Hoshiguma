pub(crate) struct CheckedUpdate<T> {
    value: Option<T>,
}

impl<T> Default for CheckedUpdate<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T: PartialEq + ufmt::uDebug> CheckedUpdate<T> {
    pub(crate) fn new(value: T) -> Self {
        Self { value: Some(value) }
    }

    pub(crate) fn store(&mut self, value: T) -> bool {
        let value = Some(value);
        let changed = value != self.value;
        self.value = value;
        changed
    }

    pub(crate) fn get(&self) -> &T {
        self.value.as_ref().unwrap()
    }
}
