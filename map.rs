use std::int::*;
use std::cast;
use std::rand;
use std::rand::RngUtil;
use std::cmp::Eq;
use std::ops::{Add, Sub};
use std::vec;

#[deriving(Eq)]
pub enum Direction {
	N = 0,
	NE,
	SE,
	S,
	SW,
	NW
}

pub struct Position {
	x : int,
	y : int
}

#[deriving(Eq)]
pub enum RelativeDir {
	FORWARD,
	BACKWARD,
	RIGHT,
	LEFT
}

pub enum Action {
	RUN(RelativeDir),
	MOVE(RelativeDir),
	TURN(RelativeDir),
	MELEE(RelativeDir),
	USE,
	WAIT
}

pub trait MoveController {
	fn get_move(&mut self, cr: @mut Creature) -> Action;
}

pub struct Creature {
	map : @mut Map,
	pos : Position,
	dir : Direction,
	last_hit_time : int,
	life : int,
	controller : @MoveController,
	action : Option<Action>,
	pre_action_ticks : uint,
	post_action_ticks : uint,
	map_visible : ~[ ~[ bool ] ],
	map_known : ~[ ~[ bool ] ],
	map_height: uint,
	map_width: uint
}

pub enum Tile {
	FLOOR,
	WALL
}

static MAP_WIDTH : uint = 32;
static MAP_HEIGHT : uint = 32;

pub enum ObjectType {
	MEDKIT
}

pub struct Object {
	objecttype : ObjectType
}

pub struct Map {
	tiles : ~[ ~[ Tile ] ],
	creatures : ~[ ~[ Option<@mut Creature> ] ],
	objects : ~[ ~[ @mut ~[ ~Object ] ] ],
	width : uint,
	height : uint
}

pub trait MapView {
	fn at(&mut self, pos: Position) -> Tile;
	fn creature_at(&mut self, pos: Position) -> Option<@mut Creature>;
	fn objects_at(&mut self, pos: Position) -> @mut ~[ ~Object ];
	fn translate(&self, pos : Position) -> Position;
}

/**
 * View of the map with rotation (dir) and offset (pos)
 */
pub struct RelativeMap {
	map : @mut Map,
	pos : Position,
	dir : Direction
}

impl Action {
	fn pre_ticks(&self, cr: @mut Creature) -> uint {
		match *self {
			MOVE(BACKWARD) | RUN(BACKWARD) => 15u,
			RUN(FORWARD) => 5u,
			MOVE(_)|RUN(_) => 10u,
			TURN(_) => 5u,
			MELEE(_) => 2u,
			WAIT => 1u,
			USE => {
				if (cr.map.objects_at(cr.pos).len() != 0) {
					30u
				} else {
					0u
				}
			}
		}
	}
	fn post_ticks(&self, cr : @mut Creature) -> uint {
		match *self {
			RUN(FORWARD) => 5u,
			MOVE(_)|RUN(_) => 10u,
			TURN(_) => 5u,
			MELEE(_) => 10u,
			WAIT => 0u,
			USE => {
				if (cr.map.objects_at(cr.pos).len() != 0) {
					30u
				} else {
					0u
				}
			}
		}
	}
}

impl RelativeDir {
	fn to_int(&self) -> int {
		match *self {
			FORWARD => 0,
			BACKWARD => 3,
			RIGHT => 1,
			LEFT => 5
		}
	}
}

impl Direction {
	fn turn_mut(&mut self, rd : RelativeDir) {
		let i = rd.to_int();
		unsafe {
			*self = cast::transmute_copy(&modulo((*self as int + i), 6))
		}
	}

	// Workaround for borrowing bug
	fn turn_m(&mut self, rd : RelativeDir) -> Direction {
		let i = rd.to_int();
		unsafe {
			cast::transmute_copy(&modulo((*self as int + i), 6))
		}
	}
	fn turn(&self, rd : RelativeDir) -> Direction {
		let i = rd.to_int();
		unsafe {
			cast::transmute_copy(&modulo((*self as int + i), 6))
		}
	}

	fn turn_by_int(&self, i : int) -> Direction {
		unsafe {
			cast::transmute_copy(&modulo((*self as int + i), 6))
		}
	}

	fn relative_to(&self, dir : Direction) -> Direction {
		unsafe {
			cast::transmute_copy(&modulo(((*self as int)  - (dir as int)), 6))
		}
	}

	fn to_uint(&self) -> uint {
		unsafe {
			cast::transmute_copy(&(*self as int))
		}
	}
}

impl Eq for Position {
	fn eq(&self, p : &Position) -> bool {
		self.x == p.x && self.y == p.y
	}

