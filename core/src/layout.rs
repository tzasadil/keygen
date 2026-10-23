/// Data structures and methods for creating and shuffling keyboard layouts.


use std;
use std::fmt;
use rand::random_range;

/* ----- *
 * TYPES *
 * ----- */

// KeyMap format:
//    LEFT HAND   |    RIGHT HAND
//  0  1  2  3  4 |  5  6  7  8  9 10
// 11 12 13 14 15 | 16 17 18 19 20 21 
// 22 23 24 25 26 | 27 28 29 30 31
//
//             32 | 33 (thumb keys)

pub type KeyMap<T> =  [T; NUM_OF_KEYS];




#[derive(Clone, PartialEq)]
pub struct Layer(KeyMap<char>);

#[derive(Clone, PartialEq)]
pub struct Layout(Layer, Layer);



pub struct LayoutPosMap([Option<KeyPress>; 128]);

#[derive(Debug, PartialEq)]
pub enum ParseLayoutError
{
	RowCount(usize),
	RowLength { row: usize, expected: usize, found: usize },
	HandSplit { row: usize },
	InvalidKey(String),
	DuplicateKey(char),
	// The corpus is letters only, so a missing letter would fake a low score.
	MissingLetter(char),
}

#[derive(Clone)]
pub struct LayoutShuffleMask(KeyMap<bool>);

