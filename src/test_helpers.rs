//! Shared testing functionality.
#![expect(
    clippy::missing_panics_doc,
    reason = "these functions are only intended for testing"
)]
#![cfg(test)]

//provides:
//static WORDLIST: &[&str] = &[...]

include!(concat!(env!("OUT_DIR"), "/wordlist.rs"));

use rand::SeedableRng;
use rand::TryRngCore;
use rand::rngs::SmallRng;

use crate::config::ConfigBuilder;
use crate::password_maker::PasswordMaker;

/// Makes a [`PasswordMaker`] with reproducible random output and a small wordlist.
#[must_use]
pub fn make_seeded_maker(seed: u64) -> PasswordMaker<SmallRng> {
    let rng = SmallRng::seed_from_u64(seed).unwrap_err();
    PasswordMaker {
        rng,
        config: ConfigBuilder::new().build().unwrap(),
        wordlist: make_wordlist(),
    }
}

/// Makes a [`PasswordMaker`] with reproducible random output and a real wordlist.
#[must_use]
pub fn make_seeded_maker_big_list(seed: u64) -> PasswordMaker<SmallRng> {
    let rng = SmallRng::seed_from_u64(seed).unwrap_err();
    PasswordMaker {
        rng,
        config: ConfigBuilder::new().build().unwrap(),
        wordlist: WORDLIST.iter().map(|s| String::from(*s)).collect(),
    }
}

/// Makes a small, easy to work with wordlist.
pub fn make_wordlist() -> Vec<String> {
    [
        "modern", "labor", "hello", "world", "water", "fire", "deep", "ice", "pie",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Check if a &str is uppercase.
pub fn str_is_uppercase(word: &str) -> bool {
    word.chars().all(char::is_uppercase)
}

/// Check if a &str is lowercase.
pub fn str_is_lowercase(word: &str) -> bool {
    word.chars().all(char::is_lowercase)
}
