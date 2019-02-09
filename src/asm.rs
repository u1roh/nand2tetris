use crate::given::debug::int2word;

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

pub mod dest {
    pub static A: u8 = 0b100;
    pub static D: u8 = 0b010;
    pub static M: u8 = 0b001;
}

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
    pub fn encode(&self) -> i16 {
        match self {
            AInstruction(address) => {
                assert!(*address >= 0); // A-instruction's MSB = 0
                *address
            },
            CInstruction(comp, dest, jump) => {
                use Computation::*;
                let mut bits = 1 << 15; // C-instruction's MSB = 1
                let (comp, a) = match comp {
                    Zero        => (0b101010, false),
                    One         => (0b111111, false),
                    MinusOne    => (0b111010, false),
                    D           => (0b001100, false),
                    X(a)        => (0b110000, *a),
                    NotD        => (0b001101, false),
                    NotX(a)     => (0b110001, *a),
                    MinusD      => (0b001111, false),
                    MinusX(a)   => (0b110011, *a),
                    DPlusOne    => (0b011111, false),
                    XPlusOne(a) => (0b110111, *a),
                    DMinusOne   => (0b001110, false),
                    XMinusOne(a)=> (0b110010, *a),
                    DPlusX(a)   => (0b000010, *a),
                    DMinusX(a)  => (0b010011, *a),
                    XMinusD(a)  => (0b000111, *a),
                    DAndX(a)    => (0b000000, *a),
                    DOrX(a)     => (0b010101, *a),
                };
                bits |= comp << 6;
                if a { bits |= 1 << 12; }
                bits |= (*dest as i16) << 3;
                bits |= match jump {
                    Jump::Null  => 0b000,
                    Jump::JGT   => 0b001,
                    Jump::JEQ   => 0b010,
                    Jump::JGE   => 0b011,
                    Jump::JLT   => 0b100,
                    Jump::JNE   => 0b101,
                    Jump::JLE   => 0b110,
                    Jump::JMP   => 0b111
                };
                bits
            }
        }
    }
    pub fn to_bits(&self) -> [bool; 16] {
        int2word(self.encode())
    }
}
