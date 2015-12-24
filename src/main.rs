// Work in progress of a 2D game using glutin for context creation/input
// handling and gl-rs for OpenGL bindings. The game will be a simple top down
// action-RPG created for educational purposes to assess the viability of Rust
// as a video game development language.
//
// Brian Ho
// brian@brkho.com
// December 2015


extern crate glutin;

use glutin::{Event, Window};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

// Main loop for the game.
fn main() {
    // Create the window. Should be using a builder here, but whatever.
    let window = Window::new().unwrap();
    unsafe { window.make_current().unwrap() };

    loop {
        // poll_events returns an iterator for Event which we match against.
        for event in window.poll_events() {
            match event {
                Event::Closed => exit(0),
                _ => ()
            }
        }

        // We can update and draw here after we handle events and swap buffers.
        window.swap_buffers().unwrap();

        // Sleep one second in between calls.
        let one_second = Duration::from_millis(1000);
        sleep(one_second);
    }
}