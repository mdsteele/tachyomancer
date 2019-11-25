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

use super::super::eval::{CircuitState, EvalError, EvalScore, PuzzleEval};
use super::iface::{Interface, InterfacePort, InterfacePosition};
use crate::geom::{Coords, Direction};
use crate::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

const ALIEN_INPUT: &[u8] =
    b"EGARAW RASLIJANI EYON VARENEM SI RIHAN UT UTASA= \
      EGARAW UT ONUBEMIA ID ISYKEA DUHAM UTASIKI= \
      EYOM IGNU YAPI DENG IX= \
      RIHAN SHAOMA UTASA UT NU GAREP EYOMI VAD= ";
const EXPECTED_OUTPUT: &[u8] =
    b"ALIENS BIZARRE US ATTACKED DURING DAY THE FIRST. \
      ALIENS THE KILLING AND DESTRUCTION WANT ONLY. \
      WE BECAUSE WHY KNOW NOT. \
      DAY THAT FIRST THE OF WAR OUR WAS. ";
const MAX_TRANSLATION_BUFFER_LEN: usize = 12;
const TRANSLATIONS: &[(u32, &[u8], &[u8])] = &[
    (4, b"DENG", b"KNOW"),
    (4, b"DUHAM", b"WANT"),
    (5, b"EGARAW", b"ALIENS"),
    (4, b"EYOM", b"WE"),
    (5, b"EYOMI", b"OUR"),
    (4, b"EYON", b"US"),
    (5, b"GAREP", b"WAR"),
    (3, b"ID", b"AND"),
    (5, b"IGNU", b"BECAUSE"),
    (8, b"ISYKEA", b"DESTRUCTION"),
    (3, b"IX", b"NOT"),
    (4, b"NU", b"OF"),
    (9, b"ONUBEMIA", b"KILLING"),
    (9, b"RASLIJANI", b"BIZARRE"),
    (5, b"RIHAN", b"DAY"),
    (5, b"SHAOMA", b"THAT"),
    (4, b"SI", b"DURING"),
    (3, b"UT", b"THE"),
    (4, b"UTASA", b"FIRST"),
    (4, b"UTASIKI", b"ONLY"),
    (5, b"VAD", b"WAS"),
    (7, b"VARENEM", b"ATTACKED"),
    (5, b"YAPI", b"WHY"),
];
const UNKNOWN_TRANSLATION: &[u8] = b"???";

//===========================================================================//

fn alien_byte_to_value(byte: u8) -> u32 {
    if byte >= b'A' && byte <= b'Z' {
        (byte - b'A' + 1).into()
    } else if byte == b'=' {
        30
    } else {
        0
    }
}

fn alien_value_to_byte(value: u32) -> u8 {
    if value == 0 {
        b' '
    } else if value >= 1 && value <= 26 {
        b'A' + ((value - 1) as u8)
    } else if value == 30 {
        b'='
    } else {
        b'?'
    }
}

fn translate_alien_word(mut alien: &[u8]) -> (u32, Vec<u8>) {
    if alien.is_empty() {
        return (1, Vec::new());
    }
    let last_index = alien.len() - 1;
    let period = alien[last_index] == b'=';
    if period {
        alien = &alien[..last_index];
    }
    let (delay, mut human) =
        match TRANSLATIONS.binary_search_by_key(&alien, |&t| t.1) {
            Ok(index) => {
                let (delay, _, human) = TRANSLATIONS[index];
                (delay, human.to_vec())
            }
            Err(_) => (alien.len() as u32, UNKNOWN_TRANSLATION.to_vec()),
        };
    if period {
        human.push(b'.');
    }
    (delay, human)
}

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Reader Interface",
        description: "Connects to an OCR scanner for reading alien text.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Read",
            description:
                "Sends (at most) one event per time step, with the next alien \
                 character to be translated.",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "Printer Interface",
        description: "Connects to a printer for printing out translated text.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Print",
            description:
                "Send translated words here, one character at a time.  Each \
                 word should be terminated by a zero.",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "Translation Interface",
        description: "Connects to a word-for-word translation database.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Alien",
                description:
                    "Send an alien word here, one character at a time, \
                     terminated by a zero; once terminated, it is an error \
                     to send more events here until the entire translation \
                     has been sent back.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Human",
                description:
                    "After some number of time steps, the translated word \
                     will be sent from here, one character at a time, \
                     terminated by a zero.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
        ],
    },
];

//===========================================================================//

pub struct TranslatorEval {
    read_wire: usize,
    print_port: (Coords, Direction),
    print_wire: usize,
    alien_port: (Coords, Direction),
    alien_wire: usize,
    human_wire: usize,
    num_bytes_read: usize,
    translation_buffer: Vec<u8>,
    pending_translation: Option<(u32, Vec<u8>)>,
    printed_bytes: Vec<u8>,
}

