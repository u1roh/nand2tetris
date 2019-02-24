extern crate glutin;
extern crate machine;
extern crate asm;
extern crate vm_translator;
use machine::*;
use std::env;
use std::io::{Read, Write};
mod window;

fn read_source(filepath: &str) -> String {
    println!("input file is '{}'", filepath);
    let mut f = std::fs::File::open(filepath).expect("cannot open the input file.");
    let mut source = String::new();
    f.read_to_string(&mut source).expect("failed to read file into a string.");
    println!("{}", source);
    source
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("usage: {} filename.asm", args[0]);
        return;
    }

    let path = std::path::Path::new(&args[1]);
    let instructions = match path.extension().and_then(|s| s.to_str()) {
        Some("hack") => {
            let mut f = std::fs::File::open(&args[1]).expect("cannot open the input file.");
            let mut buf = Vec::<u8>::new();
            let _ = f.read_to_end(&mut buf).expect("failed to read file");
            buf.chunks(2).map(|a| (a[1] as i16) << 8 | a[0] as i16).collect()
        },
        Some("asm") => {
            // read assembly source codes from input file specified with args[1]
            let asm_source = read_source(&args[1]);
            asm::asm(&asm_source).expect("failed to compile asm to binary instructions.")
        },
        Some("vm") => {
            let vm_source = read_source(&args[1]);
            let mut asm_source = String::new();
            vm_translator::compile(&mut asm_source, &vm_source);
            asm::asm(&asm_source).expect("failed to compile asm to binary instructions.")
        },
        _ => panic!("unknown extension")
    };

    // dump instructions
    println!("*** decoded instructions ***");
    for &i in &instructions {
        println!("{}", inst::Instruction::decode(i));
    }

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
            //machine.print_status();
            window.draw(machine.screen_image());
        }
        machine.print_status();
        std::io::stdout().flush();

        counter = counter + 1;
    }
}