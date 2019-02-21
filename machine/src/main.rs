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

    // read assembly source codes from input file specified with args[1]
    let source = {
        println!("input file is '{}'", args[1]);
        let mut f = std::fs::File::open(&args[1]).expect("cannot open the input file.");
        let mut source = String::new();
        f.read_to_string(&mut source).expect("failed to read file into a string.");
        println!("{}", source);
        source
    };

    // assemble the codes into binary instructions
    let instructions = {
        let instructions = asm::asm(&source).expect("failed to compile asm to binary instructions.");
        println!("*** decoded instructions ***");
        for &i in &instructions {
            println!("{}", inst::Instruction::decode(i));
        }
        instructions
    };

    // construct a machine with the instructions
    let mut machine = machine::Machine::new(&instructions);
    machine.print_status_header();

    // start events loop
    let mut events_loop = glutin::EventsLoop::new();
    let window = window::Window::new(&events_loop);
    let mut counter = 0;
    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => window.resize(logical_size),
                    glutin::WindowEvent::KeyboardInput{ input, .. } => {
                        println!("key event: {:?}, {:?}", input.virtual_keycode, input.state);
                        machine.keyboard_input(0);
                    },
                    _ => ()
                },
                _ => ()
            }
        });

        // send a clock signal to the machine
        machine.clock(false);

        // refresh screen
        if counter % 128 == 0 {
            machine.print_status();
            window.draw(machine.screen().raw_image());
        }
        counter = counter + 1;
    }
}