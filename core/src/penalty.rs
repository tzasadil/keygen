use std;
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
/// Methods for calculating the penalty of a keyboard layout given an input
/// corpus string.
//use layout;
use std::vec::Vec;

use layout::*;

pub struct KeyPenaltyDescription {
    name: &'static str,
    show: bool,
}

#[derive(Clone)]
pub struct KeyPenalty {
    pub name: &'static str,
    pub times: f64,
    pub total: f64,
    pub show: bool,
}
#[derive(Clone)]
pub struct Penalty {
    pub penalties: Vec<KeyPenalty>,
    pub fingers: [i64; 8],
    pub hands: [i64; 2],
    pub hand_penalties: [f64; 2],
    pub total: f64,
    pub len: i64,
}
impl Penalty {
    pub fn new() -> Penalty {
        let mut penalties = Vec::new();
        for desc in PenaltyDescriptions.into_iter() {
            penalties.push(KeyPenalty {
                name: desc.name,
                show: desc.show,
                total: 0.0,
                times: 0.0,
            });
        }
        Penalty {
            penalties: penalties,
            fingers: [0; 8],
            hands: [0; 2],
            hand_penalties: [0.0; 2],
            total: 0.0,
            len : 0,
        }
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
pub struct QuartadList {
    pub map: HashMap<[char; 3], i64>,
}
impl fmt::Display for KeyPenalty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.total)
    }
}

static BASE_PENALTY: KeyMap<f64> = [
    3.0, 1.0, 1.0, 1.0, 2.5,   2.5, 1.0, 1.0, 1.0, 3.0, 5.0, 
    0.5, 0.1, 0.1, 0.1, 1.0,   1.0, 0.1, 0.1, 0.1, 1.5, 5.0, 
    10.0, 1.5, 1.5, 1.0, 2.5,   2.5, 1.0, 1.5, 1.5, 3.0, 
                        0.0,    0.0,
];

