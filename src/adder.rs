use crate::given::*;
use crate::gate::*;

pub fn half_adder(a: bool, b: bool) -> (bool, bool) {
    (xor(a, b), and(a, b))
}

pub fn full_adder(a: bool, b: bool, carry: bool) -> (bool, bool) {
    let (sum1, carry1) = half_adder(a, b);
    let (sum2, carry2) = half_adder(sum1, carry);
    (sum2, or(carry1, carry2))
}

pub fn add16(a: Word, b: Word) -> Word {
    let mut sum = [false; 16];
    let mut carry = false;
    for i in 0 .. 16 {
        let (s, c) = full_adder(a[i], b[i], carry);
        sum[i] = s;
        carry = c;
    }
    sum
}

pub fn inc16(a: Word) -> Word {
    let mut sum = [false; 16];
    let mut carry = true;
    for i in 0 .. 16 {
        let (s, c) = half_adder(a[i], carry);
        sum[i] = s;
        carry = c;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_half_adder() {
        assert_eq!(half_adder(false, false), (false, false));
        assert_eq!(half_adder(true, false), (true, false));
        assert_eq!(half_adder(false, true), (true, false));
        assert_eq!(half_adder(true, true), (false, true));
    }

    #[test]
    fn test_full_adder() {
        // carry == false の場合（half_adder と同じ挙動）
        assert_eq!(full_adder(false, false, false), (false, false));
        assert_eq!(full_adder(true, false, false), (true, false));
        assert_eq!(full_adder(false, true, false), (true, false));
        assert_eq!(full_adder(true, true, false), (false, true));

        // carry == true の場合
        assert_eq!(full_adder(false, false, true), (true, false));
        assert_eq!(full_adder(true, false, true), (false, true));
        assert_eq!(full_adder(false, true, true), (false, true));
        assert_eq!(full_adder(true, true, true), (true, true));
    }

    #[test]
    fn test_add16() {
        assert_eq!(add16(int2word(0), int2word(0)), int2word(0));
        assert_eq!(add16(int2word(123), int2word(456)), int2word(579));
        let nums = [1, 2, 100, 3722, 2984, 25, 74];
        for &n in &nums {
        for &m in &nums {
            assert_eq!(add16(int2word(n), int2word(m)), int2word(n + m));
        } }
    }

    #[test]
    fn test_inc16() {
        let nums = [1, 2, 100, 3722, 2984, 25, 74];
        for &n in &nums {
            assert_eq!(word2int(inc16(int2word(n))), n+1);
        }
    }
}