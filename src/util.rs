use std::ptr::{null, null_mut};

pub(crate) trait AsPtr<T> {
    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl<T> AsPtr<T> for Option<T> {
    fn as_ptr(&self) -> *const T {
        match self {
            Some(x) => x as *const T,
            None => null(),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        match self {
            Some(x) => x as *mut T,
            None => null_mut(),
        }
    }
}
