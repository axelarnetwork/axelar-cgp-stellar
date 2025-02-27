#![cfg(any(test, feature = "alloc"))]
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

const ASCII_RANGE: u8 = i8::MAX as u8;

pub trait StringExt {
    fn is_ascii(&self) -> bool;
}

impl StringExt for soroban_sdk::String {
    fn is_ascii(&self) -> bool {
        let mut bytes: Vec<u8> = vec![0; self.len() as usize];
        self.copy_into_slice(&mut bytes);

        for &byte in bytes.iter() {
            if byte > ASCII_RANGE {
                return false;
            }
        }
        true
    }
}
