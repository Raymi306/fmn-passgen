//! TODO and WIP
#![allow(unused, missing_docs)]

// TODO why is everything public

// provides:
// static WORDLIST: &[&str] = &[...]
include!(concat!(env!("OUT_DIR"), "/wordlist.rs"));

use std::cell::OnceCell;
use std::collections::HashMap;
use std::iter;

use rand::TryRngCore;
use rand::prelude::*;
use rand_core::UnwrapErr;

use crate::consts::DIGITS;
use crate::consts::LETTERS;
use crate::consts::SYMBOLS;

// SAFETY do not access from reentrant code (interrupt handler, recursive call, etc)
static mut LABEL_MAP: OnceCell<HashMap<(&str, u8), String>> = OnceCell::new();
// TODO consider playing with thread_local!

pub trait Generator {
    fn generate<T: TryRngCore>(&self, rng: &mut UnwrapErr<T>) -> String;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    Word(Option<u8>, Option<u8>),
    Letter,
    Symbol,
    Digit,
    CharacterList(String),
    //Custom(String, Vec<ExpressionItem>)
}

impl Generator for Source {
    fn generate<R: TryRngCore>(&self, rng: &mut UnwrapErr<R>) -> String {
        match &self {
            Self::Word(optional_min, optional_max) => {
                let min = optional_min.map_or(0, |inner| inner as usize);
                let max = optional_max.map_or(255, |inner| inner as usize);

                let filtered_indices: Vec<usize> = WORDLIST
                    .iter()
                    .enumerate()
                    .filter(|(_, word)| (min..=max).contains(&word.chars().count()))
                    .map(|(i, _)| i)
                    .collect();
                if filtered_indices.is_empty() {
                    return String::new();
                }
                let index = filtered_indices.choose(rng).expect(
                    concat!(
                        "invariant 1: `filtered_indices` must not be empty and should have been guarded above.\n",
                        "invariant 2: size_hint on a slice iterator with no intermediary ",
                        "iterator adapters should always be accurate.",
                    )
                );
                WORDLIST[*index].to_owned()
            }
            Self::Letter => LETTERS.choose(rng).unwrap().to_string(),
            Self::Symbol => SYMBOLS.choose(rng).unwrap().to_string(),
            Self::Digit => DIGITS.choose(rng).unwrap().to_string(),
            Self::CharacterList(choices) => {
                if choices.is_empty() {
                    String::new()
                } else {
                    choices.chars().choose(rng).unwrap().to_string()
                }
            }
            /*
            Self::Custom(_, expression) => {
                let mut string_builder = Vec::new();
                for item in expression {
                    string_builder.push(item.apply(rng))
                }
                string_builder.join("")
            }
            */
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LabeledSource {
    pub source: Source,
    pub label: Option<u8>,
}

impl Generator for LabeledSource {
    fn generate<R: TryRngCore>(&self, rng: &mut UnwrapErr<R>) -> String {
        // temporarily disable labeling
        self.source.generate(rng)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    Reversed,
    Upper,
    Lower,
    CapitalizeFirst,
    CapitalizeLast,
    CapitalizeNotFirst,
    CapitalizeNotLast,
}

impl Filter {
    pub fn apply(&self, input: &str) -> String {
        match &self {
            Self::Reversed => input.chars().rev().collect(),
            Self::Upper => input.to_uppercase(),
            Self::Lower => input.to_lowercase(),
            Self::CapitalizeFirst => {
                let first = input.chars().take(1).map(|c| c.to_ascii_uppercase());
                first.chain(input.chars().skip(1)).collect()
            }
            Self::CapitalizeLast => {
                // UTF character length weirdness reminder
                let num_chars = input.chars().count();
                let len_minus_1 = num_chars.saturating_sub(1);
                input
                    .chars()
                    .take(len_minus_1)
                    .chain(
                        input
                            .chars()
                            .skip(len_minus_1)
                            .take(1)
                            .map(|c| c.to_ascii_uppercase()),
                    )
                    .collect()
            }
            Self::CapitalizeNotFirst => input
                .chars()
                .take(1)
                .chain(input.chars().skip(1).map(|c| c.to_ascii_uppercase()))
                .collect(),
            Self::CapitalizeNotLast => {
                // UTF character length weirdness reminder
                let num_chars = input.chars().count();
                let len_minus_1 = num_chars.saturating_sub(1);
                input
                    .chars()
                    .take(len_minus_1)
                    .map(|c| c.to_ascii_uppercase())
                    .chain(input.chars().skip(len_minus_1).take(1))
                    .collect()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyValue {
    Min(u8),
    Max(u8),
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub source: LabeledSource,
    pub filters: Vec<Filter>,
    pub repeat: u8,
}

impl Generator for Block {
    fn generate<T: TryRngCore>(&self, rng: &mut UnwrapErr<T>) -> String {
        if let Some(label) = self.source.label {
            // if there is a label and a repeat, only need to generate a result once
            let mut result = self.source.generate(rng);
            for filter in &self.filters {
                result = filter.apply(&result);
            }
            iter::repeat_n(result, self.repeat as usize).collect()
        } else {
            let mut string_builder = Vec::new();
            for _ in 0..self.repeat {
                let mut result = self.source.generate(rng);
                for filter in &self.filters {
                    result = filter.apply(&result);
                }
                string_builder.push(result);
            }
            string_builder.join("")
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Group {
    pub blocks: Vec<Block>,
    //pub filters: Vec<Filter>,
    pub repeat: u8,
}

impl Generator for Group {
    fn generate<T: TryRngCore>(&self, rng: &mut UnwrapErr<T>) -> String {
        let mut string_builder = Vec::new();
        for _ in 0..self.repeat {
            for block in &self.blocks {
                string_builder.push(block.generate(rng));
            }
        }
        string_builder.join("")
    }
}

#[derive(Debug)]
pub enum ExpressionItem {
    Block(Block),
    Group(Group),
}

impl Generator for ExpressionItem {
    fn generate<T: TryRngCore>(&self, rng: &mut UnwrapErr<T>) -> String {
        match self {
            Self::Block(v) => v.generate(rng),
            Self::Group(v) => v.generate(rng),
        }
    }
}

pub struct Expression {
    items: Vec<ExpressionItem>
}

impl Generator for Expression {
    fn generate<T: TryRngCore>(&self, rng: &mut UnwrapErr<T>) -> String {
        #![allow(unsafe_code, static_mut_refs)]
        unsafe {
            let _ = LABEL_MAP.set(HashMap::new()).unwrap_or(());
        };
        let mut string_builder = Vec::new();
        for item in &self.items {
            string_builder.push(item.generate(rng));
        }
        string_builder.join("")
    }
}
