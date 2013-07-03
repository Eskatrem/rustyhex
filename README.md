Rustyhex is a work toward implementing a simple rogue-like game with hex tiles.

It's written in Rust and it's primary purpose is to learn and practice Rust
language.

#### Status

ATM (2013-07-03) the game looks something like this:

![Rustyhex screenshot](http://i.imgur.com/CHIrUNb.png)

The game uses SDL library and [Rust SDL bindings][rust-sdl].

[rust-sdl]: https://github.com/brson/rust-sdl

Currently creatures are roaming around the map and attack anything right in
front of them.

#### Keyboard control

Move using Arrow Keys or `hjkl` keys (Vi-like).

To wait a "tick" press `.` or `,`.

Use `u` to use item on the ground. Note: it takes quite a bit of time.
Currently only medkits are implemented.

Hold Left Shift to run (for forward) or strafe (for left and right). Hold Left
Control to attack melee in given direction.

Current challenge is: survive and try killing as much enemies as possible. Tip:
use running to get behind them and strike fast.
