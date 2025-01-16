#[cfg(any(test, feature = "testutils"))]
pub use testutils::*;

pub trait ThenOk<T, E> {
    fn then_ok(self, ok: T, err: E) -> Result<T, E>;
}

impl<T, E> ThenOk<T, E> for bool {
    fn then_ok(self, ok: T, err: E) -> Result<T, E> {
        self.then_some(ok).ok_or(err)
    }
}

#[cfg(any(test, feature = "testutils"))]
mod testutils {
    extern crate std;
    use std::vec::Vec as StdVec;

    use soroban_sdk::{Bytes, Env, IntoVal, TryFromVal, Val, Vec};

    pub trait IntoVec<T> {
        fn into_vec(self, env: &Env) -> Vec<T>;
    }

    impl<T: Clone + IntoVal<Env, Val> + TryFromVal<Env, Val>> IntoVec<T> for std::vec::Vec<T> {
        fn into_vec(self, env: &Env) -> Vec<T> {
            Vec::from_slice(env, self.as_slice())
        }
    }

    pub trait BytesExt {
        fn from_hex(env: &Env, hex_string: &str) -> Bytes;
    }

    impl BytesExt for Bytes {
        fn from_hex(env: &Env, hex_string: &str) -> Bytes {
            let bytes_vec: StdVec<u8> = hex::decode(hex_string).expect("hex decoding failed");
            Self::from_slice(env, &bytes_vec)
        }
    }
}
