use crate::given::*;

pub fn not(a: bool) -> bool {
    nand(a, a)
}

pub fn and(a: bool, b: bool) -> bool {
    not(nand(a, b))
}

pub fn or(a: bool, b: bool) -> bool {
    nand(not(a), not(b))
}

pub fn xor(a: bool, b: bool) -> bool {
    or(and(a, not(b)), and(not(a), b))
}

// multiplexor
pub fn mux(a: bool, b: bool, sel: bool) -> bool {
    or(and(a, not(sel)), and(b, sel))
}

// demultiplexor
pub fn dmux(input: bool, sel: bool) -> [bool; 2] {
    [and(input, not(sel)), and(input, sel)]
}

pub fn not16(a: Word) -> Word {
    [
        not(a[ 0]), not(a[ 1]), not(a[ 2]), not(a[ 3]),
        not(a[ 4]), not(a[ 5]), not(a[ 6]), not(a[ 7]),
        not(a[ 8]), not(a[ 9]), not(a[10]), not(a[11]),
        not(a[12]), not(a[13]), not(a[14]), not(a[15]),
    ]
}

pub fn and16(a: Word, b: Word) -> Word {
    [
        and(a[ 0], b[ 0]), and(a[ 1], b[ 1]), and(a[ 2], b[ 2]), and(a[ 3], b[ 3]),
        and(a[ 4], b[ 4]), and(a[ 5], b[ 5]), and(a[ 6], b[ 6]), and(a[ 7], b[ 7]),
        and(a[ 8], b[ 8]), and(a[ 9], b[ 9]), and(a[10], b[10]), and(a[11], b[11]),
        and(a[12], b[12]), and(a[13], b[13]), and(a[14], b[14]), and(a[15], b[15]),
    ]
}

pub fn mux16(a: Word, b: Word, sel: bool) -> Word {
    [
        mux(a[ 0], b[ 0], sel), mux(a[ 1], b[ 1], sel), mux(a[ 2], b[ 2], sel), mux(a[ 3], b[ 3], sel),
        mux(a[ 4], b[ 4], sel), mux(a[ 5], b[ 5], sel), mux(a[ 6], b[ 6], sel), mux(a[ 7], b[ 7], sel),
        mux(a[ 8], b[ 8], sel), mux(a[ 9], b[ 9], sel), mux(a[10], b[10], sel), mux(a[11], b[11], sel),
        mux(a[12], b[12], sel), mux(a[13], b[13], sel), mux(a[14], b[14], sel), mux(a[15], b[15], sel),
    ]
}

pub fn or8way(a: [bool; 8]) -> bool {
    or(or(or(a[0], a[1]), or(a[2], a[3])), or(or(a[4], a[5]), or(a[6], a[7])))
}

pub fn mux4way16(a: Word, b: Word, c: Word, d: Word, sel: [bool; 2]) -> Word {
    mux16(mux16(a, b, sel[0]), mux16(c, d, sel[0]), sel[1])
}

pub fn mux8way16(a: Word, b: Word, c: Word, d: Word, e: Word, f: Word, g: Word, h: Word, sel: [bool; 3]) -> Word {
    mux16(mux4way16(a, b, c, d, [sel[0], sel[1]]), mux4way16(e, f, g, h, [sel[0], sel[1]]), sel[2])
}

pub fn dmux4way(input: bool, sel: [bool; 2]) -> [bool; 4] {
    let [ab, cd] = dmux(input, sel[1]);
    let [a, b] = dmux(ab, sel[0]);
    let [c, d] = dmux(cd, sel[0]);
    [a, b, c, d]
}

