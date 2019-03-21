
#[derive(Debug)]
pub enum UnaryOp { Neg, Not }

#[derive(Debug)]
pub enum BinaryOp { Add, Sub, And, Or }

#[derive(Debug)]
pub enum Condition { Eq, Gt, Lt }

/*
const VM_TERMINAL_ASM: &str = "
(INFINITE_LOOP)
@INFINITE_LOOP
0;JMP
";
*/

const PUSH_ASM: &str = "
@SP\nA=M\nM=D   // **SP = D
@SP\nM=M+1      // *SP = *SP + 1
";

const POP_ASM: &str = "
@SP\nM=M-1      // *SP = *SP - 1
@SP\nA=M\nD=M   // D = **SP
";

const RETURN_ASM: &str = "
@ARG\nA=M\nM=D          // *RAM[ARG] = D
@ARG\nD=M+1\n@SP\nM=D   // RAM[SP] = RAM[ARG] + 1
@LCL\nD=M\n@R13\nM=D    // RAM[R13] = RAM[LCL]
@R13\nM=M-1\nA=M\nD=M\n@THAT\nM=D   // --RAM[R13]; *THAT = *RAM[R13];
@R13\nM=M-1\nA=M\nD=M\n@THIS\nM=D   // --RAM[R13]; *THIS = *RAM[R13];
@R13\nM=M-1\nA=M\nD=M\n@ARG\nM=D    // --RAM[R13]; *ARG  = *RAM[R13];
@R13\nM=M-1\nA=M\nD=M\n@LCL\nM=D    // --RAM[R13]; *LCL  = *RAM[R13];
@R14\nA=M\n0;JMP    // goto RAM[R14]
";

pub struct AsmWriter<'a> {
    out: &'a mut std::fmt::Write,
    filename: &'a str,
    label_id: usize
}

impl<'a> Drop for AsmWriter<'a> {
    fn drop(&mut self) {
        writeln!(self.out, "(TERMINAL)").unwrap();
    }
}

impl<'a> AsmWriter<'a> {
    pub fn new(out: &'a mut std::fmt::Write, filename: &'a str) -> Self {
        Self{ out, filename, label_id: 0 }
    }
    pub fn label(&mut self, label: &str) {
        writeln!(self.out, "({})", label).unwrap();
    }
    pub fn set_ram(&mut self, symbol: &str, value: i16) {
        writeln!(self.out, "@{}\nD=A\n@{}\nM=D", value, symbol).unwrap();
    }
    fn new_unique_label(&mut self, label: &str) -> String {
        self.label_id += 1;
        format!("{}_{}", label, self.label_id)
    }
    pub fn call_sys_init(&mut self) {
        self.set_ram("SP", 256);
        self.func_call("Sys.init", 0);
        self.goto("TERMINAL");
    }
    pub fn push(&mut self) {
        self.out.write_str(PUSH_ASM).unwrap();
    }
    pub fn pop(&mut self) {
        self.out.write_str(POP_ASM).unwrap();
    }
    pub fn goto(&mut self, label: &str) {
        writeln!(self.out, "@{}\n0;JMP", label).unwrap();
    }
    pub fn if_goto(&mut self, label: &str) {
        writeln!(self.out, "@{}\nD;JNE", label).unwrap();
    }
    pub fn unary_op(&mut self, op: UnaryOp) {
        self.pop();
        let op = match op {
            UnaryOp::Neg => '-',
            UnaryOp::Not => '!'
        };
        writeln!(self.out, "D={}D", op).unwrap();
    }
    pub fn binary_op(&mut self, op: BinaryOp) {
        self.pop();
        writeln!(self.out, "@R13\nM=D").unwrap();
        self.pop();
        let op = match op {
            BinaryOp::Add => '+',
            BinaryOp::Sub => '-',
            BinaryOp::Or  => '|',
            BinaryOp::And => '&',
        };
        writeln!(self.out, "@R13\nD=D{}M", op).unwrap();
    }
    pub fn logical_op(&mut self, cond: Condition) {
        let if_true = self.new_unique_label("IF_TRUE");
        let if_end  = self.new_unique_label("IF_END");
        self.binary_op(BinaryOp::Sub);
        let jmp = match cond {
            Condition::Eq => "JEQ",
            Condition::Gt => "JGT",
            Condition::Lt => "JLT",
        };
        writeln!(self.out, "@{}\nD;{}", if_true, jmp).unwrap();
        self.out.write_str("D=0\n").unwrap();
        self.goto(&if_end);
        self.label(&if_true);
        self.out.write_str("D=-1\n").unwrap();
        self.label(&if_end);
    }

