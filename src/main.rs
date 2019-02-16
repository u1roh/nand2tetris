extern crate glutin;
mod given;
mod gate;
mod adder;
mod alu;
mod ram;
mod inst;
mod cpu;
mod blackbox;
mod machine;
mod asm;
mod window;


fn main() {
    let mut m = machine::Machine::new(&[0, 0, 0]);

    let mut events_loop = glutin::EventsLoop::new();
    let window = window::Window::new(&events_loop);

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => window.resize(logical_size),
                    _ => ()
                },
                _ => ()
            }
        });
        window.draw(m.screen().raw_image());
        m.clock(false);
    }
}