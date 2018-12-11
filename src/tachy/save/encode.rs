// +--------------------------------------------------------------------------+
// | Copyright 2018 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of Tachyomancer.                                       |
// |                                                                          |
// | Tachyomancer is free software: you can redistribute it and/or modify it  |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | Tachyomancer is distributed in the hope that it will be useful, but      |
// | WITHOUT ANY WARRANTY; without even the implied warranty of               |
// | MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU        |
// | General Public License for details.                                      |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with Tachyomancer.  If not, see <http://www.gnu.org/licenses/>.          |
// +--------------------------------------------------------------------------+

use std::char;
use std::ffi::{OsStr, OsString};
use std::u32;
use std::u64;

//===========================================================================//

pub fn decode_name(file_name: &OsStr) -> String {
    let mut decoded = String::new();
    let string = file_name.to_string_lossy();
    let mut chars = string.chars();
    while let Some(chr) = chars.next() {
        if chr == ',' {
            let mut value: u64 = 0;
            let mut any_digits = false;
            let mut found_comma = false;
            while let Some(digit_chr) = chars.next() {
                if let Some(digit) = digit_chr.to_digit(16) {
                    any_digits = true;
                    value = value * 16 + (digit as u64);
                    if value > (u32::MAX as u64) {
                        break;
                    }
                } else if digit_chr == ',' {
                    found_comma = true;
                    break;
                } else {
                    break;
                }
            }
            if !found_comma {
                // Skip past next comma:
                while let Some(digit_chr) = chars.next() {
                    if digit_chr == ',' {
                        break;
                    }
                }
            } else if any_digits && value <= (u32::MAX as u64) {
                if let Some(decoded_chr) = char::from_u32(value as u32) {
                    decoded.push(decoded_chr);
                    continue;
                }
            }
            decoded.push(char::REPLACEMENT_CHARACTER);
        } else if chr == '_' {
            decoded.push(' ');
        } else {
            decoded.push(chr);
        }
    }
    decoded
}

pub fn encode_name(name: &str) -> OsString {
    let mut encoded = String::new();
    for chr in name.chars() {
        if chr.is_ascii_alphanumeric() || chr == '-' {
            encoded.push(chr);
        } else if chr == ' ' {
            encoded.push('_');
        } else {
            encoded.push_str(&format!(",{:x},", chr as u32));
        }
    }
    OsString::from(encoded)
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{decode_name, encode_name};
    use std::char;
    use std::ffi::OsString;

    #[test]
    fn name_round_trip() {
        assert_eq!(encode_name(""), OsString::from(""));
        assert_eq!(decode_name(&OsString::from("")), "".to_string());

        assert_eq!(encode_name("Jane Doe-99"), OsString::from("Jane_Doe-99"));
        assert_eq!(decode_name(&OsString::from("Jane_Doe-99")),
                   "Jane Doe-99".to_string());

        assert_eq!(encode_name("Jane_Doe-99"),
                   OsString::from("Jane,5f,Doe-99"));
        assert_eq!(decode_name(&OsString::from("Jane,5f,Doe-99")),
                   "Jane_Doe-99".to_string());

        assert_eq!(encode_name(".."), OsString::from(",2e,,2e,"));
        assert_eq!(decode_name(&OsString::from(",2e,,2e,")), "..".to_string());

        assert_eq!(encode_name("prefs.toml"), OsString::from("prefs,2e,toml"));
        assert_eq!(decode_name(&OsString::from("prefs,2e,toml")),
                   "prefs.toml".to_string());

        assert_eq!(encode_name("/Users/janedoe/*"),
                   OsString::from(",2f,Users,2f,janedoe,2f,,2a,"));
        assert_eq!(decode_name(&OsString::from(",2f,Users,2f,janedoe\
                                                ,2f,,2a,")),
                   "/Users/janedoe/*".to_string());

        assert_eq!(encode_name("Snowman \u{2603}"),
                   OsString::from("Snowman_,2603,"));
        assert_eq!(decode_name(&OsString::from("Snowman_,2603,")),
                   "Snowman \u{2603}".to_string());
    }

    #[test]
    fn name_decode_errors() {
        // No hex digits:
        assert_eq!(decode_name(&OsString::from("Foo,,Bar")),
                   "Foo\u{fffd}Bar".to_string());
        // No closing comma:
        assert_eq!(decode_name(&OsString::from("Foo,2f")),
                   "Foo\u{fffd}".to_string());
        // Invalid hex digits:
        assert_eq!(decode_name(&OsString::from("Foo,efgh,Bar")),
                   "Foo\u{fffd}Bar".to_string());
        // Too large a value:
        assert_eq!(decode_name(&OsString::from("Foo,1234567890,Bar")),
                   "Foo\u{fffd}Bar".to_string());
        // Invalid Unicode character:
        assert!(char::from_u32(0xd800).is_none());
        assert_eq!(decode_name(&OsString::from("Foo,d800,Bar")),
                   "Foo\u{fffd}Bar".to_string());
    }
}

//===========================================================================//
