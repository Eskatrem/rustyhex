pub use sdl::util::Rect;

use core::uint::range;
use core::cast;

enum Direction {
	N = 0,
	NE,
	SE,
	S,
	SW,
	NW
}

pub struct Position {
	x : uint,
	y : uint
}

pub struct Creature {
	mut position : Position,
	mut direction : Direction
}

enum TileType {
	WALL,
	FLOOR
}

pub struct Tile {
	mut known : bool,
	mut t : TileType
}

const MAP_WIDTH : uint = 16;
const MAP_HEIGHT : uint = 16;

pub struct Map {
	mut map : [ [ Tile * 16] * 16]
}

pub const HEX_BASE_HEIGHT: uint = 35;
pub const HEX_BASE_WIDTH: uint = 20;
pub const HEX_SIDE_WIDTH: uint = 10;
pub const HEX_BORDER_HEIGHT: uint = 2;
pub const HEX_BORDER_WIDTH: uint = 2;

pub const HEX_FULL_WIDTH: uint = HEX_BASE_WIDTH + 2 * HEX_SIDE_WIDTH + HEX_BORDER_WIDTH;
pub const HEX_FULL_HEIGHT: uint = HEX_BASE_HEIGHT + HEX_BORDER_HEIGHT;

pub impl Position {

	fn to_x(&self) -> uint {
		self.x * (HEX_BASE_WIDTH + HEX_SIDE_WIDTH)

	}
	fn to_y(&self) -> uint {
		self.y * HEX_BASE_HEIGHT +
			if (self.x % 2) != 0 { HEX_BASE_HEIGHT / 2 } else { 0 }
	}
	fn to_rect(&self) -> Rect {
		Rect {
			x: self.to_x() as i16, y: self.to_y() as i16,
			w: (HEX_BASE_WIDTH + 2 * HEX_SIDE_WIDTH + HEX_BORDER_WIDTH) as u16,
			h: (HEX_BASE_HEIGHT + HEX_BORDER_HEIGHT) as u16
		}
	}

	pure fn is_neighbor(&self, position : Position) -> bool {
		let rx = self.x - position.x;
		let ry = self.y - position.y;
		if (self.x % 2) != 0 {
			match (rx, ry) {
				(0, -1) => true,
				(0, 1) => true,
				(-1, 0) => true,
				(1, 0) => true,
				(-1, -1) => true,
				(1, -1) => true,
				_ => {
						false
				}
			}
		} else {
			match (rx, ry) {
				(0, -1) => true,
				(0, 1) => true,
				(-1, 0) => true,
				(1, 0) => true,
				(-1, 1) => true,
				(1, 1) => true,
				_ => {
						false
				}
			}
		}
	}

	fn neighbor(&self, direction : Direction) -> Position {
		match direction {
			N => Position { x: self.x, y: self.y - 1 },
			S => Position { x: self.x, y: self.y + 1 },
			NE => Position { x: self.x + 1 , y: self.y - ((self.x + 1) % 2)},
			SE => Position { x: self.x + 1, y: self.y + (self.x % 2) },
			NW => Position { x: self.x - 1 , y: self.y - ((self.x + 1) % 2)},
			SW => Position { x: self.x - 1, y: self.y + (self.x % 2) }
		}
	}
}

pub impl Creature {
	static fn new(position : Position, direction : Direction) -> ~Creature {
		~Creature {
			position : position, direction : direction
		}
	}

	fn turn_right(&self) -> () {
		unsafe {
			self.direction = cast::reinterpret_cast(&((self.direction as int + 1) % 6))
		}
	}
	fn turn_left(&self) -> () {
		unsafe {
			self.direction = cast::reinterpret_cast(&((self.direction as int - 1) % 6))
		}
	}

	fn move_forward(&mut self) {
		self.position = self.position.neighbor(self.direction);
	}

	fn sees(&self, position : Position) -> bool {
		self.position.is_neighbor(position)
	}
}

pub impl Tile {
	fn is_known(&self) -> bool {
		self.known
	}

	fn is_wall(&self) -> bool {
		match self.t {
			WALL => true,
			_ => false
		}
	}
}

pub impl Map {
	static fn new() -> ~mut Map {
		let map = ~mut Map {
			map: [  [ Tile {known: false, t: WALL}, .. 16], .. 16]
		};
		let rng = rand::Rng();

		do map.each() | _ : Position, tile : &mut Tile | {
			tile.t = if (rng.gen_int_range(0, 10) == 0) {
				WALL
			} else {
				FLOOR
			}
		}
		map
	}

	fn at(&self, position : Position) -> Tile {
		copy self.map[position.x][position.y]
	}

	fn each(&mut self, f : fn(position : Position, &mut Tile)) {
		for range(0u, MAP_WIDTH) |x| {
			for range(0u, MAP_HEIGHT) |y| {
				f(Position {x: x, y: y}, &mut self.map[x][y]);
			}
		}
	}
	

}
