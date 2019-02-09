use crate::given::int2word;

pub enum Computation {
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

pub static dest_A: u8 = 0b001;
pub static dest_D: u8 = 0b010;
pub static dest_M: u8 = 0b100;

#[derive(Clone, Copy)]
pub enum Jump {
    Null,   // No jump
    JGT,    // if out >  0 jump
    JEQ,    // if out == 0 jump
    JGE,    // if out >= 0 jump
    JLT,    // if out <  0 jump
    JNE,    // if out != 0 jump
    JLE,    // if out <= 0 jump
    JMP     // jump
}

pub enum Instruction {
    AInstruction(i16),
    CInstruction(Computation, u8, Jump)
}

pub use Instruction::*;

impl Instruction {
    pub fn to_bits(&self) -> [bool; 16] {
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