impl TranslatorEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TranslatorEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 2);
        TranslatorEval {
            read_wire: slots[0][0].1,
            print_port: slots[1][0].0,
            print_wire: slots[1][0].1,
            alien_port: slots[2][0].0,
            alien_wire: slots[2][0].1,
            human_wire: slots[2][1].1,
            num_bytes_read: 0,
            translation_buffer: Vec::new(),
            pending_translation: None,
            printed_bytes: Vec::new(),
        }
    }

    pub fn bytes_read(&self) -> &[u8] {
        &ALIEN_INPUT[..self.num_bytes_read]
    }

    pub fn bytes_printed(&self) -> &[u8] {
        &self.printed_bytes
    }

    pub fn translation_buffer(&self) -> &[u8] {
        &self.translation_buffer
    }

    pub fn pending_translation(&self) -> Option<&[u8]> {
        self.pending_translation
            .as_ref()
            .map(|&(_, ref bytes)| bytes.as_slice())
    }

    fn send_next_translated_byte(&mut self, state: &mut CircuitState) {
        if let Some((0, ref mut bytes)) = self.pending_translation {
            debug_assert!(!bytes.is_empty());
            state.send_event(self.human_wire, bytes[0].into());
            bytes.remove(0);
            if bytes.is_empty() {
                self.translation_buffer.clear();
                self.pending_translation = None;
            }
        }
    }
}

impl PuzzleEval for TranslatorEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        if self.num_bytes_read < ALIEN_INPUT.len() {
            state.send_event(
                self.read_wire,
                alien_byte_to_value(ALIEN_INPUT[self.num_bytes_read]),
            );
            self.num_bytes_read += 1;
        }
        self.send_next_translated_byte(state);
        if self.printed_bytes.len() >= EXPECTED_OUTPUT.len() {
            Some(EvalScore::Value(state.time_step()))
        } else {
            None
        }
    }

    fn begin_additional_cycle(&mut self, state: &mut CircuitState) {
        self.send_next_translated_byte(state);
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if let Some(value) = state.recv_event(self.alien_wire) {
            if self.pending_translation.is_some() {
                let msg = format!(
                    "Can't push new characters to translation \
                     buffer while translation is in progress"
                );
                errors.push(state.fatal_port_error(self.alien_port, msg));
            } else if value == 0 {
                let (delay, mut human) =
                    translate_alien_word(&self.translation_buffer);
                human.push(0);
                self.pending_translation = Some((delay, human));
            } else {
                self.translation_buffer.push(alien_value_to_byte(value));
                if self.translation_buffer.len() > MAX_TRANSLATION_BUFFER_LEN {
                    let msg = format!(
                        "Too many characters in translation buffer (max word \
                         length is {})",
                        MAX_TRANSLATION_BUFFER_LEN
                    );
                    errors.push(state.fatal_port_error(self.alien_port, msg));
                }
            }
        }
        if let Some(value) = state.recv_event(self.print_wire) {
            debug_assert!(value <= 0xff);
            let byte = value as u8;
            let byte = if byte == 0 {
                b' '
            } else if (byte >= b'A' && byte <= b'Z') || byte == b'.' {
                byte
            } else {
                b'?'
            };
            let print_index = self.printed_bytes.len();
            let expected_byte = EXPECTED_OUTPUT[print_index];
            if self.printed_bytes.len() >= EXPECTED_OUTPUT.len() {
                let message = format!(
                    "Printed too many output characters (expected only {})",
                    EXPECTED_OUTPUT.len()
                );
                errors.push(state.fatal_port_error(self.print_port, message));
            } else if byte != expected_byte {
                let expected_value =
                    if expected_byte == b' ' { 0 } else { expected_byte };
                let message = format!(
                    "Printed incorrect character at position {} \
                     (expected {}, but was {})",
                    print_index, expected_value, value
                );
                errors.push(state.fatal_port_error(self.print_port, message));
            }
            self.printed_bytes.push(byte);
        }
        errors
    }

    fn needs_another_cycle(&self, _time_step: u32) -> bool {
        match self.pending_translation {
            Some((0, _)) => true,
            _ => false,
        }
    }

    fn end_time_step(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        if let Some((ref mut delay, _)) = self.pending_translation {
            debug_assert!(*delay > 0);
            *delay -= 1;
        }
        Vec::new()
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{
        alien_byte_to_value, alien_value_to_byte, translate_alien_word,
        ALIEN_INPUT, MAX_TRANSLATION_BUFFER_LEN, TRANSLATIONS,
    };

    #[test]
    fn alien_bytes_and_values() {
        for &byte in ALIEN_INPUT.iter() {
            assert_eq!(alien_value_to_byte(alien_byte_to_value(byte)), byte);
        }
        assert_eq!(alien_value_to_byte(255), b'?');
    }

    #[test]
    fn translate_words() {
        assert_eq!(translate_alien_word(b"DENG"), (4, b"KNOW".to_vec()));
        assert_eq!(translate_alien_word(b"RIHAN"), (5, b"DAY".to_vec()));
        assert_eq!(translate_alien_word(b"EYON="), (4, b"US.".to_vec()));
        assert_eq!(translate_alien_word(b"FOOBAR"), (6, b"???".to_vec()));
    }

    #[test]
    fn translation_buffer_is_large_enough() {
        for &(_, alien_word, _) in TRANSLATIONS.iter() {
            assert!(alien_word.len() <= MAX_TRANSLATION_BUFFER_LEN);
        }
    }

    #[test]
    fn translations_are_sorted() {
        let mut prev: &'static [u8] = b"";
        for &(_, alien_word, _) in TRANSLATIONS.iter() {
            assert!(
                prev < alien_word,
                "Word {:?} is out of place",
                String::from_utf8_lossy(alien_word)
            );
            prev = alien_word;
        }
    }
}

//===========================================================================//
