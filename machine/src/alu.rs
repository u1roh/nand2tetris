use crate::given::*;
use crate::gate::*;
use crate::adder;

pub struct AluOutput {
    pub out: Word,  // 16-bit output
    pub zr: bool,   // True iff out = 0
    pub ng: bool,   // True iff out < 0
}

pub fn alu(
    x: Word, y: Word,   // Two 16-bit data inputs
    zx: bool,   // Zero the x input
    nx: bool,   // Negate the x input
    zy: bool,   // Zero the y input
    ny: bool,   // Negate the y input
    f: bool,    // Function code: true for Add, false for And
    no: bool    // Negate the out output
) -> AluOutput {
    let x = mux16(x, [false; 16], zx);
    let x = mux16(x, not16(x), nx);
    let y = mux16(y, [false; 16], zy);
    let y = mux16(y, not16(y), ny);
    let out = mux16(and16(x, y), adder::add16(x, y), f);
    let out = mux16(out, not16(out), no);
    let zr = not(or(
        or8way([out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]]),
        or8way([out[8], out[9], out[10], out[11], out[12], out[13], out[14], out[15]])));
    let ng = out[15];
    AluOutput{ out: out, zr: zr, ng: ng }
}


#[cfg(test)]
mod tests {
    use super::*;
    use debug::*;

    fn assert_alu(x: i16, y: i16, zx: i8, nx: i8, zy: i8, ny: i8, f: i8, no: i8, expected: i16) {
        let wx = int2word(x);
        let wy = int2word(y);
        let result = alu(wx, wy, zx != 0, nx != 0, zy != 0, ny != 0, f != 0, no != 0);
        assert_eq!(word2int(result.out), expected);
        assert_eq!(result.zr, expected == 0);
        assert_eq!(result.ng, expected < 0);
    }

    #[test]
    fn test_alu() {
        let data = [73, 61, 973, 294, 429];
        for &x in &data {
        for &y in &data {
            assert_alu(x, y, 1, 0, 1, 0, 1, 0, 0);
            assert_alu(x, y, 1, 1, 1, 1, 1, 1, 1);
            assert_alu(x, y, 1, 1, 1, 0, 1, 0, -1);
            assert_alu(x, y, 0, 0, 1, 1, 0, 0, x);
            assert_alu(x, y, 0, 0, 1, 1, 0, 1, !x);
            assert_alu(x, y, 1, 1, 0, 0, 0, 0, y);
            assert_alu(x, y, 1, 1, 0, 0, 0, 1, !y);
            assert_alu(x, y, 0, 0, 1, 1, 1, 1, -x);
            assert_alu(x, y, 1, 1, 0, 0, 1, 1, -y);
            assert_alu(x, y, 0, 1, 1, 1, 1, 1, x + 1);
            assert_alu(x, y, 1, 1, 0, 1, 1, 1, y + 1);
            assert_alu(x, y, 0, 0, 1, 1, 1, 0, x - 1);
            assert_alu(x, y, 1, 1, 0, 0, 1, 0, y - 1);
            assert_alu(x, y, 0, 0, 0, 0, 1, 0, x + y);
            assert_alu(x, y, 0, 1, 0, 0, 1, 1, x - y);
            assert_alu(x, y, 0, 0, 0, 1, 1, 1, y - x);
            assert_alu(x, y, 0, 0, 0, 0, 0, 0, x & y);
            assert_alu(x, y, 0, 1, 0, 1, 0, 1, x | y);
        } }

    }
}
