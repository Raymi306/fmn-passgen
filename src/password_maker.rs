// TODO
#![allow(unused, missing_docs)]
//! Provides the [`PasswordMaker`] struct.
//!
//! The password generation algorithm is implemented here.
// provides:
// static WORDLIST: &[&str] = &[...]
include!(concat!(env!("OUT_DIR"), "/wordlist.rs"));

use rand::TryRngCore;
use rand::prelude::*;
use rand_core::UnwrapErr;

use crate::types::Expression;
use crate::types::Generator;

#[derive(Debug)]
pub struct PasswordMaker<T>
where
    T: TryRngCore,
{
    /// A random number generator
    rng: UnwrapErr<T>,
}

impl<T> Default for PasswordMaker<T>
where
    T: TryRngCore + Default,
{
    fn default() -> Self {
        Self {
            rng: T::default().unwrap_err(),
        }
    }
}

impl<T> PasswordMaker<T>
where
    T: TryRngCore,
{
    pub fn make_password(&mut self, expression: &Expression) -> String {
        expression.generate(&mut self.rng)
    }
    pub fn make_passwords(&mut self, expression: &Expression, count: usize) -> Vec<String> {
        let mut passwords = Vec::with_capacity(count);
        for _ in 0..count {
            passwords.push(self.make_password(expression));
        }
        passwords
    }
}
