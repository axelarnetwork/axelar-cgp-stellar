#![cfg(any(test, feature = "alloc"))]
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

const ASCII_MAX: u8 = i8::MAX as u8;

pub trait StringExt {
    fn is_ascii(&self) -> bool;
}

impl StringExt for soroban_sdk::String {
    fn is_ascii(&self) -> bool {
        let mut bytes: Vec<u8> = vec![0; self.len() as usize];
        self.copy_into_slice(&mut bytes);

        for &byte in bytes.iter() {
            if byte > ASCII_MAX {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{Env, String};

    use super::*;

    #[test]
    fn validate_ascii_strings_are_ascii() {
        let test_cases = [
            "",
            "Hello, world!",
            "The quick brown fox jumps over the lazy dog.",
            "1234567890",
            "!@#$%^&*()_+-=[]{}|;:',.<>/?",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "abcdefghijklmnopqrstuvwxyz",
        ];

        let env = Env::default();
        for ascii_string in test_cases {
            let soroban_ascii_string = String::from_str(&env, ascii_string);
            assert!(soroban_ascii_string.is_ascii());
        }
    }

    #[test]
    fn validate_non_ascii_strings_not_ascii() {
        let test_cases = [
            "Hello, 世界!",
            "こんにちは世界",
            "Привет, мир!",
            "안녕하세요 세계",
            "مرحبا بالعالم",
            "Bonjour le monde 🌍",
            "¡Hola, mundo!",
            "Γειά σου Κόσμε",
            "नमस्ते दुनिया",
            "שלום עולם",
            "你好，世界",
        ];

        let env = Env::default();
        for ascii_string in test_cases {
            let soroban_non_ascii_string = String::from_str(&env, ascii_string);
            assert!(!soroban_non_ascii_string.is_ascii());
        }
    }
}
