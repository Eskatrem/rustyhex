extern mod sdl;
mod map;

use map;
use map::*;

use core::str;
use core::io;
use core::uint::range;
use core::libc::{c_char};
use sdl::sdl;
use sdl::ll;
use sdl::video;
use sdl::img;
use sdl::keyboard::{SDLKEscape};
use sdl::event;
use sdl::video::{DoubleBuf, HWSurface, AsyncBlit};

use sdl::util::Rect;

const SCREEN_WIDTH: uint = 800;
const SCREEN_HEIGHT: uint = 600;
const SCREEN_BPP: uint = 32;



fn load_or_die(file : ~str) -> ~video::Surface {
	match img::load_img(str::concat(&[~"data/", copy file, ~".png"])) {
		result::Ok(image) => {
			image
		},
		result::Err(str) => {
			die!(str);
		}
	}
}

fn draw_each(screen : &video::Surface, map : &mut map::Map,
	f : &a/fn(position : map::Position, tile : map::Tile) -> Option<&a/video::Surface>)
{
	do map.each() | position : map::Position, tile : &mut map::Tile | {
		match f(position , *tile) {
			None => {},
			Some(surface) => if !screen.blit_surface_rect(
					surface,
					&Rect {
						x: 0, y: 0,
						w: map::HEX_FULL_WIDTH as u16,
						h: map::HEX_FULL_HEIGHT as u16
					},
					&position.to_rect()
				) { die!(~"Failed blit_surface_rect") }
		};
	}
}

fn main() {
	io::print("Hi!\n");
	sdl::sdl::init(&[sdl::sdl::InitEverything]);

	let screen = match video::set_video_mode(
			SCREEN_WIDTH as int, SCREEN_HEIGHT as int, SCREEN_BPP as int,
			&[],&[DoubleBuf]
			) {
		result::Ok(image) => {
			image
		},
		result::Err(str) => {
			io::print(str);
			return;
		}

	};


	let fog = load_or_die(~"fog");
	let floor = load_or_die(~"floor");
	let wall = load_or_die(~"wall");

	let map = map::Map::new();

	let player = Creature::new(Position {x: 5, y: 5}, N);

	loop {
		screen.fill(0);

		do draw_each(screen, map) | _ : map::Position, tile : map::Tile| {
			if tile.is_wall() {
				Some(&*wall)
			} else {
				Some(&*floor)
			}
		}
		do draw_each(screen, map) |position : map::Position, _ : map::Tile| {
			if player.sees(position) {
				Some(&*fog)
			} else {
				None
			}
		}

		screen.flip();

		match event::poll_event() {
			event::KeyDownEvent(ref key_event) => {
				if key_event.keycode == SDLKEscape {
					return;
				}
			}
			event::NoEvent => {},
			_ => {}
		}
	}
}