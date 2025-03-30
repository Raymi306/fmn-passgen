#![expect(missing_docs, reason = "derive macros could use some more polish")]
//! Shared types, enums, and structs.
use std::fmt;

use strenum_derive::StrEnum;

/// Change this type to support a wider range of integer values (lower or higher)
type MinimalSupportedInteger = u8;

/// Allow us to work on standard Rust integer types in a generic manner
/// by defining what the minimum necessary shared functionality is.
pub trait Integer: std::str::FromStr + Into<MinimalSupportedInteger> + PartialOrd + Copy {}

/// To support more standard integer types, add additional impls.
///
/// # Example
/// ```ignore
/// impl Integer for u8 {}
/// type MinimalSupportedInteger = u16;
/// ```
impl Integer for MinimalSupportedInteger {}

/// Represent pertinent data when validating data.
#[derive(Clone, Debug)]
pub enum ValidationError {
    /// Validating a number failed.
    InvalidNumber(String, MinimalSupportedInteger, MinimalSupportedInteger),
    /// Validating an enum failed.
    InvalidEnum(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::InvalidNumber(value, min, max) => {
                format!("`{value}` must be a positive integer between {min} and {max}")
            }
            Self::InvalidEnum(msg) => msg.clone(),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for ValidationError {}

/// Give enums superpowers.
///
/// Is used in conjunction with the [`StrEnum`] derive macro to provide
/// low boilerplate "types" that are easy to reason with on the command line.
pub trait StrEnum: Sized + Default + Clone + Copy
where
    Self: 'static,
{
    /// The enum name as a &str, implemented by derive macro.
    const NAME: &'static str;
    /// Constant references to enum member name, enum member pairs, implemented by derive macro.
    const NAME_MEMBER_ARR: &[(&str, Self)];
    /// Convert an enum member into its kebab-case name, implemented by derive macro.
    fn to_static_str(&self) -> &'static str;
    /// Iterate over enum member name, enum member pairs, implemented by derive macro.
    fn into_iter() -> impl Iterator<Item = &'static (&'static str, Self)>;
    /// Convert a &str to an enum member.
    ///
    /// # Errors
    ///
    /// Will return [`ValidationError`] if `member` does not represent a valid enum member.
    fn to_member(member: &str) -> Result<&Self, ValidationError> {
        Self::into_iter()
            // compare str
            .find(|(s, _)| *s == member)
            // map to Self
            .map(|(_, e)| e)
            .ok_or_else(|| {
                // lazily generate error message
                // TODO add const [] of just &'static str member names
                // TODO consider making valid_choices const
                let valid_choices = Self::into_iter()
                    .map(|(s, _)| *s)
                    .collect::<Vec<&str>>()
                    .join(", ");
                let parent = Self::NAME;
                let msg = format!(
                    "`{member}` is not a valid {parent}. Possible choices: {valid_choices}"
                );
                ValidationError::InvalidEnum(msg)
            })
    }
}
// TODO, fix main.rs help brittleness
// do more stupid stuff with macros, pull the docstring as const help text
//
// #[derive(StrEnum, Copy, Clone, Debug)]
// pub enum RngType {
//     /// the system's native secure RNG
//     #[default]
//     OsRng,
//     ...
// }

/// The different ways words can be transformed.
#[derive(StrEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum WordTransformationType {
    /// apply no transformation, useful if the wordlist already contains capital words
    None,
    /// correct horse battery staple
    Lower,
    /// CORRECT HORSE BATTERY STAPLE
    Upper,
    /// Correct Horse Battery Staple
    CapitalizeFirst,
    /// correcT horsE batterY staplE
    CapitalizeLast,
    /// cORRECT hORSE bATTERY sTAPLE
    CapitalizeNotFirst,
    /// correct HORSE battery STAPLE
    #[default]
    AlternatingLowerUpper,
    /// CORRECT horse BATTERY staple
    AlternatingUpperLower,
    /// correct HORSE battery staple
    RandomUpperLower,
}

/// The different ways padding can be applied.
#[derive(StrEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum PaddingType {
    /// apply no padding
    None,
    /// add padding-length padding-characters to front
    FixedFront,
    #[default]
    /// add padding-length padding-characters to back
    FixedBack,
    /// add padding-length padding-characters to front and back
    FixedBoth,
    /// alias for [`FixedBoth`]
    Fixed,
    /// if unpadded password is less than padding-length, prepend padding-characters to desired length
    AdaptiveFront,
    /// if unpadded password is less than padding-length, append padding-characters to desired length
    AdaptiveBack,
    /// alias for [`AdaptiveBack`]
    Adaptive,
}

/// The different random number generator options.
#[derive(StrEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum RngType {
    /// the system's native secure RNG
    #[default]
    OsRng,
    /// a reasonably secure userspace RNG
    Csprng,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem::discriminant;

    #[test]
    fn test_strenum_name() {
        assert_eq!(RngType::NAME, "RngType");
    }

    #[test]
    fn test_strenum_name_member_arr() {
        assert_eq!("os-rng", RngType::NAME_MEMBER_ARR[0].0);
        assert_eq!("csprng", RngType::NAME_MEMBER_ARR[1].0);
        assert_eq!(
            discriminant(&RngType::OsRng),
            discriminant(&RngType::NAME_MEMBER_ARR[0].1)
        );
        assert_eq!(
            discriminant(&RngType::Csprng),
            discriminant(&RngType::NAME_MEMBER_ARR[1].1)
        );
    }

    #[test]
    fn test_strenum_to_static_str() {
        assert_eq!("os-rng", RngType::OsRng.to_static_str());
    }

    #[test]
    fn test_strenum_to_member_success() {
        assert_eq!(
            discriminant(RngType::to_member("os-rng").unwrap()),
            discriminant(&RngType::OsRng)
        );
    }

    #[test]
    fn test_strenum_to_member_err() {
        RngType::to_member("not-a-member").unwrap_err();
    }
}
