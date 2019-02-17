
pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}

pub struct Flipflop { bit: bool }

impl Flipflop {
    pub fn new() -> Self {
        Flipflop{ bit: false }
    }
    pub fn out(&self) -> bool {
        self.bit
    }
    pub fn clock(&mut self, a: bool) {
        self.bit = a;
    }
}


// 16bit word
pub type Word = [bool; 16];

pub mod debug {
    use super::Word;

    pub fn int2word(a: i16) -> Word {
        let mut word = [false; 16];
        for i in 0 .. 16 { word[i] = a & (1 << i) != 0; }
        word
    }

    pub fn word2int(a: Word) -> i16 {
        let mut n = 0;
        for i in 0 .. 16 { if a[i] { n |= 1 << i; } }
        n
    }

    /*
    #[cfg(test)]
    pub fn bits2int(bits: &[bool]) -> i16 {
        let mut n = 0;
        for i in 0 .. bits.len() { if bits[i] { n |= 1 << i; } }
        n
    }
    */
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nand() {
        assert_eq!(nand(false, false), true);
        assert_eq!(nand(true, false), true);
        assert_eq!(nand(false, true), true);
        assert_eq!(nand(true, true), false);
    }

    #[test]
    fn test_flipflop() {
        let mut flipflop = Flipflop::new();
        assert_eq!(flipflop.out(), false);
        flipflop.clock(true);
        assert_eq!(flipflop.out(), true);
        flipflop.clock(true);
        assert_eq!(flipflop.out(), true);
        flipflop.clock(false);
        assert_eq!(flipflop.out(), false);
        flipflop.clock(true);
        assert_eq!(flipflop.out(), true);
        flipflop.clock(false);
        assert_eq!(flipflop.out(), false);
    }
}