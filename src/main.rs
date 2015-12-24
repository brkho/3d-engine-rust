extern crate glutin;

use glutin::Window;

fn main() {
    let window = Window::new().unwrap();

    loop {
        for event in window.poll_events() {
            match event {
                // process events here
                _ => ()
            }
        }

        // draw everything here
        let _ = window.swap_buffers();
        let ten_millis = std::time::Duration::from_millis(10);
        std::thread::sleep(ten_millis);
    }
}