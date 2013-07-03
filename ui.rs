use std::result;

use sdl;
use sdl::video;
use sdl::img;
use sdl::event;
use sdl::Rect;

use map;
use map::MapView;

/* replace with something more Rusty
 * in the future */
use std::libc::{c_int};
pub extern {
	fn usleep(n : c_int) -> c_int;
}

static SCREEN_WIDTH: uint = 800;
static SCREEN_HEIGHT: uint = 600;
static SCREEN_BPP: uint = 32;

static HEX_WIDTH: uint = 66;
static HEX_HEIGHT: uint = 56;
static HEX_SIDE_WIDTH: uint = 16;
static HEX_BORDER_WIDTH:  uint = 5;
static HEX_BORDER_HEIGHT: uint = 5;

static HEX_FULL_WIDTH: uint = HEX_WIDTH + 2 * HEX_BORDER_WIDTH;
static HEX_FULL_HEIGHT: uint = HEX_HEIGHT + 2 * HEX_BORDER_HEIGHT;

struct Sprite {
	x : uint,
	y : uint
}

struct View {
	x_offset : int,
	y_offset : int
}

pub struct UI {
	screen : ~video::Surface,
	player : Option<@mut map::Creature>,
	tiles : ~video::Surface,
	view : ~View,
	exit : bool
}

impl map::Position {
	fn to_pix_x(&self) -> int {
		self.x * ((HEX_WIDTH - HEX_SIDE_WIDTH) as int) + HEX_BORDER_WIDTH as int
	}

	fn to_pix_y(&self) -> int {
		self.y * (HEX_HEIGHT as int)
		- (self.x  * (HEX_HEIGHT as int)) / 2 + HEX_BORDER_HEIGHT as int
	}

	fn to_pix_cx(&self) -> int {
		self.to_pix_x() + (HEX_FULL_WIDTH as int) / 2
	}

	fn to_pix_cy(&self) -> int {
		self.to_pix_y() + (HEX_FULL_HEIGHT as int) / 2
	}

	fn to_rect(&self) -> Rect {
		Rect {
			x: self.to_pix_x() as i16, y: self.to_pix_y() as i16,
			w: (HEX_WIDTH + 2 * HEX_BORDER_WIDTH) as u16,
			h: (HEX_HEIGHT + 2 * HEX_BORDER_HEIGHT) as u16
		}
	}
}

impl Sprite {
	fn for_tile(tile : map::Tile, visible : bool) -> Sprite {
		let mut spr = match tile {
				map::FLOOR => Sprite{ x: 0, y: 1 },
				map::WALL => Sprite{ x: 0, y: 2 }
			};

		if (!visible) {
			spr.x += 1;
		}
		spr
	}
	fn for_creature(dir : map::Direction) -> Sprite {
		Sprite{ x: dir.to_uint(), y: 3 }
	}

	fn for_object(obj : &map::Object) -> Sprite {
		match obj.objecttype {
			map::MEDKIT => Sprite{ x: 2, y: 0 }
		}
	}
	fn for_hit() -> Sprite {
		Sprite{ x: 0, y: 0 }
	}

	fn human() -> Sprite {
		Sprite{ x: 1, y: 0 }
	}

	fn to_rect(&self) -> Rect {
		Rect {
			x: (HEX_FULL_WIDTH * self.x) as i16,
			y: (HEX_FULL_HEIGHT * self.y) as i16,
			w: HEX_FULL_WIDTH as u16,
			h: HEX_FULL_HEIGHT as u16
		}
	}
}

fn load_or_die(file : ~str) -> ~video::Surface {
	match img::load(&Path([~"data/", copy file, ~".png"].concat())) {
		result::Ok(image) => {
			image
		},
		result::Err(str) => {
			fail!(str);
		}
	}
}

impl View {
	fn new(x : int, y : int) -> View {
		View{ x_offset: x, y_offset: y }
	}

	fn draw(&self, screen: &video::Surface, pos : map::Position, surface : &video::Surface) {
		let mut drect = pos.to_rect();
		drect.x += self.x_offset as i16;
		drect.y += self.y_offset as i16;
		if !screen.blit_rect(
				surface,
				Some(Rect {
					x: 0, y: 0,
					w: HEX_FULL_WIDTH as u16,
					h: HEX_FULL_HEIGHT as u16
				}),
				Some(drect)
		) { fail!(~"Failed blit_surface_rect") }
	}

	fn draw_sprite(&self, dsurf: &video::Surface, ssurf: &video::Surface,
		pos : map::Position, sprite : Sprite) {
		let mut drect = pos.to_rect();
		let srect = sprite.to_rect();

		drect.x += self.x_offset as i16;
		drect.y += self.y_offset as i16;

		if !dsurf.blit_rect(
				ssurf,
				Some(srect),
				Some(drect)
		) { fail!(~"Failed blit_surface_rect") }
	}
}

