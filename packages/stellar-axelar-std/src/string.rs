#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub trait StringExt {
    fn is_ascii(&self) -> bool;
}

#[cfg(feature = "alloc")]
impl StringExt for soroban_sdk::String {
    fn is_ascii(&self) -> bool {
        const ASCII_RANGE: u8 = 127;
        let mut bytes: Vec<u8> = vec![0; self.len() as usize];
        self.copy_into_slice(&mut bytes);

        for &byte in bytes.iter() {
            // check if byte within ASCII range (0-127)
            if byte > ASCII_RANGE {
                return false;
            }
        }
        true
    }
}