    fn set_segment_index_address_to(&mut self, segment: &str, index: i16, dst: char) {
        // write 'index' to D-register
        writeln!(self.out, "@{}\nD=A", index).unwrap();
        match segment {
            "constant" => panic!("constant is pseudo segment"),
            "static" => {
                writeln!(self.out, "@{}.{}\n{}=A", self.filename, index, dst).unwrap();
            },
            "argument" | "local" | "this" | "that" => {
                let symbol = match segment { "argument" => "ARG", "local" => "LCL", "this" => "THIS", "that" => "THAT", _ => panic!("invalid segment") };
                writeln!(self.out, "@{}\n{}=D+M", symbol, dst).unwrap();
            },
            "pointer" | "temp" => {
                let symbol = match segment { "pointer" => "R3", "temp" => "R5", _ => panic!("invalid segment") };
                writeln!(self.out, "@{}\n{}=D+A", symbol, dst).unwrap();
            },
            _ => panic!("unknown segment")
        }
    }

    pub fn push_segment(&mut self, segment: &str, index: i16) {
        assert!(index >= 0);
        // D = segment[index]
        match segment {
            "constant" => writeln!(self.out, "@{}\nD=A", index).unwrap(),
            _ => { self.set_segment_index_address_to(segment, index, 'A'); self.out.write_str("D=M\n").unwrap(); }
        }
        self.push();
    }

    pub fn pop_segment(&mut self, segment: &str, index: i16) {
        assert!(index >= 0);
        // *R13 = segment + index
        self.set_segment_index_address_to(segment, index, 'D');
        self.out.write_str("@R13\nM=D\n").unwrap();
        self.pop();
        writeln!(self.out, "@R13\nA=M\nM=D").unwrap(); // **R13 = D
    }

    pub fn func_begin(&mut self, funcname: &str, nlocals: i16) {
        self.label(funcname);
        self.out.write_str("D=0\n").unwrap();
        for _ in 0 .. nlocals { self.push() }
    }
    pub fn func_call(&mut self, funcname: &str, nargs: i16) {
        let return_label = self.new_unique_label("RETURN");
        writeln!(self.out, "@{}\nD=A", return_label).unwrap();
        self.push();
        for symbol in &["LCL", "ARG", "THIS", "THAT"] {
            writeln!(self.out, "@{}\nA=M\nD=M", symbol).unwrap();
            self.push();
        }
        writeln!(self.out, "@{}\nD=-A\n@SP\nD=D+M\n@ARG\nM=D", nargs + 5).unwrap();  // *ARG = *SP - nargs - 5
        writeln!(self.out, "@SP\nD=M\n@LCL\nM=D").unwrap();  // *LCL = *SP
        writeln!(self.out, "@{}\n0;JMP", funcname).unwrap();
        self.label(&return_label);
    }
    pub fn func_return(&mut self) {
        self.out.write_str("@LCL\nD=M\n@5\nD=D-A\n@R14\nM=D\n").unwrap(); // RAM[R14] = RAM[LCL] - 5 (put the return-address in RAM[R14])
        self.pop();
        self.out.write_str(RETURN_ASM).unwrap();
    }
}