	fn ne(&self, p : &Position) -> bool {
		!(self == p)
	}
}

impl Add<Position, Position> for Position {
	fn add(&self, pos : &Position) -> Position {
		Position {x: self.x + pos.x, y: self.y + pos.y }
	}
}

impl Sub<Position, Position> for Position {
	fn sub(&self, pos : &Position) -> Position {
		Position {x: self.x - pos.x, y: self.y - pos.y }
	}
}

impl Position {
	pub fn relative_to(&self, pos : Position) -> ~Position {
		~Position{ x: self.x - pos.x, y: self.y - pos.y}
	}

	// Iterate over every neighbor
	pub fn each_around(&self, up : int, down : int, left : int, right : int, f : &fn(position : Position)) {
		for range(self.y - up, self.y + down + 1) |vy| {
			for range(self.x - left, self.x + right + 1) |vx| {
				let x = vx;
				let y = vy + ((vx - self.x) >> 1);
				f (Position {x: x, y: y});
			}
		}
	}

	pub fn is_neighbor(&self, position : Position) -> bool {
		let rx = self.x - position.x;
		let ry = self.y - position.y;
		match (rx, ry) {
			(0, -1)  => true,
			(0, 1)   => true,
			(-1, 0)  => true,
			(1, 0)   => true,
			(-1, -1) => true,
			(1, 1)  => true,
			_ => {
				false
			}
		}
	}

	pub fn neighbor(&self, direction : Direction) -> Position {
		match direction {
			N => Position { x: self.x, y: self.y - 1 },
			S => Position { x: self.x, y: self.y + 1 },
			SW => Position { x: self.x - 1, y: self.y },
			NE => Position { x: self.x + 1, y: self.y },
			NW => Position { x: self.x - 1, y: self.y - 1 },
			SE => Position { x: self.x + 1, y: self.y + 1 }
		}
	}
}

fn modulo(x :int, m : int) -> int {
	let r = x % m;
	if r < 0 { r+m } else { r }
}

static PLAYER_VIEW: int = 10;

impl Creature {
	pub fn new<T: MoveController + 'static>(
			map : @mut Map, position : Position, direction : Direction,
			ctr : @T
			) -> Creature {
		Creature {
			map: map,
			last_hit_time: 1000,
			life: 3,
			controller: ctr as @MoveController,
			pos : position, dir : direction,
			action: None, pre_action_ticks: 0, post_action_ticks: 0,
			map_visible: vec::from_elem(map.width, vec::from_elem(map.height, false)),
			map_known: vec::from_elem(map.width, vec::from_elem(map.height, false)),
			map_width: map.width,
			map_height: map.height,
		}
	}

	pub fn tick(@mut self) -> bool {
		let mut redraw = false;

		self.last_hit_time += 1;

		if (self.pre_action_ticks > 0) {
			self.pre_action_ticks -= 1;
		} else {
			match (self.action) {
				Some(action) => {
					redraw = true;
					match (action) {
						RUN(d) => self.move(d),
						MOVE(d) => self.move(d),
						TURN(d) => self.turn(d),
						MELEE(d) => self.melee(d),
						USE => self.use_item(),
						WAIT => {},
					}
					self.action = None
				}
				None => {
					if (self.post_action_ticks > 0) {
						self.post_action_ticks -= 1;
					} else {
						let action = self.controller.get_move(self);
						self.action = Some(action);
						self.pre_action_ticks = action.pre_ticks(self);
						self.post_action_ticks = action.post_ticks(self);
					}
				}
			}
		}
		redraw
	}

	pub fn turn(@mut self, rd : RelativeDir) {
		self.dir.turn_mut(rd);
	}

	pub fn move(@mut self, rd : RelativeDir) {
		let d = self.dir.turn_m(rd); // workaround bug
		let pos = self.pos; // workaround bug
		let new_position = pos.neighbor(d);
		self.mark_known(new_position);
		if (self.map.at(new_position).is_passable()) {
			self.map.move_creature(self, new_position);
		}
	}

	pub fn use_item(@mut self) {
		let objs = self.map.objects_at(self.pos);

		if objs.len() == 0 {
			return;
		}
		match objs.last().objecttype {
			MEDKIT => {
				self.life = self.life + 1;
				objs.pop();
			}
		}
	}
	pub fn melee(@mut self, rd : RelativeDir) {
		let pos = self.pos; // workaround bug
		let dir = self.dir;
		let new_position = pos.neighbor(dir.turn(rd));
		match self.map.creature_at(new_position) {
			Some(cr) => {
				cr.hit();
			},
			None => {}
		}
	}

	pub fn hit(@mut self) {
		self.last_hit_time = 0;
		self.life -= 1;

		if (self.life <= 0) {
			self.die();
		}
	}

	pub fn die(@mut self) {
		self.map.remove_creature(self);
	}

	pub fn alive(&mut self) -> bool {
		self.life > 0
	}

	pub fn mark_visible(&mut self, pos : Position) {
		let p = self.map.wrap_position(pos);

		self.map_visible[p.x][p.y] = true;
	}

	pub fn mark_known(&mut self, pos : Position) {
		let p = self.map.wrap_position(pos);

		self.map_known[p.x][p.y] = true;
	}

	pub fn sees(&self, pos: Position) -> bool {
		let p = self.map.wrap_position(pos);

		self.map_visible[p.x][p.y]
	}

	pub fn knows(&self, pos: Position) -> bool {
		let p = self.map.wrap_position(pos);

		self.map_known[p.x][p.y]
	}

	pub fn position(&self) -> Position {
		self.pos
	}

	// Iterate over a rectangle in front of the Creature
	pub fn each_in_view_rect(&self, f : &fn(position : Position)) {
		Position{x:0,y:0}.each_around(PLAYER_VIEW, 2, PLAYER_VIEW, PLAYER_VIEW, f)
	}

	// Very hacky, recursive LoS algorithm
	fn do_view(&mut self, pos: Position,
		main_dir : Direction, dir : Option<Direction>, pdir : Option<Direction>, depth: uint) {
		if (depth == 0) {
			return;
		}

		self.mark_visible(pos);
		self.mark_known(pos);

		let neighbors = match (dir, pdir) {
			(Some(dir), Some(pdir)) => {
				if dir == pdir {
					~[dir]
				} else {
					~[dir, pdir]
				}
			},
			(Some(dir), None) => {
				if main_dir == dir {
					~[dir, dir.turn(LEFT), dir.turn(RIGHT)]
				} else {
					~[dir, main_dir]
				}
			},
			_ => {
				~[main_dir, main_dir.turn(LEFT), main_dir.turn(RIGHT)]
			}
		};

		if self.map.at(pos).can_see_through() {
			for neighbors.iter().advance |&d| {
				let n = pos.neighbor(d);
				match dir {
					Some(_) => {
						self.do_view(n, d, Some(d), dir, depth - 1);
					},
					None => {
						self.do_view(n, main_dir, Some(d), dir, depth - 1);
					}
				};
			}
		}
	}

	pub fn update_visibility(&mut self) {
		self.map_visible = vec::from_elem(self.map.width, vec::from_elem(self.map.height, false));

		let position = copy self.pos;
		let direction = copy self.dir;

		self.do_view(position, direction, None, None, PLAYER_VIEW as uint);
	}
}

