use std;
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
/// Methods for calculating the penalty of a keyboard layout given an input
/// corpus string.
//use layout;
use std::vec::Vec;

use crate::layout::*;

// Every cost the model charges, in report row order. Weights live at the
// `log` calls in `calculate_penalty`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PenaltyKind {
    // Per-key effort from `BASE_PENALTY`.
    Base,
    // Same finger twice on different keys.
    SameFinger,
    // Top row to bottom row, or back, between pinky and ring or ring and
    // middle, plus a jump from the middle or ring bottom up to the index top.
    LongJumpConsecutive,
    // Pinky on the top row next to the ring finger on a lower row, e.g.
    // QS/SQ, PL/LP on QWERTY.
    PinkyRingTwist,
    // Ring, pinky, then middle finger of one hand, or middle, pinky, then ring.
    RollReversal,
    // Switching hands. Negative weight, so a reward.
    Alternation,
    // Rolling outwards on one hand. Negative weight.
    RollOut,
    // Inward roll on one hand onto the index or middle finger. "Roll in" then
    // refunds most of it for the comfortable ones.
    AnyRollIn,
    // Comfortable inward roll. Negative weight.
    RollIn,
    // Top row to bottom row, or back, on one finger with a keystroke between.
    LongJumpSandwich,
}

impl PenaltyKind {
    pub const ALL: [PenaltyKind; 10] = [
        PenaltyKind::Base,
        PenaltyKind::SameFinger,
        PenaltyKind::LongJumpConsecutive,
        PenaltyKind::PinkyRingTwist,
        PenaltyKind::RollReversal,
        PenaltyKind::Alternation,
        PenaltyKind::RollOut,
        PenaltyKind::AnyRollIn,
        PenaltyKind::RollIn,
        PenaltyKind::LongJumpSandwich,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PenaltyKind::Base => "Base",
            PenaltyKind::SameFinger => "Same finger",
            PenaltyKind::LongJumpConsecutive => "Long jump consecutive",
            PenaltyKind::PinkyRingTwist => "Pinky/ring twist",
            PenaltyKind::RollReversal => "Roll reversal",
            PenaltyKind::Alternation => "Alternation",
            PenaltyKind::RollOut => "Roll out",
            PenaltyKind::AnyRollIn => "Any roll in",
            PenaltyKind::RollIn => "Roll in",
            PenaltyKind::LongJumpSandwich => "Long jump sandwich",
        }
    }
}

// Rows are indexed by `kind as usize`, so ALL must list every variant in
// declaration order. This checks the order only: a variant missing from ALL
// panics out of bounds on its first `log`.
const _: () = {
    let mut i = 0;
    while i < PenaltyKind::ALL.len() {
        assert!(PenaltyKind::ALL[i] as usize == i);
        i += 1;
    }
};

#[derive(Clone, PartialEq)]
pub struct KeyPenalty {
    pub kind: PenaltyKind,
    pub times: f64,
    pub total: f64,
}
#[derive(Clone, PartialEq)]
pub struct Penalty {
    // One row per kind, in `PenaltyKind::ALL` order. Rows sum to `total`.
    pub penalties: Vec<KeyPenalty>,
    pub fingers: [i64; 8],
    pub hands: [i64; 2],
    // Includes thumbs, unlike `fingers` and `hands`, so it sums to `len`.
    pub keys: KeyMap<i64>,
    pub hand_penalties: [f64; 2],
    pub total: f64,
    pub len: i64,
}
impl Penalty {
    pub fn new() -> Penalty {
        Penalty {
            penalties: PenaltyKind::ALL
                .iter()
                .map(|&kind| KeyPenalty {
                    kind: kind,
                    times: 0.0,
                    total: 0.0,
                })
                .collect(),
            fingers: [0; 8],
            hands: [0; 2],
            keys: [0; NUM_OF_KEYS],
            hand_penalties: [0.0; 2],
            total: 0.0,
            len: 0,
        }
    }

    pub fn row(&self, kind: PenaltyKind) -> &KeyPenalty {
        &self.penalties[kind as usize]
    }
}
#[derive(Clone)]
pub struct BestLayoutsEntry {
    pub layout: Layout,
    pub penalty: Penalty,
}
impl Ord for BestLayoutsEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.penalty.total.partial_cmp(&other.penalty.total) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        }
    }
}
impl PartialEq for BestLayoutsEntry {
    fn eq(&self, other: &Self) -> bool {
        self.penalty.total == other.penalty.total
    }
}
impl Eq for BestLayoutsEntry {}
impl PartialOrd for BestLayoutsEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.penalty.total.partial_cmp(&other.penalty.total)
    }
}
#[derive(PartialEq)]
pub struct QuartadList {
    pub map: HashMap<[char; 3], i64>,
}
impl fmt::Display for KeyPenalty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind.name(), self.total)
    }
}

