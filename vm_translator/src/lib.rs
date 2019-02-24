
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

fn push(out: &mut std::fmt::Write) -> std::fmt::Result {
    // **SP = D
    writeln!(out, "@SP")?;
    writeln!(out, "A=M")?;
    writeln!(out, "M=D")?;

    // *SP = *SP + 1
    writeln!(out, "@SP")?;
    writeln!(out, "M=M+1")?;

    Ok(())
}

fn pop(out: &mut std::fmt::Write) -> std::fmt::Result {
    // *SP = *SP - 1
    writeln!(out, "@SP")?;
    writeln!(out, "M=M-1")?;

    // D = **SP
    writeln!(out, "@SP")?;
    writeln!(out, "A=M")?;
    writeln!(out, "D=M")?;

    Ok(())
}

fn store(out: &mut std::fmt::Write, symbol: &str) -> std::fmt::Result {
    writeln!(out, "@{}", symbol)?;
    writeln!(out, "M=D")?;
    Ok(())
}

fn load(out: &mut std::fmt::Write, symbol: &str) -> std::fmt::Result {
    writeln!(out, "@{}", symbol)?;
    writeln!(out, "D=M")?;
    Ok(())
}

fn set_segment_index_address_to(out: &mut std::fmt::Write, segment: &Segment, index: i16, dst: char) -> std::fmt::Result {
    writeln!(out, "@{}\nD=A", index)?; // write 'index' to D-register
    use Segment::*;
    match segment {
        Constant => panic!("constant is pseudo segment"),
        Static   => panic!("not implemented"),
        Argument | Local | This | That => {
            let symbol = match segment { Argument => "ARG", Local => "LCL", This => "THIS", That => "THAT", _ => panic!("invalid segment") };
            writeln!(out, "@{}", symbol)?;
            writeln!(out, "{}=D+M", dst)?;
        },
        Pointer | Temp => {
            let symbol = match segment { Pointer => "R3", Temp => "R5", _ => panic!("invalid segment") };
            writeln!(out, "@{}", symbol)?;
            writeln!(out, "{}=D+A", dst)?;
        }
    };
    Ok(())
}

pub fn compile(out: &mut std::fmt::Write, source: &str) -> std::fmt::Result {
    let commands = source.split("\n")
        .map(|line| if let Some(i) = line.find("//") { &line[..i] } else { line })  // remove comment
        .map(|line| line.trim())  // remove white spaces of head and tail
        .filter(|line| !line.is_empty())    // filter empty line
        .map(Command::from_line)
        .collect::<Vec<_>>();

    writeln!(out, "{}", VM_SETUP_ASM)?;
    for command in &commands {
        match command {
            Command::Add => {
                pop(out)?;
                store(out, "R13")?;
                pop(out)?;
                writeln!(out, "@R13")?;
                writeln!(out, "D=D+M")?;
                push(out)?;
            },
            Command::Push(segment, index) => {
                // D = segment[index]
                match segment {
                    Segment::Constant => { writeln!(out, "@{}\nD=A", index)?; },
                    _ => {
                        set_segment_index_address_to(out, segment, *index, 'A')?;
                        writeln!(out, "D=M")?;
                    }
                }
                push(out)?;
            },
            Command::Pop(segment, index) => {
                // *R13 = segment + index
                set_segment_index_address_to(out, segment, *index, 'D')?;
                writeln!(out, "@R13")?;
                writeln!(out, "M=D")?;

                pop(out)?;

                // **R13 = D
                writeln!(out, "@R13")?;
                writeln!(out, "A=M")?;
                writeln!(out, "M=D")?;
            }
            _ => panic!("not implemented")

        }
    }
    write!(out, "{}", VM_TERMINAL_ASM)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate machine;
    extern crate asm;
    use machine::*;
    use super::*;

    fn run_machine(vm_source: &str, nclock: usize) -> i16 {
        let mut asm_source = String::new();
        compile(&mut asm_source, vm_source);

        let bin = asm::asm(&asm_source).unwrap();
        let mut machine = Machine::new(&bin);
        for _ in 0 .. nclock {
            machine.clock(false);
        }
        machine.read_memory(machine.read_memory(0) - 1) // top of the stack
    }

    #[test]
    fn simple_add() {
        let source = "
        push constant 7
        push constant 8
        add
        ";
        assert_eq!(run_machine(source, 100), 15);
    }

}