impl Tile {
	pub fn is_wall(&self) -> bool {
		match *self {
			WALL => true,
			_ => false
		}
	}

	pub fn is_floor(&self) -> bool {
		match *self {
			FLOOR => true,
			_ => false
		}
	}

	pub fn can_see_through(&self) -> bool {
		match *self {
			WALL => false,
			_ => true
		}
	}

	pub fn is_passable(&self) -> bool {
		match *self {
			WALL => false,
			_ => true
		}
	}
}

impl MapView for Map {
	pub fn at(&mut self, pos: Position) -> Tile {
		let p = self.wrap_position(pos);
		self.tiles[p.x][p.y]
	}
	pub fn creature_at(&mut self, pos: Position) -> Option<@mut Creature> {
		let pos = self.wrap_position(pos);
		self.creatures[pos.x][pos.y]
	}
	pub fn objects_at(&mut self, pos: Position) -> @mut ~[ ~Object ] {
		let pos = self.wrap_position(pos);
		self.objects[pos.x][pos.y]
	}
	pub fn translate(&self, pos : Position) -> Position {
		pos
	}
}

fn each_in_vrect<T: MapView>(s: &mut T, cp : Position, rx : int, ry : int, f : &fn(position : Position, t: Tile)) {
	for range(-rx, rx + 1) |vx| {
		for range(-ry, ry + 1) |vy| {
			let x = cp.x + vx;
			let y = cp.y + vy + (vx >> 1);
			let p = Position {x: x as int, y: y as int};
			f(p, s.at(p));
		}
	}
}

impl Map {
	pub fn new() -> Map {
		let mut rng = rand::rng();

		let map = vec::from_fn(MAP_WIDTH, |_| {
			vec::from_fn(MAP_HEIGHT, |_| {
				if (rng.gen_int_range(0, 3) == 0) {
					WALL
				} else {
					FLOOR
				}
			})
		});

		let creatures = vec::from_fn(MAP_WIDTH, |_| {
			vec::from_fn(MAP_HEIGHT, |_| {
				None
			})
		});

		let objects = vec::from_fn(MAP_WIDTH, |_| {
			vec::from_fn(MAP_HEIGHT, |_| {
				@mut ~[]
			})
		});
		Map {
			tiles: map, creatures: creatures,
			width: MAP_WIDTH, height: MAP_HEIGHT,
			objects: objects
		}
	}

