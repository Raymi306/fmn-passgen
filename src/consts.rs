//! Crate constants.

/// 0-9
pub const DIGIT_ALPHABET: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[expect(missing_docs, reason = "the names are clear enough")]
/// Configuration defaults.
pub mod default {
    /// !@$%^&*-_+=:|~?/.
    pub const SYMBOL_ALPHABET: [char; 18] = [
        '!', '@', '$', '%', '^', '&', '*', '-', '_', '+', '=', ':', '|', '~', '?', '/', '.', ';',
    ];
    pub const COUNT: u8 = 1;
    pub const WORD_COUNT: u8 = 4;
    pub const WORD_MIN_LENGTH: u8 = 3;
    pub const WORD_MAX_LENGTH: u8 = 11;
    pub const DIGITS_BEFORE: u8 = 0;
    pub const DIGITS_AFTER: u8 = 2;
    pub const PADDING_LENGTH_FIXED: u8 = 1;
    pub const PADDING_LENGTH_ADAPTIVE: u8 = 42;
}