#[derive(Clone, Copy, PartialEq)]
pub enum Finger 
{
	Thumb,
	Index,
	Middle,
	Ring,
	Pinky,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Hand
{
	Left,
	Right,
	Thumb
}

#[derive(Clone, Copy, PartialEq)]
pub enum Row
{
	Top,
	Home,
	Bottom,
	Thumb,
}

#[derive(Clone, Copy)]
pub struct KeyPress
{
	pub kc:     char,
	pub pos:    usize,
	pub finger: Finger,
	pub hand:   Hand,
	pub row:    Row,
	pub center: bool,
}

/* ------- *
* STATICS *
* ------- */

pub const NUM_OF_KEYS: usize = 34;

// pub static _: Layout = Layout(
//     Layer([ '', '', '',  '', '',     '', '', '', '', '', '',
//             '', '', '',  '', '',     '', '', '', '', '', '',
//             '', '', '',  '', '',     '', '', '', '', '',
//                              '',     '']),
//     Layer([ '', '', '',  '', '',     '', '', '', '', '', '',
//             '', '', '',  '', '',     '', '', '', '', '', '',
//             '', '', '',  '', '',     '', '', '', '', '',
//                              '',     ''])
// );



pub static DABEST: Layout = Layout(
	Layer([	'b', 'y', 'o', 'u', '/',   'f', 'g', 'd', 'l', 'V', '-',
			'h', 'i', 'e', 'a', ',',   'd', 't', 's', 'n', 'r', '\'',
			'q', 'x', 'z', '.', ';',   'k', 'w', 'c', 'm', 'j',
	              				'\0',  ' ']),
	Layer([	'B', 'Y', 'O', 'U', '?',   'F', 'G', 'D', 'L', 'V', '_',
			'H', 'I', 'E', 'A', '<',   'D', 'T', 'S', 'N', 'R', '"',
			'Q', 'X', 'Z', '>', ':',   'K', 'W', 'C', 'M', 'J',
	              				'\0',  '\n']));

pub static X1: Layout = Layout(
	Layer([	'k', 'y', 'o', 'u', '@',   'c', 'd', 'l', 'p', 'v', '-',
			'h', 'i', 'e', 'a', 'f',   'g', 's', 't', 'n', 'r', ';',
			'>', 'x', 'q', '&', '*',   'w', 'z', 'm', 'b', 'j',
								'(',  ' ']),
	Layer([	'B', 'Y', 'O', 'U', 'F',   'G', 'P', 'D', 'L', 'V', '_',
			'H', 'I', 'E', 'A', '#',   'M', 'S', 'T', 'N', 'R', ':',
			'?', 'X', 'Q', '<', '>',   'W', 'C', 'K', 'J', 'Z',
			')',  '\n']));




// The layout I type on every day. Scores in the README come from it.
pub static MY_LAYOUT: Layout = Layout(
	Layer([	'm', 'l', 'd', 'p', 'f',   '\'', 'u', 'o', 'y', 'k', '/',
			'r', 'n', 't', 's', 'v',   'g', 'a', 'e', 'i', 'h', 'x',
			'z', 'j', 'w', 'c', 'b',   'q', ',', '.', '=', ';',
	              				'\0',  ' ']),
	Layer([	'M', 'L', 'D', 'P', 'F',   '"', 'U', 'O', 'Y', 'K', '?',
			'R', 'N', 'T', 'S', 'V',   'G', 'A', 'E', 'I', 'H', 'X',
			'Z', 'J', 'W', 'C', 'B',   'Q', '<', '>', '+', ':',
	              				'\0',  '\n']));

pub static QWERTY_LAYOUT: Layout = Layout(
	Layer([	'q', 'w', 'e', 'r', 't',   'y', 'u', 'i', 'o', 'p', '-',
			'a', 's', 'd', 'f', 'g',   'h', 'j', 'k', 'l', ';', '\'',
			'z', 'x', 'c', 'v', 'b',   'n', 'm', ',', '.', '/',
	              				'\0',  ' ']),
	Layer([	'Q', 'W', 'E', 'R', 'T',   'Y', 'U', 'I', 'O', 'P', '_',
			'A', 'S', 'D', 'F', 'G',   'H', 'J', 'K', 'L', ':', '"',
			'Z', 'X', 'C', 'V', 'B',   'N', 'M', '<', '>', '?',
	              				'\0',  '\n']));

pub static DVORAK_LAYOUT: Layout = Layout(
	Layer([	'\'', ',', '.', 'p', 'y',  'f', 'g', 'c', 'r', 'l', '/',
			'a', 'o', 'e', 'u', 'i',   'd', 'h', 't', 'n', 's', '-',
			';', 'q', 'j', 'k', 'x',   'b', 'm', 'w', 'v', 'z',
	              				'\0',  ' ']),
	Layer([	'"', ',', '.', 'P', 'Y',   'F', 'G', 'C', 'R', 'L', '?',
			'A', 'O', 'E', 'U', 'I',   'D', 'H', 'T', 'N', 'S', '_',
			':', 'Q', 'J', 'K', 'X',   'B', 'M', 'W', 'V', 'Z',
	              				'\0',  '\n']));

pub static COLEMAK_LAYOUT: Layout = Layout(
	Layer([	'q', 'w', 'f', 'p', 'g',   'j', 'l', 'u', 'y', ';', '-',
			'a', 'r', 's', 't', 'd',   'h', 'n', 'e', 'i', 'o', '\'',
			'z', 'x', 'c', 'v', 'b',   'k', 'm', ',', '.', '/',
								'\0',  ' ']),
	Layer([	'Q', 'W', 'F', 'P', 'G',   'J', 'L', 'U', 'Y', ':', '_',
			'A', 'R', 'S', 'T', 'D',   'H', 'N', 'E', 'I', 'O', '"',
			'Z', 'X', 'C', 'V', 'B',   'K', 'M', '<', '>', '?',
								'\0',  '\n']));


pub static MTGAP_LAYOUT: Layout = Layout(
	Layer([	'y', 'p', 'o', 'u', '-',    'v', 'd', 'l', 'c', 'w', 'q',
			'i', 'n', 'e', 'a', ',',    'm', 'h', 't', 's', 'r', 'x',
			'(', 'b', '\'', '.', '_',   'z', 'f', 'k', 'g', 'j',
	              				 '(',   ' ']),
	Layer([	'Y', 'P', 'O', 'U', ':',   'B', 'D', 'L', 'C', 'K', 'J',
			'I', 'N', 'E', 'A', ';',   'M', 'H', 'T', 'S', 'R', 'V',
			'&', '?', '*', '=', '<',   '>', 'F', 'W', 'G', 'X',
								'Z',   '\n']));
			
pub static CURRENT: Layout = Layout(
	Layer([	'k', 'y', 'o', 'u', '@',   'b', 'p', 'd', 'l', 'w', '-',
			'h', 'i', 'e', 'a', 'f',   'm', 's', 't', 'n', 'r', ';',
			'>', 'x', 'q', ',', '*',   'v', 'c', 'g', 'j', 'z',
								'(',  ' ']),
	Layer([	'B', 'Y', 'O', 'U', 'F',   'G', 'P', 'D', 'L', 'V', '_',
			'H', 'I', 'E', 'A', '#',   'M', 'S', 'T', 'N', 'R', ':',
			'?', 'X', 'Q', '<', '>',   'W', 'C', 'K', 'J', 'Z',
			')',  '\n']));		
pub static CURRENT2: Layout = Layout(
	Layer([	'k', 'y', 'o', 'u', '@',   'b', 'p', 'd', 'l', 'v', '-',
			'h', 'i', 'e', 'a', 'f',   'm', 's', 't', 'n', 'r', ';',
			'>', 'x', 'q', ',', '*',   'g', 'c', 'w', 'j', 'z',
								'(',  ' ']),
	Layer([	'B', 'Y', 'O', 'U', 'F',   'G', 'P', 'D', 'L', 'V', '_',
			'H', 'I', 'E', 'A', '#',   'M', 'S', 'T', 'N', 'R', ':',
			'?', 'X', 'Q', '<', '>',   'W', 'C', 'K', 'J', 'Z',
			')',  '\n']));		
pub static TESTING: Layout = Layout(
	Layer([	'k', 'y', 'o', 'u', 'q',   'g', 'b', 'd', 'l', 'p', '-',
			'h', 'i', 'e', 'a', 'f',   'm', 's', 't', 'n', 'r', ';',
			'>', 'x', '@', '&', '*',   'w', 'c', 'v', 'z', 'j',
								'(',  ' ']),
	Layer([	'B', 'Y', 'O', 'U', 'F',   'G', 'P', 'D', 'L', 'V', '_',
			'H', 'I', 'E', 'A', '#',   'M', 'S', 'T', 'N', 'R', ':',
			'?', 'X', 'Q', '<', '>',   'W', 'C', 'K', 'J', 'Z',
			')',  '\n']));

pub static TESTING2: Layout = Layout(
	Layer([	'k', 'y', 'o', 'u', 'q',   'f', 'p', 'd', 'l', 'm', '-',
			'h', 'i', 'e', 'a', 'g',   'v', 's', 't', 'n', 'r', ';',
			'>', 'x', '@', '&', '*',   'b', 'c', 'w', 'j', 'z',
								'(',  ' ']),
	Layer([	'B', 'Y', 'O', 'U', 'F',   'G', 'P', 'D', 'L', 'V', '_',
			'H', 'I', 'E', 'A', '#',   'M', 'S', 'T', 'N', 'R', ':',
			'?', 'X', 'Q', '<', '>',   'W', 'C', 'K', 'J', 'Z',
			')',  '\n']));
pub static TESTING3: Layout = Layout(
	Layer([	'k', 'y', 'o', 'u', 'q',   'f', 'p', 'd', 'l', 'm', '-',
			'h', 'i', 'e', 'a', 'w',   'v', 's', 't', 'n', 'r', ';',
			'>', 'x', '@', '&', '*',   'b', 'c', 'g', 'j', 'z',
								'(',  ' ']),
	Layer([	'B', 'Y', 'O', 'U', 'F',   'G', 'P', 'D', 'L', 'V', '_',
			'H', 'I', 'E', 'A', '#',   'M', 'S', 'T', 'N', 'R', ':',
			'?', 'X', 'Q', '<', '>',   'W', 'C', 'B', 'J', 'Z',
			')',  '\n']));

		

pub static SWAPPABLE_MAP: KeyMap<bool>= [
	false,  false,  false,  false,  false,       	true,  true,  false,  false,  false,  false,
	false,  false,  false,  false,  false,      		true,  false,	false,false,  false,  false,
	false,  false,  false,  false,  false,  		true, 	true, 	false,  false,  false,
	false, 		false 
	];
	
const  CAN_SWAP_HALVES: bool = true;
// pub static SWAPPABLE_MAP: KeyMap<bool>= [
// 	true,  true,  true,  true,  true,       	true,  true,  false,  true,  true,  false,
// 	false,  false,  true,  true,  true,       true,  false,  false,  false,  true,  false,
// 	true,  true,  true,  true,  true,       	true,  true,  false,  true,  true,
// 	false, 		false
// 	];
	
// pub static SWAPPABLE_MAP: KeyMap<bool>= [
// 	true,  false,  false,  false,  false,       true,  true,  true,  true,  true,  false,
// 	false,  false,  false,  false,  true,       true,  true,  true,  true,  true,  false,
// 	false,  false,  false,  false,  false,   	true,  true,  true,  true,  true,
// 	false, 		false
// 	];

// ALL TRUE	
// pub static SWAPPABLE_MAP: KeyMap<bool>= [
// 		true,  true,  true,  true,  true,       true,  true,  true,  true,  true,  false,
// 		true,  true,  true,  true,  true,       true,  true,  true,  true,  true,  false,
// 		true,  false,  false,  true,  true,   	true,  true,  true,  true,  true,
// 		false, 		false
		// ];

// pub static SWAPPABLE_MAP: KeyMap<bool>= [
// 	false,  false,  false,  false,  false,       true,  true,  false,  true,  true,  false,
// 	false,  false,  false,  false,  false,       true,  true,  false,  false,  false,  false,
// 	false,  false,  false,  false,  false,   	 true,  true,  false,  true,  true,
// 		false, 		false
// 		];
	

static KEY_FINGERS: KeyMap<Finger> = [
	Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index,    Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky, Finger::Pinky,
	Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index,    Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky, Finger::Pinky,
	Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index,    Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky,
	Finger::Thumb, Finger::Thumb
];
static KEY_HANDS: KeyMap<Hand> = [
	Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,    Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right,
	Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,    Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right,
	Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,    Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right,
	Hand::Thumb, Hand::Thumb
];
static KEY_ROWS: KeyMap<Row> = [
	Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,       Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,
	Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,      Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,
	Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom,    Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom,
	Row::Thumb, Row::Thumb
];
static KEY_CENTER_COLUMN: KeyMap<bool> = [
	false, false, false, false, true,    true, false, false, false, false, false,
	false, false, false, false, true,    true, false, false, false, false, false,
	false, false, false, false, true,    true, false, false, false, false,
	false, false
];

pub static KP_NONE: Option<KeyPress> = None;

static ROW_LENGTHS: [usize; 3] = [11, 11, 10];

static LAYOUT_FILE_IDXS: KeyMap<usize> = [
	0,  1,  2,  3,  4,     6,  7,  8,  9,  10, 11,
	13, 14, 15, 16, 17,    19, 20, 21, 22, 23, 24,
	26, 27, 28, 29, 30,    32, 33, 34, 35, 36, 
	37, 38];

/* ----- *
 * IMPLS *
 * ----- */

impl Layout
{
	pub fn from_string(s: &str)
	-> Layout
	{
		let s: Vec<char> = s.chars().collect();
		let mut lower: [char; 34] = ['\0'; 34];
		let mut upper: [char; 34] = ['\0'; 34];
		
		for i in 0..34 {
			let file_i = LAYOUT_FILE_IDXS[i];
			lower[i] = *s.get(file_i).unwrap_or(&'\0');
			upper[i] = *s.get(file_i + 40).unwrap_or(&'\0');
		}

		Layout(Layer(lower), Layer(upper))
	}