static BASE_PENALTY: KeyMap<f64> = [
    3.0, 1.0, 1.0, 1.0, 2.5,   2.5, 1.0, 1.0, 1.0, 3.0, 5.0, 
    0.5, 0.1, 0.1, 0.1, 1.0,   1.0, 0.1, 0.1, 0.1, 1.5, 5.0, 
    10.0, 1.5, 1.5, 1.0, 2.5,   2.5, 1.0, 1.5, 1.5, 3.0, 
                        0.0,    0.0,
];

// Lowercases, folds accents, and keeps only letters, spaces, and newlines.
// The bundled corpora are already folded this way (Czech "muze", German "fur",
// "groses"), so pasted text scores on the same terms.
pub fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(fold_accent)
        .filter_map(|c| match c {
            // Only space and newline have keys. A CR from CRLF line endings would
            // drop every trigram that spans a line break.
            '\r' => None,
            '\n' => Some('\n'),
            // Tabs and non-breaking spaces would otherwise drop their trigrams.
            c if c.is_whitespace() => Some(' '),
            // Also drops decomposed accent marks (U+0300..U+036F), so "z" +
            // combining caron folds like a precomposed "ž".
            c if c.is_alphabetic() => Some(c),
            _ => None,
        })
        .collect()
}

// Covers Czech, Slovak, German, and common Western and Central European letters.
// TODO: switch to NFD via the `unicode-normalization` crate if visitors paste
// other Latin scripts and their trigrams get dropped as non-ASCII.
fn fold_accent(c: char) -> char {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ą' => 'a',
        'ç' | 'č' | 'ć' => 'c',
        'ď' => 'd',
        'è' | 'é' | 'ê' | 'ë' | 'ě' | 'ę' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ľ' | 'ĺ' | 'ł' => 'l',
        'ñ' | 'ň' | 'ń' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ő' | 'ø' => 'o',
        'ř' | 'ŕ' => 'r',
        // The German corpus spells "groß" as "gros".
        'š' | 'ś' | 'ß' => 's',
        'ť' => 't',
        'ù' | 'ú' | 'û' | 'ü' | 'ů' | 'ű' => 'u',
        'ý' | 'ÿ' => 'y',
        'ž' | 'ź' | 'ż' => 'z',
        other => other,
    }
}

// Windows over chars, not bytes: slicing a &str by char index panics inside
// multi-byte characters such as 'ž'.
pub fn prepare_quartad_list(string: &str) -> QuartadList {
    let chars: Vec<char> = string.chars().collect();
    let mut quartads: HashMap<[char; 3], i64> = HashMap::new();

    for window in chars.windows(3) {
        // The layout maps only ASCII keys.
        if window.iter().all(|c| c.is_ascii()) {
            *quartads.entry([window[0], window[1], window[2]]).or_insert(0) += 1;
        }
    }
    QuartadList { map: quartads }
}

