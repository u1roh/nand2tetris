use crate::given::*;
use crate::gate::*;
use crate::ram::*;
use crate::alu::*;


pub struct CpuOutput {
    pub outM: Word,
    pub writeM: bool,
    pub addressM: Word,
    pub pc: Word
}

pub struct Cpu {
    A: Register,    // A register
    D: Register,    // D register
    PC: Counter,    // Program counter
}

impl Cpu {
    pub fn new() -> Self {
        Self { A: Register::new(), D: Register::new(), PC: Counter::new() }
    }
    pub fn clock(&mut self, inM: Word, instruction: Word, reset: bool) -> CpuOutput {
        let is_C_instruction = instruction[15];
        self.A.clock(instruction, true);
        self.PC.clock([false; 16], true, false, reset);
        let out = alu(
            self.D.out(),   // x = D
            mux16(self.A.out(), inM, instruction[12]),  // y = A or M
            instruction[11],    // zx
            instruction[10],    // nx
            instruction[ 9],    // zy
            instruction[ 8],    // ny
            instruction[ 7],    // f
            instruction[ 6]);   // no
        CpuOutput{ outM: out.out, writeM: instruction[3], addressM: self.A.out(), pc: self.PC.out() }
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

    struct Destination {
        A: bool,
        D: bool,
        M: bool
    }

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
        CInstruction(Computation, Destination, Jump)
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
                    bits[5] = dest.A;
                    bits[4] = dest.D;
                    bits[3] = dest.M;
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

    #[test]
    fn test_pc() {
        let mut cpu = Cpu::new();

        let out = cpu.clock([false; 16], [false; 16], false);
        assert_eq!(word2int(out.pc), 1);

        let out = cpu.clock([false; 16], [false; 16], false);
        assert_eq!(word2int(out.pc), 2);

        let out = cpu.clock([false; 16], [false; 16], false);
        assert_eq!(word2int(out.pc), 3);

        let out = cpu.clock([false; 16], [false; 16], true);
        assert_eq!(word2int(out.pc), 0);
    }

    impl Cpu {
        fn clock_for_test(&mut self, inM: i16, inst: Instruction) -> CpuOutput {
            self.clock(int2word(inM), inst.to_bits(), false)
        }
    }

    #[test]
    fn test_A_instruction() {
        let mut cpu = Cpu::new();
        let addrs = [0, 1, 4, 24, 726, 395];
        for &a in &addrs {
            let out = cpu.clock_for_test(0, AInstruction(a));
            assert_eq!(word2int(out.addressM), a);
        }
    }

    #[test]
    fn test_writeM() {
        let mut cpu = Cpu::new();

        let out = cpu.clock_for_test(0, CInstruction(
            Computation::Zero,
            Destination{ A: false, D: false, M: false },
            Jump::Null));
        assert_eq!(out.writeM, false);

        let out = cpu.clock_for_test(0, CInstruction(
            Computation::Zero,
            Destination{ A: false, D: false, M: true },
            Jump::Null));
        assert_eq!(out.writeM, true);
    }

    #[test]
    fn test_inM_to_outM() {
        let mut cpu = Cpu::new();
        let values = [123, 456, 789];
        for &x in &values {
            let out = cpu.clock_for_test(x, CInstruction(
                Computation::X(true),
                Destination{ A: false, D: false, M: false },
                Jump::Null));
            assert_eq!(word2int(out.outM), x);
        }
    }

    #[test]
    fn test_D_register() {
        let mut cpu = Cpu::new();

        // get D-register value
        let out = cpu.clock_for_test(0, CInstruction(
            Computation::D,
            Destination{ A: false, D: false, M: false },
            Jump::Null));
        assert_eq!(word2int(out.outM), 0);

        // set 1 to D-register
        let out = cpu.clock_for_test(0, CInstruction(
            Computation::One,
            Destination{ A: false, D: true, M: false },
            Jump::Null));
        assert_eq!(word2int(out.outM), 0);

        // get D-register value
        let out = cpu.clock_for_test(0, CInstruction(
            Computation::D,
            Destination{ A: false, D: false, M: false },
            Jump::Null));
        assert_eq!(word2int(out.outM), 1);

    }
}