	// Same rows as `Display` without the thumb line: keys separated by
	// whitespace, optional "|" between the hands. Letters may be either case.
	// Thumbs match the reference layouts:
	// space on the right thumb, newline on shifted space. Shifted punctuation
	// stays unmapped, as `normalize_text` drops punctuation anyway.
	pub fn parse(text: &str)
	-> Result<Layout, ParseLayoutError>
	{
		let rows: Vec<Vec<&str>> = text.lines()
			.map(|line| line.split_whitespace().collect::<Vec<_>>())
			.filter(|keys| !keys.is_empty())
			.collect();
		if rows.len() != ROW_LENGTHS.len() {
			return Err(ParseLayoutError::RowCount(rows.len()));
		}

		let mut lower = ['\0'; NUM_OF_KEYS];
		let mut i = 0;
		for (row, (keys, &expected)) in rows.iter().zip(ROW_LENGTHS.iter()).enumerate() {
			// A misplaced "|" would silently shift keys to the other hand.
			match keys.iter().position(|k| *k == "|") {
				Some(5) | None => {},
				Some(_) => return Err(ParseLayoutError::HandSplit { row: row + 1 }),
			}
			let keys: Vec<&str> = keys.iter().cloned().filter(|k| *k != "|").collect();
			// Checked per row, so 10/11/11 keys cannot shift into the wrong rows.
			if keys.len() != expected {
				return Err(ParseLayoutError::RowLength { row: row + 1, expected: expected, found: keys.len() });
			}
			for key in keys {
				let mut chars = key.chars();
				let c = match (chars.next(), chars.next()) {
					(Some(c), None) if c.is_ascii_graphic() => c.to_ascii_lowercase(),
					_ => return Err(ParseLayoutError::InvalidKey(key.to_string())),
				};
				if lower[..i].contains(&c) {
					return Err(ParseLayoutError::DuplicateKey(c));
				}
				lower[i] = c;
				i += 1;
			}
		}
		if let Some(c) = (b'a'..=b'z').map(char::from).find(|c| !lower.contains(c)) {
			return Err(ParseLayoutError::MissingLetter(c));
		}

		lower[33] = ' ';
		let mut upper = lower;
		for c in upper.iter_mut() {
			*c = c.to_ascii_uppercase();
		}
		upper[33] = '\n';
		Ok(Layout(Layer(lower), Layer(upper)))
	}

