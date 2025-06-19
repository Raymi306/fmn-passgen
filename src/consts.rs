//! Crate constants.

/// 0-9
pub const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
/// displayable symbols
pub const SYMBOLS: [char; 32] = [
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=',
    '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
];
/// a-zA-Z
pub const LETTERS: [char; 52] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
/// pronounceable syllables mapped to represent 7 bits
pub const KOREMUTAKE_SYLLABLES: [&str; 128] = [
    "ba", "be", "bi", "bo", "bu", "by", "da", "de", "di", "do", "du", "dy", "fa", "fe", "fi", "fo",
    "fu", "fy", "ga", "ge", "gi", "go", "gu", "gy", "ha", "he", "hi", "ho", "hu", "hy", "ja", "je",
    "ji", "jo", "ju", "jy", "ka", "ke", "ki", "ko", "ku", "ky", "la", "le", "li", "lo", "lu", "ly",
    "ma", "me", "mi", "mo", "mu", "my", "na", "ne", "ni", "no", "nu", "ny", "pa", "pe", "pi", "po",
    "pu", "py", "ra", "re", "ri", "ro", "ru", "ry", "sa", "se", "si", "so", "su", "sy", "ta", "te",
    "ti", "to", "tu", "ty", "va", "ve", "vi", "vo", "vu", "vy", "bra", "bre", "bri", "bro", "bru",
    "bry", "dra", "dre", "dri", "dro", "dru", "dry", "fra", "fre", "fri", "fro", "fru", "fry",
    "gra", "gre", "gri", "gro", "gru", "gry", "pra", "pre", "pri", "pro", "pru", "pry", "sta",
    "ste", "sti", "sto", "stu", "sty", "tra", "tre",
];
