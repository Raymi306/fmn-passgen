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
/// ## Example
/// ```
/// impl Integer for u8 {}
/// type MinimalSupportedInteger = u16
/// ```
impl Integer for MinimalSupportedInteger {}

/// Represent pertinent data when validating data.
#[derive(Clone, Debug)]
pub enum ValidationError {
    InvalidNumber(String, MinimalSupportedInteger, MinimalSupportedInteger),
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
/// - Allows for referencing the enum's name                           (implemented by derive macro)
/// - Allows for referencing the kebab-case names of each enum member. (implemented by derive macro)
/// - Allows for iterating over enum member, enum name pairs           (implemented by derive macro)
/// - Allows for converting an enum member into it's kebab-case name.  (implemented by derive macro)
/// - Allows for converting a &str to an enum member.                  (default implementation provided)
///
/// Is used in conjunction with the `AutoStrEnum` derive macro to provide
/// low boilerplate "types" that are easy to reason with on the command line.
pub trait StrEnum: Sized + Default + Clone + Copy
where
    Self: 'static,
{
    const NAME: &'static str;
    const NAME_MEMBER_ARR: &[(&str, Self)];
    fn to_static_str(&self) -> &'static str;
    fn into_iter() -> impl Iterator<Item = &'static (&'static str, Self)>;
    fn to_member(member: &str) -> Result<&Self, ValidationError> {
        Self::into_iter()
            .find(|(s, _)| *s == member)
            .map(|inner| &inner.1)
            .ok_or_else(|| {
                let valid_choices = Self::into_iter()
                    .map(|inner| inner.0)
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

/// The different ways words can be transformed.
#[derive(StrEnum, Copy, Clone, Debug)]
pub enum WordTransformationType {
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
#[derive(StrEnum, Copy, Clone, Debug)]
pub enum PaddingType {
    None,
    /// add padding-length padding-characters to front and back
    #[default]
    Fixed,
    /// if unpadded password is less than padding-length, append padding-characters to desired length
    Adaptive,
}

// TODO, fixes main.rs help brittleness in a const manner
// do more stupid stuff with macros
//
// #[derive(StrEnum, Copy, Clone, Debug)]
// pub enum RngType {
//     #[default, example = "(the system's native secure RNG)"]
//     OsRng,
//     #[example = "(a reasonably secure userspace RNG)"]
//     Csprng,
// }
/// The different random number generator options.
#[derive(StrEnum, Copy, Clone, Debug)]
pub enum RngType {
    /// the system's native secure RNG
    #[default]
    OsRng,
    /// a reasonably secure userspace RNG
    Csprng,
}