	// Unshifted keys in `KeyMap` order.
	pub fn base_keys(&self)
	-> &KeyMap<char>
	{
		let Layout(Layer(ref keys), _) = *self;
		keys
	}

	// Swaps both layers, so a key keeps its shifted character.
	pub fn swap(&mut self, i: usize, j: usize)
	{
		let Layout(lower, upper) = self;
		lower.swap(i, j);
		upper.swap(i, j);
	}

	pub fn shuffle(&mut self, times: usize)
	{
		for _ in 0..times {
			loop {
				let (i, j) = Layout::shuffle_position();
				if CAN_SWAP_HALVES || (KEY_HANDS[i] == KEY_HANDS[j]){
					self.swap(i, j);
					break;
				}
			}
		}
	}

	pub fn get_position_map(&self) -> LayoutPosMap
	{
		let Layout(ref lower, ref upper) = *self;
		let mut map = [None; 128];
		lower.fill_position_map(&mut map);
		upper.fill_position_map(&mut map);

		LayoutPosMap(map)
	}

	fn  shuffle_position() -> (usize, usize)
	{
		let mut i = random_range(0..NUM_OF_KEYS);
		let mut j = random_range(0..NUM_OF_KEYS);

		while !SWAPPABLE_MAP[i] {
			i = random_range(0..NUM_OF_KEYS);
		}
		while !SWAPPABLE_MAP[j] {
			j = random_range(0..NUM_OF_KEYS);
		}
		(i,j)
	}
}

impl Layer
{
	fn swap(&mut self, i: usize, j: usize)
	{
		let Layer(ref mut layer) = *self;
		let temp = layer[i];
		layer[i] = layer[j];
		layer[j] = temp;
	}

