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

impl Segment {
    fn from_str(s: &str) -> Option<Segment> {
        use Segment::*;
        match s {
            "argument" => Some(Argument),
            "local" => Some(Local),
            "static" => Some(Static),
            "constant" => Some(Constant),
            "this" => Some(This),
            "that" => Some(That),
            "pointer" => Some(Pointer),
            "temp" => Some(Temp),
            _ => None
        }
    }
}

impl<'a> Command<'a> {
    fn from_line(line: &str) -> Self {
        assert!(!line.is_empty());
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        assert!(tokens.len() != 0);
        match tokens[0] {
            "add"   => Command::Add,
            "sub"   => Command::Sub,
            "neg"   => Command::Neg,
            "eq"    => Command::Eq,
            "gt"    => Command::Gt,
            "lt"    => Command::Lt,
            "and"   => Command::And,
            "or"    => Command::Or,
            "Not"   => Command::Not,
            "push"  => {
                assert_eq!(tokens.len(), 3);
                let segment = Segment::from_str(tokens[1]).unwrap();
                let index = tokens[2].parse::<i16>().unwrap();
                Command::Push(segment, index)
            },
            "pop"   => {
                assert_eq!(tokens.len(), 3);
                let segment = Segment::from_str(tokens[1]).unwrap();
                let index = tokens[2].parse::<i16>().unwrap();
                Command::Pop(segment, index)
            }
            _ => panic!("unknown command: {}", line)
        }
    }
}


static VM_SETUP_ASM: &str = "
@256
D=A
@SP
M=D

@300
D=A
@LCL
M=D

@400
D=A
@ARG
M=D

@3000
D=A
@THIS
M=D

@3010
D=A
@THAT
M=D
";

static VM_TERMINAL_ASM: &str = "
(INFINITE_LOOP)
@INFINITE_LOOP
0;JMP
";

fn push(out: &mut std::io::Write) {
    // **SP = D
    writeln!(out, "@SP");
    writeln!(out, "A=M");
    writeln!(out, "M=D");

    // *SP = *SP + 1
    writeln!(out, "@SP");
    writeln!(out, "M=M+1");
}

fn pop(out: &mut std::io::Write) {
    // *SP = *SP - 1
    writeln!(out, "@SP");
    writeln!(out, "M=M-1");

    // D = **SP
    writeln!(out, "@SP");
    writeln!(out, "A=M");
    writeln!(out, "D=M");
}

fn store(out: &mut std::io::Write, symbol: &str) {
    writeln!(out, "@{}", symbol);
    writeln!(out, "M=D");
}

fn load(out: &mut std::io::Write, symbol: &str) {
    writeln!(out, "@{}", symbol);
    writeln!(out, "D=M");
}

fn set_segment_index_address_to(out: &mut std::io::Write, segment: &Segment, index: i16, dst: char) {
    writeln!(out, "@{}\nD=A", index); // write 'index' to D-register
    use Segment::*;
    match segment {
        Constant => panic!("constant is pseudo segment"),
        Static   => panic!("not implemented"),
        Argument | Local | This | That => {
            let symbol = match segment { Argument => "ARG", Local => "LCL", This => "THIS", That => "THAT", _ => panic!("invalid segment") };
            writeln!(out, "@{}", symbol);
            writeln!(out, "{}=D+M", dst);
        },
        Pointer | Temp => {
            let symbol = match segment { Pointer => "R3", Temp => "R5", _ => panic!("invalid segment") };
            writeln!(out, "@{}", symbol);
            writeln!(out, "{}=D+A", dst);
        }
    };
}

fn compile(out: &mut std::io::Write, commands: &[Command]) {
    writeln!(out, "{}", VM_SETUP_ASM);
    for command in commands {
        match command {
            Command::Add => {
                pop(out);
                store(out, "R13");
                pop(out);
                writeln!(out, "@R13");
                writeln!(out, "D=D+M");
                push(out);
            },
            Command::Push(segment, index) => {
                // D = segment[index]
                match segment {
                    Segment::Constant => { writeln!(out, "@{}\nD=A", index); },
                    _ => {
                        set_segment_index_address_to(out, segment, *index, 'A');
                        writeln!(out, "D=M");
                    }
                }
                push(out);
            },
            Command::Pop(segment, index) => {
                // *R13 = segment + index
                set_segment_index_address_to(out, segment, *index, 'D');
                writeln!(out, "@R13");
                writeln!(out, "M=D");

                pop(out);

                // *R13 = D
                writeln!(out, "@R13");
                writeln!(out, "A=M");
                writeln!(out, "M=D");
            }
            _ => panic!("not implemented")

        }
    }
    write!(out, "{}", VM_TERMINAL_ASM);
}

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

    let commands = source.split("\n")
        .map(|line| if let Some(i) = line.find("//") { &line[..i] } else { line })  // remove comment
        .map(|line| line.trim())  // remove white spaces of head and tail
        .filter(|line| !line.is_empty())    // filter empty line
        .map(Command::from_line)
        .collect::<Vec<_>>();

    let path = std::path::Path::new(&args[1]);
    let mut file = std::fs::File::create(path.with_extension("asm")).unwrap();

    compile(&mut file, &commands);
}
