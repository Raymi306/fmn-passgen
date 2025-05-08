//! TODO and WIP
#![allow(missing_docs, unsafe_code)]

// TODO why is everything public

// provides:
// static WORDLIST: &[&str] = &[...]
include!(concat!(env!("OUT_DIR"), "/wordlist.rs"));

use std::collections::HashMap;
use std::iter;
use std::mem::MaybeUninit;

use rand::TryRngCore;
use rand::prelude::*;
use rand_core::UnwrapErr;

use crate::consts::DIGITS;
use crate::consts::LETTERS;
use crate::consts::SYMBOLS;

// SAFETY do not access from reentrant code (interrupt handler, recursive call, etc)
// TODO consider playing with thread_local instead
static mut LABEL_MAP: MaybeUninit<HashMap<(String, u8), String>> = MaybeUninit::uninit();

pub trait Generator {
    fn generate<T: TryRngCore>(&self, rng: &mut UnwrapErr<T>) -> String;
}

#[derive(Clone, Debug)]
pub enum Source {
    Word(Option<u8>, Option<u8>),
    Letter,
    Symbol,
    Digit,
    CharacterList(String),
    Custom(String, Vec<ExpressionItem>)
}

impl Generator for Source {
    fn generate<R: TryRngCore>(&self, rng: &mut UnwrapErr<R>) -> String {
        #[expect(clippy::unwrap_used)]
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
            Self::Custom(_, expression) => {
                let mut string_builder = Vec::new();
                for item in expression {
                    string_builder.push(item.generate(rng));
                }
                string_builder.join("")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct LabeledSource {
    pub source: Source,
    pub label: Option<u8>,
}

impl Generator for LabeledSource {
    fn generate<R: TryRngCore>(&self, rng: &mut UnwrapErr<R>) -> String {
        if let Some(label) = self.label {
            let type_string = match &self.source {
                Source::Word(_, _) => "word".to_owned(),
                Source::Letter => "letter".to_owned(),
                Source::Symbol => "symbol".to_owned(),
                Source::Digit => "digit".to_owned(),
                Source::CharacterList(_) => "character_list".to_owned(),
                Source::Custom(val, _) => val.clone(),
            };
            let cache = unsafe {
                #[expect(static_mut_refs)]
                LABEL_MAP.assume_init_mut()
            };
            if let Some(inner) = cache.get(&(type_string.clone(), label)) {
                return inner.to_string();
            }
            let result = self.source.generate(rng);
            cache.insert((type_string, label), result.clone());
            return result;
        }
        self.source.generate(rng)
    }
}

impl LabeledSource {
    pub unsafe fn init_cache() {
        unsafe {
            #[expect(static_mut_refs)]
            LABEL_MAP.write(HashMap::new());
        }
    }
    pub unsafe fn clear_cache() {
        unsafe {
            #[expect(static_mut_refs)]
            LABEL_MAP.assume_init_mut().clear();
        }
    }
}

#[derive(Clone, Debug)]
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
    #[must_use]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyValue {
    Min(u8),
    Max(u8),
}

#[derive(Clone, Debug)]
pub struct Block {
    pub source: LabeledSource,
    pub filters: Vec<Filter>,
    pub repeat: u8,
}

impl Generator for Block {
    fn generate<T: TryRngCore>(&self, rng: &mut UnwrapErr<T>) -> String {
        if self.source.label.is_some() {
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

#[derive(Clone, Debug)]
pub struct Group {
    pub blocks: Vec<Block>,
    pub filters: Vec<Filter>,
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
        let mut result = string_builder.join("");
        for filter in &self.filters {
            result = filter.apply(&result);
        }
        result
    }
}

#[derive(Clone, Debug)]
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
    pub items: Vec<ExpressionItem>
}

impl Generator for Expression {
    fn generate<T: TryRngCore>(&self, rng: &mut UnwrapErr<T>) -> String {
        let mut string_builder = Vec::new();
        for item in &self.items {
            string_builder.push(item.generate(rng));
        }
        unsafe {
            LabeledSource::clear_cache();
        }
        string_builder.join("")
    }
}
