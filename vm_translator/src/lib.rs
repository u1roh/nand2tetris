
enum Command<'a> {
    // arithmetic and logical commands
    BinaryOp(&'a str),
    UnaryOp(&'a str),

    // memory access commands
    Push{ segment: &'a str, index: i16 },
    Pop { segment: &'a str, index: i16 },

    // program flow commands
    Label(&'a str),
    Goto(&'a str),
    IfGoto(&'a str),

    // function calling commands
    Function(&'a str, i16),
    Call(&'a str, i16),
    Return
}

impl<'a> Command<'a> {
    fn from_line(line: &'a str) -> Self {
        assert!(!line.is_empty());
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        assert!(tokens.len() != 0);
        match tokens[0] {
            "neg" | "not" => Command::UnaryOp(tokens[0]),
            "add" | "sub" | "eq" | "gt" | "lt" | "and" | "or" => Command::BinaryOp(tokens[0]),
            "push"  => {
                assert_eq!(tokens.len(), 3);
                Command::Push{ segment: tokens[1], index: tokens[2].parse::<i16>().unwrap() }
            },
            "pop"   => {
                assert_eq!(tokens.len(), 3);
                Command::Pop{ segment: tokens[1], index: tokens[2].parse::<i16>().unwrap() }
            },
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

fn set_segment_index_address_to(out: &mut std::fmt::Write, segment: &str, index: i16, dst: char) -> std::fmt::Result {
    writeln!(out, "@{}\nD=A", index)?; // write 'index' to D-register
    match segment {
        "constant" => panic!("constant is pseudo segment"),
        "static"   => panic!("not implemented"),
        "argument" | "local" | "this" | "that" => {
            let symbol = match segment { "argument" => "ARG", "local" => "LCL", "this" => "THIS", "that" => "THAT", _ => panic!("invalid segment") };
            writeln!(out, "@{}", symbol)?;
            writeln!(out, "{}=D+M", dst)?;
        },
        "pointer" | "temp" => {
            let symbol = match segment { "pointer" => "R3", "temp" => "R5", _ => panic!("invalid segment") };
            writeln!(out, "@{}", symbol)?;
            writeln!(out, "{}=D+A", dst)?;
        },
        _ => panic!("unknown segment")
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
            Command::BinaryOp(name) => {
                pop(out)?;
                store(out, "R13")?;
                pop(out)?;
                writeln!(out, "@R13")?;
                match *name {
                    "add" => writeln!(out, "D=D+M")?,
                    "sub" => writeln!(out, "D=D-M")?,
                    _ => panic!("unknown command")
                }
                push(out)?;
            },
            Command::Push{ segment, index } => {
                // D = segment[index]
                match *segment {
                    "constant" => { writeln!(out, "@{}\nD=A", index)?; },
                    _ => {
                        set_segment_index_address_to(out, segment, *index, 'A')?;
                        writeln!(out, "D=M")?;
                    }
                }
                push(out)?;
            },
            Command::Pop{ segment, index } => {
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
    fn add() {
        let source = "
        push constant 7
        push constant 8
        add
        ";
        assert_eq!(run_machine(source, 100), 15);
    }

    #[test]
    fn sub() {
        let source = "
        push constant 327
        push constant 193
        sub
        ";
        assert_eq!(run_machine(source, 100), 327 - 193);
    }
}