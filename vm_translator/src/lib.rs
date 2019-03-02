
#[derive(Debug)]
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

static PUSH_ASM: &str = "
// <STACK PUSH>
// **SP = D
@SP
A=M
M=D
// *SP = *SP + 1
@SP
M=M+1
// </STACK PUSH>
";

static POP_ASM: &str = "
// <STACK POP>
// *SP = *SP - 1
@SP
M=M-1
// D = **SP
@SP
A=M
D=M
// </STACK POP>
";

struct VmWriter<'a> { out: &'a mut std::fmt::Write }

impl<'a> VmWriter<'a> {
    fn write(&mut self, s: &str) {
        writeln!(self.out, "{}", s).unwrap();
    }
    fn write_lines(&mut self, lines: &[&str]) {
        for line in lines { self.write(line); }
    }
    fn push(&mut self) {
        self.write(PUSH_ASM)
    }
    fn pop(&mut self) {
        self.write(POP_ASM)
    }
    // RAM[symbol] = D
    fn store(&mut self, symbol: &str) {
        self.write_lines(&[
            &format!("@{}", symbol),
            "M=D"
        ])
    }
    // D = RAM[symbol]
    fn load(&mut self, symbol: &str) {
        self.write_lines(&[
            &format!("@{}", symbol),
            "D=M"
        ])
    }
    fn set_segment_index_address_to(&mut self, segment: &str, index: i16, dst: char) {
        // write 'index' to D-register
        self.write_lines(&[&format!("@{}", index), "D=A"]);
        match segment {
            "constant" => panic!("constant is pseudo segment"),
            "static"   => panic!("not implemented"),
            "argument" | "local" | "this" | "that" => {
                let symbol = match segment { "argument" => "ARG", "local" => "LCL", "this" => "THIS", "that" => "THAT", _ => panic!("invalid segment") };
                self.write_lines(&[
                    &format!("@{}", symbol),
                    &format!("{}=D+M", dst)
                ]);
            },
            "pointer" | "temp" => {
                let symbol = match segment { "pointer" => "R3", "temp" => "R5", _ => panic!("invalid segment") };
                self.write_lines(&[
                    &format!("@{}", symbol),
                    &format!("{}=D+A", dst)
                ]);
            },
            _ => panic!("unknown segment")
        }
    }

}

fn push(out: &mut std::fmt::Write) -> std::fmt::Result {
    out.write_str(PUSH_ASM)
    //writeln!(out, PUSH_ASM)
}

fn pop(out: &mut std::fmt::Write) -> std::fmt::Result {
    out.write_str(POP_ASM)
}

// RAM[symbol] = D
fn store(out: &mut std::fmt::Write, symbol: &str) -> std::fmt::Result {
    writeln!(out, "@{}", symbol)?;
    writeln!(out, "M=D")?;
    Ok(())
}

// D = RAM[symbol]
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

pub fn compile(out: &mut std::fmt::Write, source: &str) {
    let commands = source.split("\n")
        .map(|line| if let Some(i) = line.find("//") { &line[..i] } else { line })  // remove comment
        .map(|line| line.trim())  // remove white spaces of head and tail
        .filter(|line| !line.is_empty())    // filter empty line
        .map(Command::from_line)
        .collect::<Vec<_>>();

    let mut out = VmWriter{ out: out };
    out.write(VM_SETUP_ASM);
    for command in &commands {
        out.write(&format!("// <{:?}>", command));
        match command {
            Command::BinaryOp(name) => {
                let mnemonic = match *name {
                    "add" => "D+M",
                    "sub" => "D-M",
                    "and" => "D&M",
                    _ => panic!("unknown command")
                };
                out.pop();
                out.store("R13");
                out.pop();
                out.write("@R13");
                out.write(&format!("D={}", mnemonic));
                out.push();
            },
            Command::Push{ segment, index } => {
                // D = segment[index]
                match *segment {
                    "constant" => {
                        out.write(&format!("@{}", index));
                        out.write("D=A");
                    },
                    _ => {
                        out.set_segment_index_address_to(segment, *index, 'A');
                        out.write("D=M");
                    }
                }
                out.push();
            },
            Command::Pop{ segment, index } => {
                // *R13 = segment + index
                out.set_segment_index_address_to(segment, *index, 'D');
                out.write("@R13");
                out.write("M=D");

                out.pop();

                // **R13 = D
                out.write("@R13");
                out.write("A=M");
                out.write("M=D");
            }
            _ => panic!("not implemented")

        }
        out.write(&format!("// </{:?}>", command));
    }
    out.write(VM_TERMINAL_ASM);
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

    #[test]
    fn and() {
        let source = "
        push constant 826
        push constant 294
        and
        ";
        assert_eq!(run_machine(source, 100), 826 & 294);
    }
}