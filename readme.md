# Woof3d
## Wander aimlessly around an enclosed maze.

### Features
New randomly generated maze every time you open.
"Stunning" 2.5d Graphics
Some form of collision detection.


### Pre-Setup
Requires [rust](https://www.rust-lang.org) as well as the [SDL2](https://www.libsdl.org) development libraries.

For easy rust-installation check out [rustup](https://www.rustup.rs/)

To get SDL2 dev-libraries on your machine, the [SDL2 crate has great instructions](https://github.com/AngryLawyer/rust-sdl2#sdl20-development-libraries)

### Running
Assuming you've got rust and SDL2 setup, to get the program running follow these easy steps

`git clone git@github.com:ggrochow/woof3d.git`

`cd woof3d`

`cargo run --release`

Thats it, it might take a moment to compile, but after its ready a game-window should open

( You can ignore the complaints about the un-used results in the terminal, I'll get to those eventually (probably not)) 


### Controls
`W` - Move forward
`A` - Turn left
`D` - Turn right
`Z` - Look up
`X` - Look down