impl UI {
	pub fn new() -> UI {
		sdl::init(&[sdl::InitEverything]);
		img::init([img::InitPNG]);

		sdl::wm::set_caption("rustyhex", "rustyhex");

		let screen = match video::set_video_mode(
					SCREEN_WIDTH as int, SCREEN_HEIGHT as int, SCREEN_BPP as int,
					&[],&[video::DoubleBuf]
					) {
				result::Ok(image) => {
					image
				},
				result::Err(str) => {
					fail!(str);
				}
			};

		let tiles = load_or_die(~"tiles");

		UI {
			screen: screen,
			player: None,
			exit: false,
			view: ~View {
			  x_offset: (SCREEN_WIDTH - HEX_FULL_WIDTH) as int / 2,
			  y_offset: (SCREEN_HEIGHT - HEX_FULL_HEIGHT) as int * 7 / 8
			},
			tiles: tiles
		}
	}

	pub fn set_player(&mut self, p : @mut map::Creature) {
		self.player = Some(p);
	}

	pub fn update(&mut self) {

		let player = match self.player {
			Some(p) => p,
			None => {
				return;
			}
		};

		self.screen.fill(video::RGB(0, 0, 0));

		let mut rm = map::RelativeMap::new(player.map, player.pos, player.dir);

		do player.each_in_view_rect() | pos : map::Position | {
			let tpos = rm.translate(pos);
			let base = rm.base();
			if player.knows(tpos) {
				let t = base.at(tpos);
				let sprite = Sprite::for_tile(t, player.sees(tpos));
				self.view.draw_sprite(self.screen, self.tiles, pos, sprite);

				if player.sees(tpos) {
					let objs = base.objects_at(tpos);
					for objs.iter().advance |&obj| {
						self.view.draw_sprite(self.screen, self.tiles, pos, Sprite::for_object(obj));
					}

					match base.creature_at(tpos) {
						Some(creature) => {
							if (creature.last_hit_time < 8) {
								let sprite = Sprite::for_hit();
								self.view.draw_sprite(self.screen, self.tiles, pos, sprite);
							}
							let d = player.dir; // workarounds
							let cd = creature.dir;
							let d = cd.relative_to(d);
							let sprite = Sprite::for_creature(d);
							self.view.draw_sprite(self.screen, self.tiles, pos, sprite);
						},
						None => {}
					};
				}
			}
		}

		if (player.alive()) {
			self.view.draw_sprite(self.screen, self.tiles, map::Position {x:0, y:0}, Sprite::human());
		}

		self.screen.flip();

		unsafe {
			usleep(1000);
		}
	}

	pub fn keyevent_to_action(&mut self, key : &event::Key, m : &[event::Mod] ) -> Option<map::Action> {
		let attack = m.contains(&event::LCtrlMod);
		let strafe = m.contains(&event::LShiftMod);
		let dir = match *key {
			event::KKey | event::UpKey => {
				Some(map::FORWARD)
			},
			event::JKey | event::DownKey => {
				Some(map::BACKWARD)
			},
			event::HKey | event::LeftKey => {
				Some(map::LEFT)
			},
			event::LKey | event::RightKey => {
				Some(map::RIGHT)
			},
			_ => None
		};
		match *key {
			event::EscapeKey => {
				self.exit = true;
				return Some(map::WAIT);
			},
			event::PeriodKey | event::CommaKey => {
				return Some(map::WAIT);
			},
			event::UKey => {
				return Some(map::USE)
			},
			_ => {}
		};
		match (dir, strafe, attack) {
			(Some(d), _, true) => {
				Some(map::MELEE(d))
			},
			(Some(d), true, _) => {
				if (d == map::FORWARD) {
					Some(map::RUN(d))
				} else {
					Some(map::MOVE(d))
				}
			},
			(Some(d), false, _) => {
				match (d) {
					map::FORWARD|map::BACKWARD => {
						Some(map::MOVE(d))
					},
					__=> {
						Some(map::TURN(d))
					}
				}
			},
			_ => None
		}
	}

	pub fn check_exit_input(&mut self) {
		match event::poll_event() {
			event::KeyEvent(key, true , _, _) => {
				match (key) {
					event::EscapeKey => {
						self.exit = true;
					},
					_ => {}
				}
			},
			_ => {}
		}
	}

	pub fn get_input(&mut self) -> map::Action {
		loop {
			match event::wait_event() {
				event::KeyEvent(key, true , m, _) => {
					match self.keyevent_to_action(&key, m) {
						Some(a) => {
							return a;
						},
						None => {}
					}
				},
				event::NoEvent => {},
				_ => {}
			}
		}
	}
}