pub fn calculate_penalty(quartads: &QuartadList, layout: &Layout) -> BestLayoutsEntry {
    let mut result = Penalty::new();
    let position_map = layout.get_position_map();

    for (&[c2, c1, c0], count) in &quartads.map {
        let old2 = position_map.get_key_position(c2);
        let old1 = position_map.get_key_position(c1);
        let curr = match position_map.get_key_position(c0) {
            &Some(ref kp) => kp,
            &None => continue,
        };
        result.len += count;
        result.keys[curr.pos] += count;

        let useFinger = |result: &mut Penalty, i: usize| match curr.finger {
            Finger::Pinky => result.fingers[i] += count,
            Finger::Ring => result.fingers[i + 1] += count,
            Finger::Middle => result.fingers[i + 2] += count,
            Finger::Index => result.fingers[i + 3] += count,
            Finger::Thumb => {}
        };
        match curr.hand {
            Hand::Left => {
                result.hands[0] += count;
                useFinger(&mut result, 0);
            }
            Hand::Right => {
                result.hands[1] += count;
                useFinger(&mut result, 4);
            }
            _ => {}
        };
        let mut log = |kind: PenaltyKind, penalty: f64| {
            let p = penalty * *count as f64;
            result.penalties[kind as usize].times += *count as f64;
            result.penalties[kind as usize].total += p;
            match curr.hand {
                Hand::Left => {
                    result.hand_penalties[0] += p;
                }
                Hand::Right => {
                    result.hand_penalties[1] += p;

                }
                _ => {}
            };
            result.total += p;
        };

        let count = *count as f64;
        log(PenaltyKind::Base, BASE_PENALTY[curr.pos] / 3.0);

        let old1 = match *old1 {
            Some(ref o) => o,
            None => continue,
        };

        if curr.hand == old1.hand && curr.hand != Hand::Thumb {
            if curr.finger == old1.finger && curr.pos == old1.pos {
                continue;
            }
            
            if curr.finger == old1.finger && curr.pos != old1.pos {
                log(PenaltyKind::SameFinger, 5.0);
            }

            if curr.row == Row::Top && old1.row == Row::Bottom
                || curr.row == Row::Bottom && old1.row == Row::Top
            {
                if curr.finger == Finger::Ring && old1.finger == Finger::Pinky
                    || curr.finger == Finger::Pinky && old1.finger == Finger::Ring
                    || curr.finger == Finger::Middle && old1.finger == Finger::Ring
                    || curr.finger == Finger::Ring && old1.finger == Finger::Middle
                    || (curr.finger == Finger::Index
                        && (old1.finger == Finger::Middle || old1.finger == Finger::Ring)
                        && curr.row == Row::Top
                        && old1.row == Row::Bottom)
                {
                    log(PenaltyKind::LongJumpConsecutive, 10.0);
                }
            }

            if (curr.finger == Finger::Ring
                && old1.finger == Finger::Pinky
                && (curr.row == Row::Home && old1.row == Row::Top
                    || curr.row == Row::Bottom && old1.row == Row::Top))
                || (curr.finger == Finger::Pinky
                    && old1.finger == Finger::Ring
                    && (curr.row == Row::Top && old1.row == Row::Home
                        || curr.row == Row::Top && old1.row == Row::Bottom))
            {
                log(PenaltyKind::PinkyRingTwist, 10.0);
            }
            
            if is_roll_out(curr.finger, old1.finger) {
                log(PenaltyKind::RollOut, -0.01);
            }

            if is_roll_in(curr.finger, old1.finger) {
                // Rolls from the bottom row or across top and bottom feel awkward,
                // so they get no refund.
                if old1.row != Row::Bottom && !(
                    curr.row == Row::Top && old1.row == Row::Bottom
                    || (curr.row == Row::Bottom && old1.row == Row::Top && curr.finger != Finger::Index)
                ) {
                    log(PenaltyKind::RollIn, -0.99);
                }
                log(PenaltyKind::AnyRollIn, 2.0);
            }
        }
        let old2 = match *old2 {
            Some(ref o) => o,
            None => continue,
        };
    

        if curr.hand != old1.hand {
            log(PenaltyKind::Alternation, -0.1);
        }

        if curr.hand == old1.hand && old1.hand == old2.hand {
            if (curr.finger == Finger::Middle
                && old1.finger == Finger::Pinky
                && old2.finger == Finger::Ring)
                || curr.finger == Finger::Ring
                    && old1.finger == Finger::Pinky
                    && old2.finger == Finger::Middle
            {
                log(PenaltyKind::RollReversal, 3.0);
            }
        }

        if curr.hand == old2.hand && curr.finger == old2.finger {
            if curr.row == Row::Top && old2.row == Row::Bottom
                || curr.row == Row::Bottom && old2.row == Row::Top
            {
                log(PenaltyKind::LongJumpSandwich, 5.0);
            }
        }

        fn is_roll_out(curr: Finger, prev: Finger) -> bool {
            match curr {
                Finger::Middle => prev == Finger::Index,
                Finger::Ring => {
                    prev != Finger::Pinky && prev != Finger::Ring && prev != Finger::Thumb
                }
                Finger::Pinky => prev != Finger::Pinky && prev != Finger::Thumb,
                _ => false,
            }
        }
        // my restricted roll-in, as not all inward rolls feel good
        fn is_roll_in(curr: Finger, prev: Finger) -> bool {
            match curr {
                Finger::Index => prev != Finger::Thumb && prev != Finger::Index,
                Finger::Middle => prev == Finger::Pinky || prev == Finger::Ring,
                _ => false,
            }
        }
    }
    //result.total = *(result.hand_penalties.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
    BestLayoutsEntry {
        layout: layout.clone(),
        penalty: result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_text_folds_precomposed_and_decomposed_accents() {
        assert_eq!(normalize_text("Žluťoučký kůň"), "zlutoucky kun");
        assert_eq!(normalize_text("z\u{30C}lu\u{30C}"), "zlu");
        assert_eq!(normalize_text("Größe, ja!"), "grose ja");
        assert_eq!(normalize_text("a\tb\r\nc\u{A0}d"), "a b\nc d");
    }

    #[test]
    fn penalty_rows_and_key_presses_account_for_every_keystroke() {
        let quartads = prepare_quartad_list("the quick brown fox jumps over the lazy dog\n");
        let penalty = calculate_penalty(&quartads, &QWERTY_LAYOUT).penalty;
        let rows: f64 = penalty.penalties.iter().map(|p| p.total).sum();
        assert!((rows - penalty.total).abs() < 1e-9);
        assert_eq!(penalty.keys.iter().sum::<i64>(), penalty.len);
    }

    #[test]
    fn prepare_quartad_list_counts_by_char() {
        assert!(prepare_quartad_list("ab").map.is_empty());
        let quartads = prepare_quartad_list("žabab");
        assert_eq!(quartads.map.get(&['a', 'b', 'a']), Some(&1));
        assert_eq!(quartads.map.get(&['ž', 'a', 'b']), None);
    }
}
