#[allow(dead_code)]
pub struct AppleIIeString {
    pub string: String,
    pub a2string: String,
}

#[allow(dead_code)]
impl AppleIIeString {
    pub fn from_str(str: String) -> Self {
        let mut a2 = String::with_capacity(str.len());
        for (_, c) in str.char_indices() {
            let newc = {
                if c.is_ascii_uppercase() {
                    let letter = c as u8 - 'A' as u8;
                    (0x1 + letter) as char
                } else if c.is_ascii_lowercase() {
                    let letter = c as u8 - 'a' as u8;
                    (0x61 + letter) as char
                } else if c.is_ascii_digit() {
                    let letter = c as u8 - '0' as u8;
                    (0x30 + letter) as char
                } else {
                    let val = match c {
                        '[' => 0x1B,
                        '\\' => 0x1C,
                        ']' => 0x1D,
                        '^' => 0x1E,
                        '_' => 0x1F,
                        ' ' => 0x20,
                        '!' => 0x21,
                        '"' => 0x22,
                        '#' => 0x23,
                        '$' => 0x24,
                        '%' => 0x25,
                        '&' => 0x26,
                        '\'' => 0x27,
                        '(' => 0x28,
                        ')' => 0x29,
                        '*' => 0x2A,
                        '+' => 0x2B,
                        ',' => 0x2C,
                        '-' => 0x2D,
                        '.' => 0x2E,
                        '/' => 0x2F,
                        ':' => 0x3A,
                        ';' => 0x3B,
                        '<' => 0x3C,
                        '=' => 0x3D,
                        '>' => 0x3E,
                        '?' => 0x3F,
                        _ => 0x56u8
                    };
                    val as char
                }
            };
            a2.push(newc);
        }
        Self {
            string: str,
            a2string: a2,
        }
    }

    pub fn from_a2(a2: String) -> Self {
        let str = String::with_capacity(a2.len());
        Self {
            string: str,
            a2string: a2
        }
    }
}
