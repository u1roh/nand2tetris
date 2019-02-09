// Machine Language Specification
use crate::given::Word;
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
    pub fn to_word(&self) -> Word {
        int2word(self.encode())
    }
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
    pub fn decode(instruction: i16) -> Self {
        if instruction >= 0 { AInstruction(instruction) } else {
            use Computation::*;
            let a = instruction & (1 << 12) != 0;
            let comp = match (instruction >> 6) & 0b111111 {
                0b101010 => Zero,
                0b111111 => One,
                0b111010 => MinusOne,
                0b001100 => D,
                0b110000 => X(a),
                0b001101 => NotD,
                0b110001 => NotX(a),
                0b001111 => MinusD,
                0b110011 => MinusX(a),
                0b011111 => DPlusOne,
                0b110111 => XPlusOne(a),
                0b001110 => DMinusOne,
                0b110010 => XMinusOne(a),
                0b000010 => DPlusX(a),
                0b010011 => DMinusX(a),
                0b000111 => XMinusD(a),
                0b000000 => DAndX(a),
                0b010101 => DOrX(a),
                _ => panic!("invalid instruction")
            };
            let jump = match instruction & 0b111 {
                0b000 => Jump::Null,
                0b001 => Jump::JGT,
                0b010 => Jump::JEQ,
                0b011 => Jump::JGE,
                0b100 => Jump::JLT,
                0b101 => Jump::JNE,
                0b110 => Jump::JLE,
                0b111 => Jump::JMP,
                _ => panic!("invalid instruction")
            };
            let dest = (instruction >> 3) & 0b111;
            CInstruction(comp, dest as u8, jump)
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AInstruction(address) => {
                assert!(*address >= 0); // A-instruction's MSB = 0
                write!(f, "@{}", *address)
            },
            CInstruction(comp, dest, jump) => {
                use Computation::*;
                let comp = match comp {
                    Zero        => "0",
                    One         => "1",
                    MinusOne    => "-1",
                    D           => "D",
                    X(a)        => if *a { "M" } else { "A" },
                    NotD        => "!D",
                    NotX(a)     => if *a { "!M" } else { "!A" },
                    MinusD      => "-D",
                    MinusX(a)   => if *a { "-M" } else { "-A" },
                    DPlusOne    => "D+1",
                    XPlusOne(a) => if *a { "M+1" } else { "A+1" },
                    DMinusOne   => "D-1",
                    XMinusOne(a)=> if *a { "M-1" } else { "A-1" },
                    DPlusX(a)   => if *a { "D+M" } else { "D+A" },
                    DMinusX(a)  => if *a { "D-M" } else { "D-A" },
                    XMinusD(a)  => if *a { "M-D" } else { "A-D" },
                    DAndX(a)    => if *a { "D&M" } else { "D&A" },
                    DOrX(a)     => if *a { "D|M" } else { "D|A" }
                };
                let dest = match dest {
                    0b000 => "null",
                    0b001 => "M",
                    0b010 => "D",
                    0b011 => "MD",
                    0b100 => "A",
                    0b101 => "AM",
                    0b110 => "AD",
                    0b111 => "AMD",
                    _ => panic!("invalid destination")
                };
                let jump = match jump {
                    Jump::Null  => "null",
                    Jump::JGT   => "JGT",
                    Jump::JEQ   => "JEQ",
                    Jump::JGE   => "JGE",
                    Jump::JLT   => "JLT",
                    Jump::JNE   => "JNE",
                    Jump::JLE   => "JLE",
                    Jump::JMP   => "JMP"
                };
                write!(f, "{}={};{}", dest, comp, jump)
            }
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn assert_instruction(inst: Instruction) {
        let a = inst.encode();
        let b = Instruction::decode(a).encode();
        assert_eq!(a, b);
    }

    #[test]
    fn test_instructions() {
        assert_instruction(AInstruction(10));
        assert_instruction(CInstruction(Computation::Zero, 0, Jump::Null));
        assert_instruction(CInstruction(Computation::One, dest::A, Jump::Null));
        assert_instruction(CInstruction(Computation::DPlusX(true), dest::M, Jump::JGT));
    }
}