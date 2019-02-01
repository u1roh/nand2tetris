
pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}


// 16bit word
pub type Word = [bool; 16];

pub fn int2word(a: u16) -> Word {
    let mut word = [false; 16];
    for i in 0 .. 16 { word[i] = a & (1 << i) != 0; }
    word
}

pub fn word2int(a: Word) -> u16 {
    let mut n = 0;
    for i in 0 .. 16 { if a[i] { n |= 1 << i; } }
    n
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
}