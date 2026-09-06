//! Payment parsers get hostile input. Nothing here may panic.
//!
//! `cargo fuzz` covers the same ground properly, but needs nightly. This runs
//! on stable in CI, on a fixed seed so a failure is reproducible.

mod common;

use khqr_core::{decode, format_tlv, parse_tlv, verify_crc};

/// xorshift64, so the corpus is the same on every machine and every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn pick(&mut self, from: &[char]) -> char {
        from[self.below(from.len())]
    }
}

const ALPHABET: [char; 16] = [
    '0',
    '1',
    '2',
    '5',
    '9',
    'A',
    'K',
    'H',
    '@',
    '.',
    '-',
    ' ',
    'ភ',
    'ñ',
    '\u{0}',
    '\u{10FFFF}',
];

fn exercise(input: &str) {
    let _ = verify_crc(input);
    let _ = decode(input);

    if let Ok(fields) = parse_tlv(input) {
        for field in fields {
            let _ = format_tlv(&field.tag, &field.value);
            let _ = parse_tlv(&field.value);
        }
    }
}

#[test]
fn random_input_never_panics() {
    let mut rng = Rng(0x5150_4b48_5152_0001);

    for _ in 0..20_000 {
        let length = rng.below(60);
        let input: String = (0..length).map(|_| rng.pick(&ALPHABET)).collect();

        exercise(&input);
    }
}

#[test]
fn mutated_vectors_never_panic() {
    let mut rng = Rng(0x4b48_5152_2603_0001);

    for _ in 0..20_000 {
        let mut chars: Vec<char> = common::ALL[rng.below(common::ALL.len())].chars().collect();

        match rng.below(4) {
            0 => {
                let at = rng.below(chars.len());
                chars[at] = rng.pick(&ALPHABET);
            }
            1 => chars.truncate(rng.below(chars.len())),
            2 => {
                let at = rng.below(chars.len());
                chars.insert(at, rng.pick(&ALPHABET));
            }
            _ => {
                let at = rng.below(chars.len());
                chars.remove(at);
            }
        }

        exercise(&chars.iter().collect::<String>());
    }
}

#[test]
fn awkward_shapes_never_panic() {
    for input in [
        "",
        "6",
        "63",
        "630",
        "6304",
        "00",
        "0002",
        "000299",
        "5999ភ្នំពេញ",
        "6304ភ្នំ",
        "29990014a@b",
        "\u{0}\u{0}\u{0}\u{0}",
        "ភភភភភភភភ",
    ] {
        exercise(input);
    }
}
