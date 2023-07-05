pub trait UnwrapSimple {
    type Ok;

    fn unwrap_simple(self) -> Self::Ok;
}

impl<T, E> UnwrapSimple for Result<T, E> {
    type Ok = T;

    fn unwrap_simple(self) -> T {
        match self {
            Ok(v) => v,
            Err(_) => panic!(),
        }
    }
}