pub fn dmux8way(input: bool, sel: [bool; 3]) -> [bool; 8] {
    let [abcd, efgh] = dmux(input, sel[2]);
    let [a, b, c, d] = dmux4way(abcd, [sel[0], sel[1]]);
    let [e, f, g, h] = dmux4way(efgh, [sel[0], sel[1]]);
    [a, b, c, d, e, f, g, h]
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
    fn test_not() {
        assert_eq!(not(true), false);
        assert_eq!(not(false), true);
    }

    #[test]
    fn test_and() {
        assert_eq!(and(false, false), false);
        assert_eq!(and(true, false), false);
        assert_eq!(and(false, true), false);
        assert_eq!(and(true, true), true);
    }

    #[test]
    fn test_or() {
        assert_eq!(or(false, false), false);
        assert_eq!(or(true, false), true);
        assert_eq!(or(false, true), true);
        assert_eq!(or(true, true), true);
    }

    #[test]
    fn test_xor() {
        assert_eq!(xor(false, false), false);
        assert_eq!(xor(true, false), true);
        assert_eq!(xor(false, true), true);
        assert_eq!(xor(true, true), false);
    }

    #[test]
    fn test_mux() {
        let bits = [false, true];
        for &sel in &bits {
            for &a in &bits {
            for &b in &bits {
                assert_eq!(mux(a ,b, sel), if sel { b } else { a });
            } }
        }
    }

    #[test]
    fn test_dmux() {
        let bits = [false, true];
        for &sel in &bits {
            for &x in &bits {
                assert_eq!(dmux(x, sel), if sel { [false, x] } else { [x, false] })
            }
        }
    }

    #[test]
    fn test_or8way() {
        let bits = [false, true];
        for &a0 in &bits {
        for &a1 in &bits {
        for &a2 in &bits {
        for &a3 in &bits {
        for &a4 in &bits {
        for &a5 in &bits {
        for &a6 in &bits {
        for &a7 in &bits {
            assert_eq!(or8way([a0, a1, a2, a3, a4, a5, a6, a7]), a0 || a1 || a2 || a3 || a4 || a5 || a6 || a7);
        }}}}}}}}
    }

    use debug::*;

    #[test]
    fn test_mux4way16() {
        let a = int2word(1);
        let b = int2word(2);
        let c = int2word(3);
        let d = int2word(4);
        for &sel0 in &[false, true] {
        for &sel1 in &[false, true] {
            let expected = match (sel0, sel1) {
                (false, false) => a,
                (true,  false) => b,
                (false, true ) => c,
                (true,  true ) => d,
            };
            assert_eq!(mux4way16(a, b, c, d, [sel0, sel1]), expected);
        } }
    }

    #[test]
    fn test_mux8way16() {
        let a = int2word(1);
        let b = int2word(2);
        let c = int2word(3);
        let d = int2word(4);
        let e = int2word(5);
        let f = int2word(6);
        let g = int2word(7);
        let h = int2word(8);
        for &sel0 in &[false, true] {
        for &sel1 in &[false, true] {
        for &sel2 in &[false, true] {
            let expected = match (sel0, sel1, sel2) {
                (false, false, false) => a,
                (true,  false, false) => b,
                (false, true , false) => c,
                (true,  true , false) => d,
                (false, false, true ) => e,
                (true,  false, true ) => f,
                (false, true , true ) => g,
                (true,  true , true ) => h,
            };
            assert_eq!(mux8way16(a, b, c, d, e, f, g, h, [sel0, sel1, sel2]), expected);
        } } }
    }

    #[test]
    fn test_dmux4way() {
        for &input in &[false, true] {
            for &sel0 in &[false, true] {
            for &sel1 in &[false, true] {
                let expected = match (sel0, sel1) {
                    (false, false) => [input, false, false, false],
                    (true,  false) => [false, input, false, false],
                    (false, true ) => [false, false, input, false],
                    (true,  true ) => [false, false, false, input],
                };
                assert_eq!(dmux4way(input, [sel0, sel1]), expected);
            } }
        }
    }

    #[test]
    fn test_dmux8way() {
        for &input in &[false, true] {
            for &sel0 in &[false, true] {
            for &sel1 in &[false, true] {
            for &sel2 in &[false, true] {
                let expected = match (sel0, sel1, sel2) {
                    (false, false, false) => [input, false, false, false, false, false, false, false],
                    (true,  false, false) => [false, input, false, false, false, false, false, false],
                    (false, true , false) => [false, false, input, false, false, false, false, false],
                    (true,  true , false) => [false, false, false, input, false, false, false, false],
                    (false, false, true ) => [false, false, false, false, input, false, false, false],
                    (true,  false, true ) => [false, false, false, false, false, input, false, false],
                    (false, true , true ) => [false, false, false, false, false, false, input, false],
                    (true,  true , true ) => [false, false, false, false, false, false, false, input],
                };
                assert_eq!(dmux8way(input, [sel0, sel1, sel2]), expected);
            } } }
        }
    }
}