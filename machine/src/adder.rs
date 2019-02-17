use crate::given::*;
use crate::gate::*;

pub fn half_adder(a: bool, b: bool) -> [bool; 2] {
    [xor(a, b), and(a, b)]
}

pub fn full_adder(a: bool, b: bool, carry: bool) -> [bool; 2] {
    let [sum1, carry1] = half_adder(a, b);
    let [sum2, carry2] = half_adder(sum1, carry);
    [sum2, or(carry1, carry2)]
}

pub fn add16(a: Word, b: Word) -> Word {
    let [sum00, carry] = half_adder(a[ 0], b[ 0]);
    let [sum01, carry] = full_adder(a[ 1], b[ 1], carry);
    let [sum02, carry] = full_adder(a[ 2], b[ 2], carry);
    let [sum03, carry] = full_adder(a[ 3], b[ 3], carry);
    let [sum04, carry] = full_adder(a[ 4], b[ 4], carry);
    let [sum05, carry] = full_adder(a[ 5], b[ 5], carry);
    let [sum06, carry] = full_adder(a[ 6], b[ 6], carry);
    let [sum07, carry] = full_adder(a[ 7], b[ 7], carry);
    let [sum08, carry] = full_adder(a[ 8], b[ 8], carry);
    let [sum09, carry] = full_adder(a[ 9], b[ 9], carry);
    let [sum10, carry] = full_adder(a[10], b[10], carry);
    let [sum11, carry] = full_adder(a[11], b[11], carry);
    let [sum12, carry] = full_adder(a[12], b[12], carry);
    let [sum13, carry] = full_adder(a[13], b[13], carry);
    let [sum14, carry] = full_adder(a[14], b[14], carry);
    let [sum15, _    ] = full_adder(a[15], b[15], carry);
    [
        sum00, sum01, sum02, sum03,
        sum04, sum05, sum06, sum07,
        sum08, sum09, sum10, sum11,
        sum12, sum13, sum14, sum15,
    ]
}

pub fn inc16(a: Word) -> Word {
    let [sum00, carry] = half_adder(a[ 0], true);
    let [sum01, carry] = half_adder(a[ 1], carry);
    let [sum02, carry] = half_adder(a[ 2], carry);
    let [sum03, carry] = half_adder(a[ 3], carry);
    let [sum04, carry] = half_adder(a[ 4], carry);
    let [sum05, carry] = half_adder(a[ 5], carry);
    let [sum06, carry] = half_adder(a[ 6], carry);
    let [sum07, carry] = half_adder(a[ 7], carry);
    let [sum08, carry] = half_adder(a[ 8], carry);
    let [sum09, carry] = half_adder(a[ 9], carry);
    let [sum10, carry] = half_adder(a[10], carry);
    let [sum11, carry] = half_adder(a[11], carry);
    let [sum12, carry] = half_adder(a[12], carry);
    let [sum13, carry] = half_adder(a[13], carry);
    let [sum14, carry] = half_adder(a[14], carry);
    let [sum15, _    ] = half_adder(a[15], carry);
    [
        sum00, sum01, sum02, sum03,
        sum04, sum05, sum06, sum07,
        sum08, sum09, sum10, sum11,
        sum12, sum13, sum14, sum15,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use debug::*;

    #[test]
    fn test_half_adder() {
        assert_eq!(half_adder(false, false), [false, false]);
        assert_eq!(half_adder(true, false), [true, false]);
        assert_eq!(half_adder(false, true), [true, false]);
        assert_eq!(half_adder(true, true), [false, true]);
    }

    #[test]
    fn test_full_adder() {
        // carry == false の場合（half_adder と同じ挙動）
        assert_eq!(full_adder(false, false, false), [false, false]);
        assert_eq!(full_adder(true, false, false), [true, false]);
        assert_eq!(full_adder(false, true, false), [true, false]);
        assert_eq!(full_adder(true, true, false), [false, true]);

        // carry == true の場合
        assert_eq!(full_adder(false, false, true), [true, false]);
        assert_eq!(full_adder(true, false, true), [false, true]);
        assert_eq!(full_adder(false, true, true), [false, true]);
        assert_eq!(full_adder(true, true, true), [true, true]);
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