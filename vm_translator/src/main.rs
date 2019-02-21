use std::env;
use std::io::Read;

enum Segment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp
}

enum Command<'a> {
    // arithmetic and logical commands
    Add, Sub, Neg, Eq, Gt, Lt, And, Or, Not,

    // memory access commands
    Push(Segment, i16),
    Pop(Segment, i16),

    // program flow commands
    Label(&'a str),
    Goto(&'a str),
    IfGoto(&'a str),

    // function calling commands
    Function(&'a str, i16),
    Call(&'a str, i16),
    Return
}

static VM_SETUP_ASM: &str = "
@256
D=A
@SP
M=D
";

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("usage: {} filename.vm", args[0]);
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

    println!("{}", VM_SETUP_ASM);
}
