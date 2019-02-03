use crate::given::*;
use crate::gate::*;
use crate::ram::*;
use crate::alu::*;

pub struct Cpu {
    A: Register,    // A register ('A' stands for address)
    D: Register,    // D register ('D' stands for data)
    PC: Counter,    // Program counter
}

#[derive(Clone, Copy)]
pub struct CpuInput {
    pub inM: Word,
    pub instruction: Word,
    pub reset: bool
}

pub struct CpuOutput {
    pub outM: Word,
    pub writeM: bool,
    pub addressM: Word,
    pub pc: Word
}

struct ControlBits {
    in_a: Word,     // input to A-register
    in_d: Word,     // input to D-register
    in_pc: Word,    // input to program counter
    load_a: bool,   // load bit for A-register
    load_d: bool,   // load bit for D-register
    jump: bool      // load bit for program counter
}

impl Cpu {
    pub fn new() -> Self {
        Self { A: Register::new(), D: Register::new(), PC: Counter::new() }
    }
    fn alu(&self, inM: Word, instruction: Word) -> AluOutput {
        let x = self.D.out();   // x = D
        let y = mux16(self.A.out(), inM, instruction[12]);  // y = A or M
        alu(x, y,
            instruction[11],    // zx
            instruction[10],    // nx
            instruction[ 9],    // zy
            instruction[ 8],    // ny
            instruction[ 7],    // f
            instruction[ 6])    // no
    }
    fn decode(&self, inM: Word, instruction: Word) -> ControlBits {
        let is_c_instruction = instruction[15];
        let alu_out = self.alu(inM, instruction);
        ControlBits {
            in_a: mux16(instruction, alu_out.out, is_c_instruction),
            in_d: alu_out.out,
            in_pc: self.A.out(),
            load_a: or(instruction[5], not(is_c_instruction)),
            load_d: instruction[4],
            jump: or(or(
                and(instruction[0], not(or(alu_out.zr, alu_out.ng))),
                and(instruction[1], alu_out.zr)),
                and(instruction[2], alu_out.ng))
        }
    }
    pub fn out(&self, input: CpuInput) -> CpuOutput {
        CpuOutput{
            outM: self.alu(input.inM, input.instruction).out,
            writeM: input.instruction[3],
            addressM: self.A.out(),
            pc: self.PC.out()
        }
    }
    pub fn clock(&mut self, input: CpuInput) {
        let c = self.decode(input.inM, input.instruction);
        self.A.clock(c.in_a, c.load_a);
        self.D.clock(c.in_d, c.load_d);
        self.PC.clock(c.in_pc, not(c.jump), c.jump, input.reset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum Computation {
        Zero,               // 0
        One,                // 1
        MinusOne,           // -1
        D,                  // D
        X(bool),            // A or M
        NotD,               // !D
        NotX(bool),         // !A or !M
        MinusD,             // -D
        MinusX(bool),       // -A or -M
        DPlusOne,           // D + 1
        XPlusOne(bool),     // A + 1 or M + 1
        DMinusOne,          // D - 1
        XMinusOne(bool),    // A - 1 or M - 1
        DPlusX(bool),       // D + A or D + M
        DMinusX(bool),      // D - A or D - M
        XMinusD(bool),      // A - D or M - D
        DAndX(bool),        // D & A or D & M
        DOrX(bool),         // D | A or D | M
    }

    static dest_A: u8 = 0b001;
    static dest_D: u8 = 0b010;
    static dest_M: u8 = 0b100;

    #[derive(Clone, Copy)]
    enum Jump {
        Null,   // No jump
        JGT,    // if out >  0 jump
        JEQ,    // if out == 0 jump
        JGE,    // if out >= 0 jump
        JLT,    // if out <  0 jump
        JNE,    // if out != 0 jump
        JLE,    // if out <= 0 jump
        JMP     // jump
    }

    enum Instruction {
        AInstruction(i16),
        CInstruction(Computation, u8, Jump)
    }
    use Instruction::*;

    impl Instruction {
        fn to_bits(&self) -> [bool; 16] {
            use Instruction::*;
            match self {
                AInstruction(address) => {
                    assert!(*address >= 0); // A-instruction's MSB = 0
                    int2word(*address)
                },
                CInstruction(comp, dest, jump) => {
                    use Computation::*;
                    let mut bits = [false; 16];
                    bits[15] = true;    // C-instruction's MSB = 1
                    let (comp, a) =
                        match comp {
                            Zero        => ([1, 0, 1, 0, 1, 0], false),
                            One         => ([1, 1, 1, 1, 1, 1], false),
                            MinusOne    => ([1, 1, 1, 0, 1, 0], false),
                            D           => ([0, 0, 1, 1, 0, 0], false),
                            X(a)        => ([1, 1, 0, 0, 0, 0], *a),
                            NotD        => ([0, 0, 1, 1, 0, 1], false),
                            NotX(a)     => ([1, 1, 0, 0, 0, 1], *a),
                            MinusD      => ([0, 0, 1, 1, 1, 1], false),
                            MinusX(a)   => ([1, 1, 0, 0, 1, 1], *a),
                            DPlusOne    => ([0, 1, 1, 1, 1, 1], false),
                            XPlusOne(a) => ([1, 1, 0, 1, 1, 1], *a),
                            DMinusOne   => ([0, 0, 1, 1, 1, 0], false),
                            XMinusOne(a)=> ([1, 1, 0, 0, 1, 0], *a),
                            DPlusX(a)   => ([0, 0, 0, 0, 1, 0], *a),
                            DMinusX(a)  => ([0, 1, 0, 0, 1, 1], *a),
                            XMinusD(a)  => ([0, 0, 0, 1, 1, 1], *a),
                            DAndX(a)    => ([0, 0, 0, 0, 0, 0], *a),
                            DOrX(a)     => ([0, 1, 0, 1, 0, 1], *a),
                        };
                    bits[12] = a;
                    for i in 0 .. 6 { bits[11 - i] = comp[i] != 0; }
                    bits[5] = dest & dest_A != 0;
                    bits[4] = dest & dest_D != 0;
                    bits[3] = dest & dest_M != 0;
                    let jump =
                        match jump {
                            Jump::Null  => [0, 0, 0],
                            Jump::JGT   => [0, 0, 1],
                            Jump::JEQ   => [0, 1, 0],
                            Jump::JGE   => [0, 1, 1],
                            Jump::JLT   => [1, 0, 0],
                            Jump::JNE   => [1, 0, 1],
                            Jump::JLE   => [1, 1, 0],
                            Jump::JMP   => [1, 1, 1]
                        };
                    for i in 0 .. 3 { bits[2 - i] = jump[i] != 0; }
                    bits
                }
            }
        }
    }

    fn make_input(inM: i16, inst: Instruction) -> CpuInput {
        CpuInput {
            inM: int2word(inM),
            instruction: inst.to_bits(),
            reset: false
        }
    }

    fn reset_input() -> CpuInput {
        CpuInput {
            inM: [false; 16],
            instruction: [false; 16],
            reset: true
        }
    }

    fn null_input() -> CpuInput {
        make_input(0, AInstruction(0))
    }
    
    fn set_D_input(inM: i16) -> CpuInput {
        make_input(inM, CInstruction(Computation::X(true), dest_D, Jump::Null))
    }

    fn set_A_input(inM: i16) -> CpuInput {
        make_input(inM, CInstruction(Computation::X(true), dest_A, Jump::Null))
    }

    #[test]
    fn test_pc() {
        let mut cpu = Cpu::new();
        assert_eq!(word2int(cpu.out(null_input()).pc), 0);

        cpu.clock(null_input());
        assert_eq!(word2int(cpu.out(null_input()).pc), 1);

        cpu.clock(null_input());
        assert_eq!(word2int(cpu.out(null_input()).pc), 2);

        cpu.clock(null_input());
        assert_eq!(word2int(cpu.out(null_input()).pc), 3);

        cpu.clock(reset_input());
        assert_eq!(word2int(cpu.out(null_input()).pc), 0);
    }

    #[test]
    fn test_A_instruction() {
        let mut cpu = Cpu::new();
        let addrs = [0, 1, 4, 24, 726, 395];
        for &a in &addrs {
            cpu.clock(make_input(0, AInstruction(a)));
            assert_eq!(word2int(cpu.out(null_input()).addressM), a);
        }
    }

    #[test]
    fn test_writeM() {
        let cpu = Cpu::new();

        let out = cpu.out(make_input(0, CInstruction(Computation::Zero, 0, Jump::Null)));
        assert_eq!(out.writeM, false);

        let out = cpu.out(make_input(0, CInstruction(Computation::Zero, dest_M, Jump::Null)));
        assert_eq!(out.writeM, true);
    }

    #[test]
    fn test_inM_to_outM() {
        let cpu = Cpu::new();
        let values = [123, 456, 789];
        for &x in &values {
            let out = cpu.out(make_input(x, CInstruction(Computation::X(true), 0, Jump::Null)));
            assert_eq!(word2int(out.outM), x);
        }
    }

    #[test]
    fn test_D_register() {
        let mut cpu = Cpu::new();

        // getting D-register value input
        let get_D = make_input(0, CInstruction(Computation::D, 0, Jump::Null));

        // get D-register value
        assert_eq!(word2int(cpu.out(get_D).outM), 0);

        // set 1 to D-register
        cpu.clock(make_input(0, CInstruction(Computation::One, dest_D, Jump::Null)));

        // get D-register value
        assert_eq!(word2int(cpu.out(get_D).outM), 1);

        // set -1 to D-register
        cpu.clock(make_input(0, CInstruction(Computation::MinusOne, dest_D, Jump::Null)));

        // get D-register value
        assert_eq!(word2int(cpu.out(get_D).outM), -1);

        // set 1 to no register
        cpu.clock(make_input(0, CInstruction(Computation::One, 0, Jump::Null)));

        // D-register must be kept as -1
        assert_eq!(word2int(cpu.out(get_D).outM), -1);

        let values = [27, 430, 736, 285];
        for &inM in &values {
            // set inM to D-register
            cpu.clock(set_D_input(inM));
            assert_eq!(word2int(cpu.out(get_D).outM), inM);
        }
    }

    #[test]
    fn test_A_register() {
        let mut cpu = Cpu::new();

        // getting A-register value input
        let get_A = make_input(0, CInstruction(Computation::X(false), 0, Jump::Null));

        // get A-register value
        assert_eq!(word2int(cpu.out(get_A).outM), 0);

        // set 1 to A-register
        cpu.clock(make_input(0, CInstruction(Computation::One, dest_A, Jump::Null)));

        // get A-register value
        assert_eq!(word2int(cpu.out(get_A).outM), 1);

        // set -1 to A-register
        cpu.clock(make_input(0, CInstruction(Computation::MinusOne, dest_A, Jump::Null)));

        // get A-register value
        assert_eq!(word2int(cpu.out(get_A).outM), -1);

        // set 1 to no register
        cpu.clock(make_input(0, CInstruction(Computation::One, 0, Jump::Null)));

        // D-register must be kept as -1
        assert_eq!(word2int(cpu.out(get_A).outM), -1);

        let values = [27, 430, 736, 285];
        for &inM in &values {
            // set inM to A-register
            cpu.clock(set_A_input(inM));
            assert_eq!(word2int(cpu.out(get_A).outM), inM);
        }
    }

    #[test]
    fn test_add() {
        let mut cpu = Cpu::new();

        let D_plus_A = make_input(0, CInstruction(Computation::DPlusX(false), 0, Jump::Null));

        fn D_plus_M(inM: i16) -> CpuInput {
            make_input(inM, CInstruction(Computation::DPlusX(true), 0, Jump::Null))
        }

        let values = [2, 5, 34, 27, 731];
        for &d in &values {
        for &a in &values {
            cpu.clock(set_D_input(d));
            cpu.clock(set_A_input(a));
            assert_eq!(word2int(cpu.out(D_plus_A).outM), d + a);
        } }
        for &d in &values {
        for &m in &values {
            cpu.clock(set_D_input(d));
            assert_eq!(word2int(cpu.out(D_plus_M(m)).outM), d + m);
        } }
    }

    fn get_pc_address(address: i16, comp: Computation, jump: Jump) -> i16 {
        let mut cpu = Cpu::new();
        cpu.clock(make_input(0, AInstruction(address)));
        cpu.clock(make_input(0, CInstruction(comp, 0, jump)));
        word2int(cpu.out(null_input()).pc)
    }

    #[test]
    fn test_jump() {
        let address = 123;

        // JMP
        assert_eq!(get_pc_address(address, Computation::Zero, Jump::JMP), address);

        // JGT: if out > 0 jump
        let jump = Jump::JGT;
        assert_eq!(get_pc_address(address, Computation::Zero, jump), 2);
        assert_eq!(get_pc_address(address, Computation::One, jump), address);
        assert_eq!(get_pc_address(address, Computation::MinusOne, jump), 2);

        // JEQ: if out == 0 jump
        let jump = Jump::JEQ;
        assert_eq!(get_pc_address(address, Computation::Zero, jump), address);
        assert_eq!(get_pc_address(address, Computation::One, jump), 2);
        assert_eq!(get_pc_address(address, Computation::MinusOne, jump), 2);

        // JGE: if out >= 0 jump
        let jump = Jump::JGE;
        assert_eq!(get_pc_address(address, Computation::Zero, jump), address);
        assert_eq!(get_pc_address(address, Computation::One, jump), address);
        assert_eq!(get_pc_address(address, Computation::MinusOne, jump), 2);

        // JLT: if out <  0 jump
        let jump = Jump::JLT;
        assert_eq!(get_pc_address(address, Computation::Zero, jump), 2);
        assert_eq!(get_pc_address(address, Computation::One, jump), 2);
        assert_eq!(get_pc_address(address, Computation::MinusOne, jump), address);

        // JNE: if out != 0 jump
        let jump = Jump::JNE;
        assert_eq!(get_pc_address(address, Computation::Zero, jump), 2);
        assert_eq!(get_pc_address(address, Computation::One, jump), address);
        assert_eq!(get_pc_address(address, Computation::MinusOne, jump), address);

        // JLE: if out <= 0 jump
        let jump = Jump::JLE;
        assert_eq!(get_pc_address(address, Computation::Zero, jump), address);
        assert_eq!(get_pc_address(address, Computation::One, jump), 2);
        assert_eq!(get_pc_address(address, Computation::MinusOne, jump), address);
    }
}
