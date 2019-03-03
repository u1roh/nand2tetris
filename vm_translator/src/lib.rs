
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
    Function{ funcname: &'a str, nlocals: i16 },
    Call{ funcname: &'a str, nargs: i16 },
    Return
}

fn line_to_command(line: &str) -> Command {
    assert!(!line.is_empty());
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    assert!(tokens.len() != 0);
    match tokens[0] {
        "neg" | "not" => Command::UnaryOp(tokens[0]),
        "add" | "sub" | "and" | "or" => Command::BinaryOp(tokens[0]),
        "eq" | "gt" | "lt" => Command::LogicalOp(tokens[0]),
        "push"      => Command::Push{ segment: tokens[1], index: tokens[2].parse::<i16>().unwrap() },
        "pop"       => Command::Pop{ segment: tokens[1], index: tokens[2].parse::<i16>().unwrap() },
        "label"     => Command::Label(tokens[1]),
        "if-goto"   => Command::IfGoto(tokens[1]),
        "goto"      => Command::Goto(tokens[1]),
        "function"  => Command::Function{ funcname: tokens[1], nlocals: tokens[2].parse::<i16>().unwrap() },
        "call"      => Command::Call{ funcname: tokens[1], nargs: tokens[2].parse::<i16>().unwrap() },
        "return"    => Command::Return,
        _ => panic!("unknown command: {}", line)
    }
}

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

static RETURN_ASM: &str = "
@ARG\nA=M\nM=D          // *RAM[ARG] = D
@ARG\nD=M+1\n@SP\nM=D   // RAM[SP] = RAM[ARG] + 1
@LCL\nD=M\n@R13\nM=D    // RAM[R13] = RAM[LCL]
@R13\nM=M-1\nA=M\nD=M\n@THAT\nM=D   // --RAM[R13]; *THAT = *RAM[R13];
@R13\nM=M-1\nA=M\nD=M\n@THIS\nM=D   // --RAM[R13]; *THIS = *RAM[R13];
@R13\nM=M-1\nA=M\nD=M\n@ARG\nM=D    // --RAM[R13]; *ARG  = *RAM[R13];
@R13\nM=M-1\nA=M\nD=M\n@LCL\nM=D    // --RAM[R13]; *LCL  = *RAM[R13];
@R13\nM=M-1\nA=M\nA=M\n0;JMP        // --RAM[R13]; goto *RAM[R13];
";

struct AsmWriter<'a> {
    out: &'a mut std::fmt::Write,
    filename: &'a str,
    label_id: usize
}

impl<'a> AsmWriter<'a> {
    fn new(out: &'a mut std::fmt::Write, filename: &'a str) -> Self {
        Self{ out: out, filename: filename, label_id: 0 }
    }
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
    fn set_ram(&mut self, symbol: &str, value: i16) {
        writeln!(self.out, "@{}\nD=A\n@{}\nM=D", value, symbol).unwrap();
    }
    // RAM[symbol] = D
    fn store(&mut self, symbol: &str) {
        writeln!(self.out, "@{}\nM=D", symbol).unwrap();
    }
    fn push(&mut self) {
        self.write(PUSH_ASM)
    }
    fn pop(&mut self) {
        self.write(POP_ASM)
    }
    fn jump(&mut self, label: &str) {
        writeln!(self.out, "@{}\n0;JMP", label).unwrap();
    }
    fn jump_if(&mut self, label: &str, jmp: &str) {
        writeln!(self.out, "@{}\nD;{}", label, jmp).unwrap();
    }
    fn binary_op(&mut self, op: char) {
        self.pop();
        self.store("R13");
        self.pop();
        self.write("@R13");
        self.write(&format!("D=D{}M", op));
    }
    fn logical_op(&mut self, jmp: &str) {
        let if_true = format!("IF_TRUE_{}", self.label_id);
        let if_end  = format!("IF_END_{}", self.label_id);
        self.binary_op('-');
        self.jump_if(&if_true, jmp);
        self.write("D=0");
        self.jump(&if_end);
        self.label(&if_true);
        self.write("D=-1");
        self.label(&if_end);
        self.label_id += 1;
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

fn translate_command(out: &mut AsmWriter, command: &Command) {
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
            out.logical_op(jmp);
            out.push();
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
            out.store("R13");

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
        Command::Function{ funcname, nlocals } => {
            out.label(funcname);
            out.write("D=0");
            for _ in 0 .. *nlocals { out.push() }
        },
        Command::Return => {
            out.pop();
            out.write(RETURN_ASM);
        },
        _ => panic!("not implemented: {:?}", command)
    }
    out.write(&format!("// </{:?}>", command));
}

fn translate_vm_source(out: &mut AsmWriter, source: &str) {
    let commands = source.split("\n")
        .map(|line| if let Some(i) = line.find("//") { &line[..i] } else { line })  // remove comment
        .map(|line| line.trim())  // remove white spaces of head and tail
        .filter(|line| !line.is_empty())    // filter empty line
        .map(line_to_command)
        .collect::<Vec<_>>();
    for command in &commands {
        translate_command(out, command);
    }
}

pub fn compile(out: &mut std::fmt::Write, source_filename: &str, source: &str) {
    let mut out = AsmWriter::new(out, source_filename);
    out.set_ram("SP", 256);
    out.set_ram("LCL", 300);
    out.set_ram("ARG", 400);
    out.set_ram("THIS", 3000);
    out.set_ram("THAT", 3010);
    translate_vm_source(&mut out, source);
    //out.write(VM_TERMINAL_ASM);
}

#[cfg(test)]
mod tests {
    extern crate machine;
    extern crate asm;
    use machine::*;
    use super::*;

