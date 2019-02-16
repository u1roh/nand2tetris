extern crate glutin;
use std::env;
use std::io::Read;
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
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("usage: {} filename.asm", args[0]);
        return;
    }

    let source = {
        println!("input file is '{}'", args[1]);
        let mut f = std::fs::File::open(&args[1]).expect("cannot open the input file.");
        let mut source = String::new();
        f.read_to_string(&mut source).expect("failed to read file into a string.");
        println!("{}", source);
        source
    };

    let instructions = {
        let instructions = asm::asm(&source).expect("failed to compile asm to binary instructions.");
        println!("*** decoded instructions ***");
        for &i in &instructions {
            println!("{}", inst::Instruction::decode(i));
        }
        instructions
    };

    let mut machine = machine::Machine::new(&instructions);

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
        window.draw(machine.screen().raw_image());
        machine.clock(false);
    }
}