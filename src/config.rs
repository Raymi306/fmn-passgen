//! Configuration and validation.
use config_builder_derive::ConfigBuilder;

use crate::consts::default;
use crate::types::Integer;
use crate::types::PaddingType;
use crate::types::RngType;
use crate::types::StrEnum;
use crate::types::ValidationError;
use crate::types::WordTransformationType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// how many passwords to make
    pub count: u8,
    /// number of words to choose
    pub word_count: u8,
    /// minimum length of a chosen word
    pub word_min_length: u8,
    /// maximum length of a chosen word
    pub word_max_length: u8,
    /// transformation to apply to the selected words
    pub word_transformation: WordTransformationType,
    /// number of digits to prepend
    pub digits_before: u8,
    /// number of digits to append
    pub digits_after: u8,
    /// how to apply padding
    pub padding_type: PaddingType,
    /// how much to pad
    pub padding_length: u8,
    /// list of characters from which to choose the padding character
    pub padding_characters: Vec<char>,
    /// list of characters from which to choose the separator character
    pub separator_characters: Vec<char>,
    /// method of random number generation
    pub rng_type: RngType,
}

impl Default for Config {
    fn default() -> Self {
        ConfigBuilder::new().build().unwrap()
    }
}

/// Provide a way in which to create a validated [`Config`].
#[derive(ConfigBuilder, Debug, Default)]
pub struct ConfigBuilder {
    count: Option<String>,
    word_count: Option<String>,
    word_min_length: Option<String>,
    word_max_length: Option<String>,
    word_transformation: Option<String>,
    digits_before: Option<String>,
    digits_after: Option<String>,
    padding_type: Option<String>,
    padding_length: Option<String>,
    padding_characters: Option<String>,
    separator_characters: Option<String>,
    rng_type: Option<String>,
}

/// Ensure an [`Integer`] is between `min` and `max`.
/// If no `value` is provided, return `default`
fn validate_int<T: Integer>(
    value: Option<String>,
    min: T,
    max: T,
    default: T,
) -> Result<T, ValidationError> {
    value.map_or(Ok(default), |inner| {
        let Ok(result) = inner.parse::<T>() else {
            return Err(ValidationError::InvalidNumber(
                inner,
                min.into(),
                max.into(),
            ));
        };

        if (min..=max).contains(&result) {
            Ok(result)
        } else {
            Err(ValidationError::InvalidNumber(
                inner,
                min.into(),
                max.into(),
            ))
        }
    })
}

/// Ensure `value` references a valid [`StrEnum`] member.
/// If no `value` is provided, return `default`
fn validate_enum<T: StrEnum>(value: Option<String>) -> Result<T, ValidationError> {
    #[expect(clippy::or_fun_call, reason = "Function call is not expensive.")]
    value.map_or(Ok(T::default()), |inner| {
        T::to_member(&inner.to_ascii_lowercase()).copied()
    })
}

/// Turn a [`String`] into a [`Vec<char>`] with no duplicates.
/// If no `value` is provided, return `default`
fn uniquify_chars(value: Option<String>, default: &[char]) -> Vec<char> {
    value.map_or_else(
        || default.to_vec(),
        |inner| {
            let mut result = inner.chars().collect::<Vec<char>>();
            result.sort_unstable();
            result.dedup();
            result
        },
    )
}

/// Setters are auto generated by [`strenum_derive::StrEnum`].
impl ConfigBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> Result<Config, ValidationError> {
        // TODO add constraints to consts.rs
        let count = validate_int::<u8>(self.count, 1, 255, default::COUNT)?;
        let word_count = validate_int::<u8>(self.word_count, 0, 32, default::WORD_COUNT)?;
        let word_min_length =
            validate_int::<u8>(self.word_min_length, 1, 255, default::WORD_MIN_LENGTH)?;
        let word_max_length = validate_int::<u8>(
            self.word_max_length,
            word_min_length,
            255,
            default::WORD_MAX_LENGTH,
        )?;
        let word_transformation =
            validate_enum::<WordTransformationType>(self.word_transformation)?;
        let digits_before = validate_int::<u8>(self.digits_before, 0, 255, default::DIGITS_BEFORE)?;
        let digits_after = validate_int::<u8>(self.digits_after, 0, 255, default::DIGITS_AFTER)?;
        let padding_characters = uniquify_chars(self.padding_characters, &default::SYMBOL_ALPHABET);
        let padding_type = if padding_characters.is_empty() {
            PaddingType::None
        } else {
            validate_enum::<PaddingType>(self.padding_type)?
        };
        let padding_length = validate_int::<u8>(self.padding_length, 0, 255, {
            match padding_type {
                PaddingType::Fixed => default::PADDING_LENGTH_FIXED,
                PaddingType::Adaptive => default::PADDING_LENGTH_ADAPTIVE,
                PaddingType::None => 0,
            }
        })?;
        let separator_characters =
            uniquify_chars(self.separator_characters, &default::SYMBOL_ALPHABET);
        let rng_type = validate_enum::<RngType>(self.rng_type)?;

        Ok(Config {
            count,
            word_count,
            word_min_length,
            word_max_length,
            word_transformation,
            digits_before,
            digits_after,
            padding_type,
            padding_length,
            padding_characters,
            separator_characters,
            rng_type,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "testing")]
    use super::*;
    use std::mem::discriminant;

    #[test]
    fn test_default() {
        let config = ConfigBuilder::default().build().unwrap();
        assert_eq!(config.count, default::COUNT);
        assert_eq!(config.word_count, default::WORD_COUNT);
        assert_eq!(config.word_min_length, default::WORD_MIN_LENGTH);
        assert_eq!(config.word_max_length, default::WORD_MAX_LENGTH);
        assert_eq!(
            discriminant(&config.word_transformation),
            discriminant(&WordTransformationType::default())
        );
        assert_eq!(config.digits_before, default::DIGITS_BEFORE);
        assert_eq!(config.digits_after, default::DIGITS_AFTER);
        assert_eq!(
            discriminant(&config.padding_type),
            discriminant(&PaddingType::default())
        );
        assert_eq!(config.padding_length, default::PADDING_LENGTH_FIXED);
        assert_eq!(config.padding_characters, default::SYMBOL_ALPHABET.to_vec());
        assert_eq!(
            config.separator_characters,
            default::SYMBOL_ALPHABET.to_vec()
        );
        assert_eq!(
            discriminant(&config.rng_type),
            discriminant(&RngType::default())
        );
    }

    #[test]
    fn test_word_max_length_bound_to_min() {
        let config_err = ConfigBuilder::new()
            .word_min_length(Some("42".to_owned()))
            .word_max_length(Some("41".to_owned()))
            .build()
            .unwrap_err();
        println!("{config_err:?}");
        let what = matches!(config_err, ValidationError::InvalidNumber(provided, 42, 255) if provided == *"41");
        assert!(what);
    }

    #[test]
    fn test_padding_length_default_changes_with_padding_type() {
        let config = ConfigBuilder::new()
            .padding_type(Some("adaptive".to_owned()))
            .build()
            .unwrap();
        assert_eq!(config.padding_length, default::PADDING_LENGTH_ADAPTIVE);
    }
}