    fn run_machine(vm_source: &str, max_clock: usize) -> i16 {
        let mut asm_source = String::new();
        compile(&mut asm_source, "test_file", vm_source);
        println!("{}", asm_source);

        let bin = asm::asm(&asm_source).unwrap();
        let mut machine = Machine::new(&bin);
        let mut nclock = 0;
        while !machine.is_terminated() {
            println!("{}", inst::Instruction::decode(machine.next_instruction()));
            machine.clock(false);
            println!("SP = {}, STACK TOP = {}", machine.read_memory(0), machine.read_memory(machine.read_memory(0) - 1));
            nclock += 1;
            assert!(nclock < max_clock);
        }
        machine.read_memory(machine.read_memory(0) - 1) // top of the stack
    }

    fn test(expected: i16, source: &str) {
        let max_clock = 1000;
        assert_eq!(expected, run_machine(source, max_clock));
    }

    #[test]
    fn add() {
        test(15, "
        push constant 7
        push constant 8
        add
        ");
    }

    #[test]
    fn sub() {
        test(327 - 193,"
        push constant 327
        push constant 193
        sub
        ");
    }

    #[test]
    fn and() {
        test(826 & 294, "
        push constant 826
        push constant 294
        and
        ");
    }

    #[test]
    fn or() {
        test(826 | 294, "
        push constant 826
        push constant 294
        or
        ");
    }

    #[test]
    fn eq() {
        test(-1, "
        push constant 123
        push constant 123
        eq
        ");
        test(0, "
        push constant 100
        push constant 200
        eq
        ");
    }

    #[test]
    fn gt() {
        test(-1, "
        push constant 826
        push constant 294
        gt
        ");
        test(0, "
        push constant 1
        push constant 2
        gt
        ");
        test(0, "
        push constant 3
        push constant 3
        gt
        ");
    }

    #[test]
    fn lt() {
        test(0, "
        push constant 826
        push constant 294
        lt
        ");
        test(-1, "
        push constant 1
        push constant 2
        lt
        ");
        test(0, "
        push constant 3
        push constant 3
        lt
        ");
    }

    #[test]
    fn neg() {
        test(-123, "
        push constant 123
        neg
        ");
        test(321, "
        push constant 321
        neg
        neg
        ");
    }

    #[test]
    fn not() {
        test(!0, "
        push constant 0
        not
        ");
        test(!123, "
        push constant 123
        not
        ");
    }

    #[test]
    fn arithmetic() {
        test(1 - (2 + 3), "
        push constant 1
        push constant 2
        push constant 3
        add
        sub
        ");
        test((1 + 2) - 3, "
        push constant 1
        push constant 2
        add
        push constant 3
        sub
        ");
        test((1 + 3) - (5 + 7), "
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
        test(123, "
        push constant 123
        pop pointer 0
        push pointer 0
        ");
        test(456, "
        push constant 456
        pop pointer 1
        push pointer 1
        ");
    }

    #[test]
    fn this_segment() {
        test(789, "
        push constant 789
        pop this 0
        push this 0
        ");
        test(111, "
        push constant 111
        pop this 1
        push this 1
        ");
        test(789, "
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
        test(123, "
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
        test(124, &format!("
        push constant 0
        {}
        ", vm));
        test(457, &format!("
        push constant 0
        not
        {}
        ", vm));
    }

    #[test]
    fn sum() {
        test(1 + 2 + 3 + 4 + 5, "
        // argument[0] = 5
        push constant 5
        pop argument 0

        // Computes the sum 1 + 2 + ... + argument[0] and pushes the 
        // result onto the stack.
        push constant 0    
        pop local 0         // initializes sum = 0
        label LOOP_START
        push argument 0    
        push local 0
        add
        pop local 0	        // sum = sum + counter
        push argument 0
        push constant 1
        sub
        pop argument 0      // counter--
        push argument 0
        if-goto LOOP_START  // If counter > 0, goto LOOP_START
        push local 0
        ");
    }

    #[test]
    fn simple_function() {
        test(3, "
        function test_add 0
            push argument 0
            push argument 1
            add
            return
        push constant 1
        push constant 2
        call test_add 2
        ");
    }
}