	fn fill_position_map(&self, map: &mut [Option<KeyPress>; 128])
	{
		let Layer(ref layer) = *self;
		for (i, c) in layer.into_iter().enumerate() {
			if *c < (128 as char) {
				map[*c as usize] = Some(KeyPress {
					kc: *c,
					pos: i,
					finger: KEY_FINGERS[i],
					hand: KEY_HANDS[i],
					row: KEY_ROWS[i],
					center: KEY_CENTER_COLUMN[i],
				});
			}
		}
	}
}

impl LayoutPosMap
{
	pub fn get_key_position(&self, kc: char)
	-> &Option<KeyPress>
	{
		let LayoutPosMap(ref map) = *self;
		if kc < (128 as char) {
			&map[kc as usize]
		} else {
			&KP_NONE
		}
	}
}


impl fmt::Display for ParseLayoutError
{
	fn fmt(&self, f: &mut fmt::Formatter)
	-> fmt::Result
	{
		match *self {
			ParseLayoutError::RowCount(found) =>
				write!(f, "expected {} rows, found {}", ROW_LENGTHS.len(), found),
			ParseLayoutError::RowLength { row, expected, found } =>
				write!(f, "row {}: expected {} keys, found {}", row, expected, found),
			ParseLayoutError::HandSplit { row } =>
				write!(f, "row {}: put | after the 5th key", row),
			ParseLayoutError::InvalidKey(ref key) =>
				write!(f, "invalid key \"{}\": use one printable ASCII character", key),
			ParseLayoutError::DuplicateKey(c) =>
				write!(f, "key '{}' appears twice", c),
			ParseLayoutError::MissingLetter(c) =>
				write!(f, "letter '{}' has no key", c),
		}
	}
}

impl std::error::Error for ParseLayoutError {}

impl fmt::Display for Layout
{
	fn fmt(&self, f: &mut fmt::Formatter)
	-> fmt::Result
	{
		let Layout(ref lower, _) = *self;
		lower.fmt(f)
	}
}

impl fmt::Display for Layer
{
	fn fmt(&self, f: &mut fmt::Formatter)
	-> fmt::Result
	{
		let Layer(ref layer) = *self;
		write!(f, "{} {} {} {} {} | {} {} {} {} {} {}
{} {} {} {} {} | {} {} {} {} {} {}
{} {} {} {} {} | {} {} {} {} {}
        {} | {}",
			layer[0], layer[1], layer[2], layer[3], layer[4],
			layer[5], layer[6], layer[7], layer[8], layer[9], layer[10],
			layer[11], layer[12], layer[13], layer[14], layer[15],
			layer[16], layer[17], layer[18], layer[19], layer[20], layer[21],
			layer[22], layer[23], layer[24], layer[25], layer[26],
			layer[27], layer[28], layer[29], layer[30], layer[31],
			layer[32], layer[33])
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::penalty::{calculate_penalty, prepare_quartad_list};

	#[test]
	fn parsed_qwerty_scores_like_the_static_layout() {
		let parsed = Layout::parse("q w e r t | y u i o p -\na s d f g | h j k l ; '\nz x c v b | n m , . /").unwrap();
		// One quartad list for both, so the f64 sums run in the same order.
		let quartads = prepare_quartad_list("the quick brown fox\njumps over the lazy dog\n");
		let expected = calculate_penalty(&quartads, &QWERTY_LAYOUT).penalty;
		let actual = calculate_penalty(&quartads, &parsed).penalty;
		assert_eq!(actual.keys, expected.keys);
		assert_eq!(actual.total, expected.total);
	}

	#[test]
	fn reference_layouts_reproduce_readme_scores() {
		let corpus = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../corpus/english.txt")).unwrap();
		let quartads = prepare_quartad_list(&crate::penalty::normalize_text(&corpus));
		let score = |layout| {
			let penalty = calculate_penalty(&quartads, layout).penalty;
			penalty.total / penalty.len as f64
		};
		assert!((score(&QWERTY_LAYOUT) - 0.663).abs() < 0.0005);
		assert!((score(&MY_LAYOUT) - 0.313).abs() < 0.0005);
	}

	#[test]
	fn swap_moves_the_shifted_character_along() {
		let mut layout = QWERTY_LAYOUT.clone();
		layout.swap(0, 1);
		let map = layout.get_position_map();
		let pos = |c| map.get_key_position(c).as_ref().map(|k| k.pos);
		assert_eq!((pos('q'), pos('Q'), pos('w'), pos('W')), (Some(1), Some(1), Some(0), Some(0)));
		layout.swap(1, 0);
		assert!(layout == QWERTY_LAYOUT);
	}

	#[test]
	fn parse_rejects_invalid_layouts() {
		let parse = |text: &str| Layout::parse(text).err();
		let rows = "q w e r t | y u i o p -\na s d f g | h j k l ; '\nz x c v b | n m , . /";
		assert_eq!(parse(&format!("{}\n0 | 1", rows)), Some(ParseLayoutError::RowCount(4)));
		assert_eq!(parse("q w e r t | y u i o p\na s d f g | h j k l ; '\nz x c v b | n m , . / -"),
			Some(ParseLayoutError::RowLength { row: 1, expected: 11, found: 10 }));
		assert_eq!(parse(&rows.replace("t |", "| t")), Some(ParseLayoutError::HandSplit { row: 1 }));
		assert_eq!(parse(&rows.to_uppercase()), None);
		assert_eq!(parse(&rows.replace("-", "ab")), Some(ParseLayoutError::InvalidKey("ab".to_string())));
		assert_eq!(parse(&rows.replace("-", "q")), Some(ParseLayoutError::DuplicateKey('q')));
		assert_eq!(parse(&rows.replace("z", "1")), Some(ParseLayoutError::MissingLetter('z')));
	}
}