	fn wrap_position(&self, pos : Position) -> Position {
		Position {
			x: modulo(pos.x, self.width as int),
			y: modulo(pos.y, self.height as int)
		}
	}

	fn for_each_tile(&mut self, f : &fn(Position, &mut Tile)) {
		for range(0, self.width as int) |x| {
			for range(0, self.height as int) |y| {
				f(Position {x: x as int, y: y as int}, &mut self.tiles[x][y]);
			}
		}
	}

	fn for_each_creature(&mut self, f : &fn(@mut Creature)) {
		for range(0, self.width as int) |x| {
			for range(0, self.height as int) |y| {
				match (self.creatures[x][y]) {
					Some(creature) => f(creature),
					None => {}
				}
			}
		}
	}

	fn spawn_creature<T:MoveController + 'static>(@mut self, pos : Position, dir : Direction,
			controller : @T
			) -> Option<@mut Creature> {
		if (!self.at(pos).is_passable()) {
			return None;
		}
		match (self.creatures[pos.x][pos.y]) {
			Some(_) => None,
			None => {
				let c = @mut Creature::new(self, pos, dir, controller);
				self.creatures[pos.x][pos.y] = Some(c);
				Some(c)
			}
		}
	}

	pub fn spawn_object(@mut self, pos : Position, obj : ~Object) {
		if (!self.at(pos).is_passable()) {
			return;
		}
		self.objects[pos.x][pos.y].push(obj);
		//vec::append_one(self.objects[pos.x][pos.y], obj);
	}

	pub fn random_pos(&self) -> Position {
		let mut rng = rand::rng();

		Position {
			x: rng.gen_int_range(0, self.width as int),
			y: rng.gen_int_range(0, self.height as int)
		}
	}

	pub fn spawn_random_creature<T:MoveController + 'static>(
			@mut self, controller : @T
			) -> @mut Creature {

		let mut rng = rand::rng();

		let pos = self.random_pos();

		let dir = N.turn_by_int(rng.gen_int_range(0, 6));

		match (self.spawn_creature(pos, dir, controller)) {
			None => self.spawn_random_creature(controller),
			Some(creature) => creature
		}
	}

	fn move_creature(&mut self, cr : @mut Creature, pos : Position) {
		let pos = &self.wrap_position(pos);
		match (self.creatures[pos.x][pos.y]) {
			Some(_) => {},
			None => {
				self.creatures[cr.pos.x][cr.pos.y] = None;
				cr.pos = *pos;
				self.creatures[pos.x][pos.y] = Some(cr);
			}
		}
	}

	fn remove_creature(&mut self, cr : @mut Creature) {
		let pos = cr.pos;
		let pos = self.wrap_position(pos);
		self.creatures[pos.x][pos.y] = None;
	}
}

impl RelativeMap {
	pub fn new(map: @mut Map, pos : Position, dir : Direction) -> RelativeMap {
		RelativeMap{ map: map, pos: pos, dir: dir }
	}

	// Underlying map.
	pub fn base(&mut self) -> @mut Map {
		self.map
	}
}

impl MapView for RelativeMap {
	fn at(&mut self, pos: Position) -> Tile {
		let pos = self.translate(pos);
		self.map.at(pos)
	}

	fn creature_at(&mut self, pos: Position) -> Option<@mut Creature> {
		let pos = self.translate(pos);
		self.map.creature_at(pos)
	}

	pub fn objects_at(&mut self, pos: Position) -> @mut ~[ ~Object ] {
		let pos = self.translate(pos);
		self.map.objects[pos.x][pos.y]
	}
	fn translate(&self, pos : Position) -> Position {
		match self.dir {
			N => Position {
				x: self.pos.x + pos.x,
				y: self.pos.y + pos.y
			},
			S => Position {
				x: self.pos.x - pos.x,
				y: self.pos.y - pos.y
			},
			NW => Position {
				x: self.pos.x + pos.y ,
				y: self.pos.y + pos.y - pos.x
			},
			SE => Position {
				x: self.pos.x - pos.y,
				y: self.pos.y - pos.y + pos.x
			},
			NE => Position {
				x: self.pos.x + pos.x - pos.y,
				y: self.pos.y + pos.x
			},
			SW => Position {
				x: self.pos.x - pos.x + pos.y,
				y: self.pos.y - pos.x
			}
		}
	}
}
