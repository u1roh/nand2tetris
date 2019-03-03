
#[derive(Debug)]
enum Command<'a> {
    // arithmetic commands
    BinaryOp(&'a str),
    UnaryOp(&'a str),

    // logical commands
    LogicalOp(&'a str), // eq, gt, lt

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
            "add" | "sub" | "and" | "or" => Command::BinaryOp(tokens[0]),
            "eq" | "gt" | "lt" => Command::LogicalOp(tokens[0]),
            "push"  => {
                assert_eq!(tokens.len(), 3);
                Command::Push{ segment: tokens[1], index: tokens[2].parse::<i16>().unwrap() }
            },
            "pop" => {
                assert_eq!(tokens.len(), 3);
                Command::Pop{ segment: tokens[1], index: tokens[2].parse::<i16>().unwrap() }
            },
            "label" => {
                assert_eq!(tokens.len(), 2);
                Command::Label(tokens[1])
            },
            "if-goto" => {
                assert_eq!(tokens.len(), 2);
                Command::IfGoto(tokens[1])
            },
            "goto" => {
                assert_eq!(tokens.len(), 2);
                Command::Goto(tokens[1])
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

struct VmWriter<'a> { out: &'a mut std::fmt::Write, filename: &'a str }

impl<'a> VmWriter<'a> {
    fn write(&mut self, s: &str) {
        writeln!(self.out, "{}", s).unwrap();
    }
    fn write_lines(&mut self, lines: &[&str]) {
        for line in lines { self.write(line); }
    }
    fn label(&mut self, label: &str) {
        writeln!(self.out, "({})", label).unwrap();
    }
    fn symbol(&mut self, symbol: &str) {
        writeln!(self.out, "@{}", symbol).unwrap();
    }
    fn push(&mut self) {
        self.write(PUSH_ASM)
    }
    fn pop(&mut self) {
        self.write(POP_ASM)
    }
    fn binary_op(&mut self, op: char) {
        self.pop();
        self.store("R13");
        self.pop();
        self.write("@R13");
        self.write(&format!("D=D{}M", op));
    }
    fn logical_op(&mut self, jmp: &str, label_id: usize) {
        let if_true = format!("IF_TRUE_{}", label_id);
        let if_end  = format!("IF_END_{}", label_id);
        self.binary_op('-');
        self.symbol(&if_true);
        self.write(&format!("D;{}", jmp));
        self.write("D=0");
        self.symbol(&if_end);
        self.write("0;JMP");
        self.label(&if_true);
        self.write("D=-1");
        self.label(&if_end);
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
            "static" => {
                self.write(&format!("@{}.{}", self.filename, index));
                if dst != 'A' { self.write(&format!("{}=A", dst)) }
            },
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

pub fn compile(out: &mut std::fmt::Write, source_filename: &str, source: &str) {
    let commands = source.split("\n")
        .map(|line| if let Some(i) = line.find("//") { &line[..i] } else { line })  // remove comment
        .map(|line| line.trim())  // remove white spaces of head and tail
        .filter(|line| !line.is_empty())    // filter empty line
        .map(Command::from_line)
        .collect::<Vec<_>>();

    let mut label_counter = 0;
    let mut out = VmWriter{ out: out, filename: source_filename };
    out.write(VM_SETUP_ASM);
    for command in &commands {
        out.write(&format!("// <{:?}>", command));
        match command {
            Command::UnaryOp(name) => {
                let mnemonic = match *name {
                    "neg" => "-D",
                    "not" => "!D",
                    _ => panic!("unknown unary operation: {}", name)
                };
                out.pop();
                out.write(&format!("D={}", mnemonic));
                out.push();
            },
            Command::BinaryOp(name) => {
                let op = match *name {
                    "add" => '+',
                    "sub" => '-',
                    "and" => '&',
                    "or"  => '|',
                    _ => panic!("unknown binary operation: {}", name)
                };
                out.binary_op(op);
                out.push();
            },
            Command::LogicalOp(name) => {
                let jmp = match *name {
                    "eq" => "JEQ",
                    "gt" => "JGT",
                    "lt" => "JLT",
                    _ => panic!("unknown logical operation: {}", name)
                };
                out.logical_op(jmp, label_counter);
                out.push();
                label_counter += 1;
            },
            Command::Push{ segment, index } => {
                assert!(*index >= 0);
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
                assert!(*index >= 0);
                // *R13 = segment + index
                out.set_segment_index_address_to(segment, *index, 'D');
                out.write("@R13");
                out.write("M=D");

                out.pop();

                // **R13 = D
                out.write("@R13");
                out.write("A=M");
                out.write("M=D");
            },
            Command::Label(symbol) => {
                out.label(symbol);
            },
            Command::Goto(symbol) => {
                out.symbol(symbol);
                out.write("0;JMP");
            },
            Command::IfGoto(symbol) => {
                out.pop();
                out.symbol(symbol);
                out.write("D;JNE");
            },
            _ => panic!("not implemented: {:?}", command)
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
        compile(&mut asm_source, "test_file", vm_source);
        println!("{}", asm_source);

        let bin = asm::asm(&asm_source).unwrap();
        let mut machine = Machine::new(&bin);
        for _ in 0 .. nclock {
            println!("{}", inst::Instruction::decode(machine.next_instruction()));
            machine.clock(false);
            println!("SP = {}, STACK TOP = {}", machine.read_memory(0), machine.read_memory(machine.read_memory(0) - 1));
        }
        machine.read_memory(machine.read_memory(0) - 1) // top of the stack
    }

    fn test(nclock: usize, expected: i16, source: &str) {
        assert_eq!(expected, run_machine(source, nclock));
    }

    #[test]
    fn add() {
        test(100, 15, "
        push constant 7
        push constant 8
        add
        ");
    }

    #[test]
    fn sub() {
        test(100, 327 - 193,"
        push constant 327
        push constant 193
        sub
        ");
    }

    #[test]
    fn and() {
        test(100, 826 & 294, "
        push constant 826
        push constant 294
        and
        ");
    }

    #[test]
    fn or() {
        test(100, 826 | 294, "
        push constant 826
        push constant 294
        or
        ");
    }

    #[test]
    fn eq() {
        test(100, -1, "
        push constant 123
        push constant 123
        eq
        ");
        test(100, 0, "
        push constant 100
        push constant 200
        eq
        ");
    }

    #[test]
    fn gt() {
        test(100, -1, "
        push constant 826
        push constant 294
        gt
        ");
        test(100, 0, "
        push constant 1
        push constant 2
        gt
        ");
        test(100, 0, "
        push constant 3
        push constant 3
        gt
        ");
    }

    #[test]
    fn lt() {
        test(100, 0, "
        push constant 826
        push constant 294
        lt
        ");
        test(100, -1, "
        push constant 1
        push constant 2
        lt
        ");
        test(100, 0, "
        push constant 3
        push constant 3
        lt
        ");
    }

    #[test]
    fn neg() {
        test(100, -123, "
        push constant 123
        neg
        ");
        test(100, 321, "
        push constant 321
        neg
        neg
        ");
    }

    #[test]
    fn not() {
        test(100, !0, "
        push constant 0
        not
        ");
        test(100, !123, "
        push constant 123
        not
        ");
    }

    #[test]
    fn arithmetic() {
        test(100, 1 - (2 + 3), "
        push constant 1
        push constant 2
        push constant 3
        add
        sub
        ");
        test(100, (1 + 2) - 3, "
        push constant 1
        push constant 2
        add
        push constant 3
        sub
        ");
        test(200, (1 + 3) - (5 + 7), "
        push constant 1
        push constant 3
        add
        push constant 5
        push constant 7
        add
        sub
        ");
    }

    #[test]
    fn pointer_segment() {
        test(100, 123, "
        push constant 123
        pop pointer 0
        push pointer 0
        ");
        test(100, 456, "
        push constant 456
        pop pointer 1
        push pointer 1
        ");
    }

    #[test]
    fn this_segment() {
        test(100, 789, "
        push constant 789
        pop this 0
        push this 0
        ");
        test(100, 111, "
        push constant 111
        pop this 1
        push this 1
        ");
        test(100, 3001, "
        push constant 3000
        push constant 1
        add
        pop pointer 0
        push pointer 0
        ");
        test(200, 789, "
        push constant 789
        pop this 1
        push pointer 0
        push constant 1
        add
        pop pointer 0
        push pointer 0
        push this 0
        ");
    }

    #[test]
    fn static_segment() {
        test(200, 123, "
        push constant 123
        pop static 3
        push static 3
        ");
    }

    #[test]
    fn conditional() {
        let vm = "
        if-goto L1
        push constant 123
        goto L2
        label L1
        push constant 456
        label L2
        push constant 1
        add
        ";
        test(200, 124, &format!("
        push constant 0
        {}
        ", vm));
        test(200, 457, &format!("
        push constant 0
        not
        {}
        ", vm));
    }
}