static PenaltyDescriptions: [KeyPenaltyDescription; 15] = [
    // Base penalty.
    KeyPenaltyDescription {
        name: "Base",
        show: true,
    },
    // Penalise 5 points for using the same finger twice on different keys.
    // An extra 5 points for using the centre column.
    KeyPenaltyDescription {
        name: "Same finger",
        show: true,
    },
    // Penalise 1 point for jumping from top to bottom row or from bottom to
    // top row on the same hand.
    KeyPenaltyDescription {
        name: "Long jump hand",
        show: false,
    },
    // Penalise 10 points for jumping from top to bottom row or from bottom to
    // top row on the same finger.
    KeyPenaltyDescription {
        name: "Long jump",
        show: true,
    },
    // Penalise 5 points for jumping from top to bottom row or from bottom to
    // top row on consecutive fingers, except for middle finger-top row ->
    // index finger-bottom row.
    KeyPenaltyDescription {
        name: "Long jump consecutive",
        show: false,
    },
    // Penalise 10 points for awkward pinky/ring combination where the pinky
    // reaches above the ring finger, e.g. QA/AQ, PL/LP, ZX/XZ, ;./.; on Qwerty.
    KeyPenaltyDescription {
        name: "Rinky/ring twist",
        show: false,
    },
    // Penalise 20 points for reversing a roll at the end of the hand, i.e.
    // using the ring, pinky, then middle finger of the same hand, or the
    // middle, pinky, then ring of the same hand.
    KeyPenaltyDescription {
        name: "Roll reversal",
        show: true,
    },
    //7
    KeyPenaltyDescription {
        name: "Long roll out",
        show: false,
    },
    KeyPenaltyDescription {
        name: "Alternation",
        show: true,
    },
    // Penalise 0.125 points for rolling outwards.
    KeyPenaltyDescription {
        name: "Roll out",
        show: true,
    },
    // Award 0.125 points for rolling inwards.
    KeyPenaltyDescription {
        name: "Roll in",
        show: true,
    },
    // Penalise 3 points for jumping from top to bottom row or from bottom to
    // top row on the same finger with a keystroke in between.
    KeyPenaltyDescription {
        name: "long jump sandwich",
        show: false,
    },
    // Penalise 10 points for three consecutive keystrokes going up or down the
    // three rows of the keyboard in a roll.
    KeyPenaltyDescription {
        name: "twist",
        show: false,
    },
    //13
    KeyPenaltyDescription {
        name: "4 times no alternation",
        show: false,
    },
    //14
    KeyPenaltyDescription {
        name: "4 alternations in a row",
        show: false,
    },
];

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
        let mut log = |i: usize, penalty: f64| {
            let p = penalty * *count as f64;
            //println!("{}; {}", i, penalty);
            if i!=99 {//&& curr.hand!= Hand::Left{
                result.penalties[i].times += *count as f64;
                result.penalties[i].total += p;
                

            }
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
        // 0: Base penalty.
        log(0, BASE_PENALTY[curr.pos] / 3.0);

        let old1 = match *old1 {
            Some(ref o) => o,
            None => continue,
        };

        if curr.hand == old1.hand && curr.hand != Hand::Thumb {
            if curr.finger == old1.finger && curr.pos == old1.pos {
                continue;
            }
            
            // 1: Same finger.
            if curr.finger == old1.finger && curr.pos != old1.pos {
                log(1, 5.0 );
            }

            // 2: Long jump hand.
            if curr.row == Row::Top && old1.row == Row::Bottom
                || curr.row == Row::Bottom && old1.row == Row::Top
            {
                log(2, 0.0);
            }

            // 3: Long jump.
            if curr.finger == old1.finger {
                if curr.row == Row::Top && old1.row == Row::Bottom
                    //|| curr.row == Row::Bottom && old1.row == Row::Top
                {
                    log(3, 0.0);
                }
            }
            
            // 4: Long jump consecutive.
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
                    log(3, 10.0);
                }
            }

            // 5: Pinky/ring twist.
            if (curr.finger == Finger::Ring
                && old1.finger == Finger::Pinky
                && (curr.row == Row::Home && old1.row == Row::Top
                    || curr.row == Row::Bottom && old1.row == Row::Top))
                || (curr.finger == Finger::Pinky
                    && old1.finger == Finger::Ring
                    && (curr.row == Row::Top && old1.row == Row::Home
                        || curr.row == Row::Top && old1.row == Row::Bottom))
            {
                log(5, 10.0);
            }
            
            // 9: Roll out.
            if is_roll_out(curr.finger, old1.finger) {
                log(9, -0.01);
                
            }

            // 10: Roll in.
            if is_roll_in(curr.finger, old1.finger) {
                if old1.row!= Row::Bottom&& !(
                    curr.row == Row::Top && old1.row == Row::Bottom
                    || (curr.row == Row::Bottom && old1.row == Row::Top && curr.finger != Finger::Index)
                ){
                    log(10, -0.99);
                }
                else{ //unconfortable roll in
                    
                }

            }
            if old1.finger != Finger::Thumb && ( is_roll_in2(curr.finger, old1.finger) && is_roll_in(curr.finger, old1.finger)){
                log(99,2.0);
            }
            
        }
        let old2 = match *old2 {
            Some(ref o) => o,
            None => continue,
        };
    

        // if curr.hand == old1.hand && old1.hand == old2.hand && old2.hand == old3.hand {
        //     // 13: 4 no alternation
        //     log(13, 5.2);
        // } else if curr.hand != old1.hand
        //     && old1.hand != old2.hand
        //     && old2.hand != old3.hand
        //     && curr.hand != Hand::Thumb
        //     && old1.hand != Hand::Thumb
        //     && old2.hand != Hand::Thumb
        //     && old3.hand != Hand::Thumb
        // {
            // 14: 4 alternations in a row.
        //     log(14, 0.1);
        // }
        //8: Alternation
        if curr.hand != old1.hand {
            log(8, -0.1);
        }

        if curr.hand == old1.hand && old1.hand == old2.hand {
            // 6: Roll reversal.
            if (curr.finger == Finger::Middle
                && old1.finger == Finger::Pinky
                && old2.finger == Finger::Ring)
                || curr.finger == Finger::Ring
                    && old1.finger == Finger::Pinky
                    && old2.finger == Finger::Middle
            {
                log(6, 3.0);
            }

            // 12: Twist.
            if ((curr.row == Row::Top && old1.row == Row::Home && old2.row == Row::Bottom)
            || (curr.row == Row::Bottom && old1.row == Row::Home && old2.row == Row::Top))
                && ((is_roll_out(curr.finger, old1.finger)
                    && is_roll_out(old1.finger, old2.finger))
                    || (is_roll_in(curr.finger, old1.finger)
                        && is_roll_in(old1.finger, old2.finger)))
            {
                log(12, 0.0);
            }
        }

        // 11: Long jump sandwich.
        if curr.hand == old2.hand && curr.finger == old2.finger {
            if curr.row == Row::Top && old2.row == Row::Bottom
                || curr.row == Row::Bottom && old2.row == Row::Top
            {
                log(11, 5.0);
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
        fn is_roll_out2(curr: Finger, prev: Finger) -> bool {
            match curr {
                Finger::Thumb => false,
                Finger::Index => prev == Finger::Thumb,
                Finger::Middle => prev == Finger::Thumb || prev == Finger::Index,
                Finger::Ring => prev != Finger::Pinky && prev != Finger::Ring,
                Finger::Pinky => prev != Finger::Pinky,
            }
        }
        // all roll-ins
        fn is_roll_in2(curr: Finger, prev: Finger) -> bool {
            match curr {
                Finger::Thumb => prev != Finger::Thumb,
                Finger::Index => prev != Finger::Thumb && prev != Finger::Index,
                Finger::Middle => prev == Finger::Pinky || prev == Finger::Ring,
                Finger::Ring => prev == Finger::Pinky,
                Finger::Pinky => false,
            }
        }
    }
    //result.total = *(result.hand_penalties.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
    BestLayoutsEntry {
        layout: layout.clone(),
        penalty: result,
    }
}
