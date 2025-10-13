/// Data structures and methods for creating and shuffling keyboard layouts.

extern crate rand;

use std::fmt;
use self::rand::random;

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




#[derive(Clone)]
pub struct Layer(KeyMap<char>);

#[derive(Clone)]
pub struct Layout(Layer, Layer);



pub struct LayoutPosMap([Option<KeyPress>; 128]);

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

const NUM_OF_KEYS: usize = 34;

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

	pub fn shuffle(&mut self, times: usize)
	{
		for _ in 0..times {
			loop {
				let (i, j) = Layout::shuffle_position();
				if CAN_SWAP_HALVES || (KEY_HANDS[i] == KEY_HANDS[j]){
					let Layout(ref mut lower, ref mut upper) = *self;
					
					lower.swap(i, j);
					upper.swap(i, j);
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
		let mut i = random::<usize>() % NUM_OF_KEYS;
		let mut j = random::<usize>() % NUM_OF_KEYS;

		while !SWAPPABLE_MAP[i] {
			i = random::<usize>() % NUM_OF_KEYS;
		}
		while !SWAPPABLE_MAP[j] {
			j = random::<usize>() % NUM_OF_KEYS;
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
