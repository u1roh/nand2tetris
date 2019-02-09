use crate::given::*;
use crate::gate::*;
use crate::adder::*;

// single bit register
pub struct BitRegister { flipflop: Flipflop }
impl BitRegister {
    pub fn new() -> Self {
        BitRegister{ flipflop: Flipflop::new() }
    }
    pub fn out(&self) -> bool {
        self.flipflop.out()
    }
    pub fn clock(&mut self, input: bool, load: bool)  {
        self.flipflop.clock(mux(self.out(), input, load));
    }
}

// 16bit register
pub struct Register { bits: [BitRegister; 16] }
impl Register {
    pub fn new() -> Self {
        Register{ bits: [
            BitRegister::new(), BitRegister::new(), BitRegister::new(), BitRegister::new(),
            BitRegister::new(), BitRegister::new(), BitRegister::new(), BitRegister::new(),
            BitRegister::new(), BitRegister::new(), BitRegister::new(), BitRegister::new(),
            BitRegister::new(), BitRegister::new(), BitRegister::new(), BitRegister::new(),
        ] }
    }
    pub fn out(&self) -> Word {
        [ self.bits[ 0].out(), self.bits[ 1].out(), self.bits[ 2].out(), self.bits[ 3].out(),
          self.bits[ 4].out(), self.bits[ 5].out(), self.bits[ 6].out(), self.bits[ 7].out(),
          self.bits[ 8].out(), self.bits[ 9].out(), self.bits[10].out(), self.bits[11].out(),
          self.bits[12].out(), self.bits[13].out(), self.bits[14].out(), self.bits[15].out(),
        ]
    }
    pub fn clock(&mut self, input: Word, load: bool) {
        for i in 0 .. 16 { self.bits[i].clock(input[i], load) }
    }
}

pub struct RAM8 { registers: [Register; 8] }
impl RAM8 {
    pub fn new() -> Self {
        RAM8 { registers: [
            Register::new(), Register::new(), Register::new(), Register::new(),
            Register::new(), Register::new(), Register::new(), Register::new(),
        ]}
    }
    pub fn out(&self, address: [bool; 3]) -> Word {
        mux8way16(
            self.registers[0].out(), self.registers[1].out(), self.registers[2].out(), self.registers[3].out(),
            self.registers[4].out(), self.registers[5].out(), self.registers[6].out(), self.registers[7].out(),
            address)
    }
    pub fn clock(&mut self, address: [bool; 3], input: Word, load: bool) {
        let load = dmux8way(load, address);
        self.registers[0].clock(input, load.0);
        self.registers[1].clock(input, load.1);
        self.registers[2].clock(input, load.2);
        self.registers[3].clock(input, load.3);
        self.registers[4].clock(input, load.4);
        self.registers[5].clock(input, load.5);
        self.registers[6].clock(input, load.6);
        self.registers[7].clock(input, load.7);
    }
}

pub struct RAM64 { rams: [RAM8; 8] }
impl RAM64 {
    pub fn new() -> Self {
        Self { rams: [
            RAM8::new(), RAM8::new(), RAM8::new(), RAM8::new(),
            RAM8::new(), RAM8::new(), RAM8::new(), RAM8::new(),
        ]}
    }
    pub fn out(&self, address: [bool; 6]) -> Word {
        let lo = [address[0], address[1], address[2]];
        let hi = [address[3], address[4], address[5]];
        mux8way16(
            self.rams[0].out(lo), self.rams[1].out(lo), self.rams[2].out(lo), self.rams[3].out(lo),
            self.rams[4].out(lo), self.rams[5].out(lo), self.rams[6].out(lo), self.rams[7].out(lo),
            hi)
    }
    pub fn clock(&mut self, address: [bool; 6], input: Word, load: bool) {
        let lo = [address[0], address[1], address[2]];
        let hi = [address[3], address[4], address[5]];
        let load = dmux8way(load, hi);
        self.rams[0].clock(lo, input, load.0);
        self.rams[1].clock(lo, input, load.1);
        self.rams[2].clock(lo, input, load.2);
        self.rams[3].clock(lo, input, load.3);
        self.rams[4].clock(lo, input, load.4);
        self.rams[5].clock(lo, input, load.5);
        self.rams[6].clock(lo, input, load.6);
        self.rams[7].clock(lo, input, load.7);
    }
}

pub struct RAM512 { rams: [RAM64; 8] }
impl RAM512 {
    pub fn new() -> Self {
        Self { rams: [
            RAM64::new(), RAM64::new(), RAM64::new(), RAM64::new(),
            RAM64::new(), RAM64::new(), RAM64::new(), RAM64::new(),
        ]}
    }
    pub fn out(&self, address: [bool; 9]) -> Word {
        let lo = [address[0], address[1], address[2], address[3], address[4], address[5]];
        let hi = [address[6], address[7], address[8]];
        mux8way16(
            self.rams[0].out(lo), self.rams[1].out(lo), self.rams[2].out(lo), self.rams[3].out(lo),
            self.rams[4].out(lo), self.rams[5].out(lo), self.rams[6].out(lo), self.rams[7].out(lo),
            hi)
    }
    pub fn clock(&mut self, address: [bool; 9], input: Word, load: bool) {
        let lo = [address[0], address[1], address[2], address[3], address[4], address[5]];
        let hi = [address[6], address[7], address[8]];
        let load = dmux8way(load, hi);
        self.rams[0].clock(lo, input, load.0);
        self.rams[1].clock(lo, input, load.1);
        self.rams[2].clock(lo, input, load.2);
        self.rams[3].clock(lo, input, load.3);
        self.rams[4].clock(lo, input, load.4);
        self.rams[5].clock(lo, input, load.5);
        self.rams[6].clock(lo, input, load.6);
        self.rams[7].clock(lo, input, load.7);
    }
}

pub struct RAM4K { rams: [RAM512; 8] }
impl RAM4K {
    pub fn new() -> Self {
        Self { rams: [
            RAM512::new(), RAM512::new(), RAM512::new(), RAM512::new(),
            RAM512::new(), RAM512::new(), RAM512::new(), RAM512::new(),
        ]}
    }
    pub fn out(&self, address: [bool; 12]) -> Word {
        let lo = [
            address[0], address[1], address[2],
            address[3], address[4], address[5],
            address[6], address[7], address[8] ];
        let hi = [address[9], address[10], address[11]];
        mux8way16(
            self.rams[0].out(lo), self.rams[1].out(lo), self.rams[2].out(lo), self.rams[3].out(lo),
            self.rams[4].out(lo), self.rams[5].out(lo), self.rams[6].out(lo), self.rams[7].out(lo),
            hi)
    }
    pub fn clock(&mut self, address: [bool; 12], input: Word, load: bool) {
        let lo = [
            address[0], address[1], address[2],
            address[3], address[4], address[5],
            address[6], address[7], address[8] ];
        let hi = [address[9], address[10], address[11]];
        let load = dmux8way(load, hi);
        self.rams[0].clock(lo, input, load.0);
        self.rams[1].clock(lo, input, load.1);
        self.rams[2].clock(lo, input, load.2);
        self.rams[3].clock(lo, input, load.3);
        self.rams[4].clock(lo, input, load.4);
        self.rams[5].clock(lo, input, load.5);
        self.rams[6].clock(lo, input, load.6);
        self.rams[7].clock(lo, input, load.7);
    }
}

pub struct RAM16K { rams: [RAM4K; 4] }
impl RAM16K {
    pub fn new() -> Self {
        Self { rams: [ RAM4K::new(), RAM4K::new(), RAM4K::new(), RAM4K::new(), ]}
    }
    pub fn out(&self, address: [bool; 14]) -> Word {
        let lo = [
            address[0], address[ 1], address[ 2],
            address[3], address[ 4], address[ 5],
            address[6], address[ 7], address[ 8],
            address[9], address[10], address[11] ];
        let hi = [address[12], address[13]];
        mux4way16(
            self.rams[0].out(lo), self.rams[1].out(lo), self.rams[2].out(lo), self.rams[3].out(lo),
            hi)
    }
    pub fn clock(&mut self, address: [bool; 14], input: Word, load: bool) {
        let lo = [
            address[0], address[ 1], address[ 2],
            address[3], address[ 4], address[ 5],
            address[6], address[ 7], address[ 8],
            address[9], address[10], address[11] ];
        let hi = [address[12], address[13]];
        let load = dmux4way(load, hi);
        self.rams[0].clock(lo, input, load.0);
        self.rams[1].clock(lo, input, load.1);
        self.rams[2].clock(lo, input, load.2);
        self.rams[3].clock(lo, input, load.3);
    }
}

pub struct Counter { register: Register }
impl Counter {
    pub fn new() -> Self { Self { register: Register::new() } }
    pub fn out(&self) -> Word {
        self.register.out()
    }
    pub fn clock(&mut self, input: Word, inc: bool, load: bool, reset: bool) {
        self.register.clock(
            mux16(mux16(inc16(self.register.out()), input, load), [false; 16], reset),
            or(inc, or(load, reset)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use debug::*;

    #[test]
    fn test_bit() {
        let mut bit = BitRegister::new();
        let inputs = [
            (false, false),
            (true, false),
            (true, false),
            (false, true),
            (true, true),
            (false, false),
            (false, true),
        ];
        let mut previous = bit.out();
        for &(input, load) in &inputs {
            bit.clock(input, load);
            assert_eq!(bit.out(), if load { input } else { previous });
            previous = bit.out();
        }
    }

    #[test]
    fn test_register() {
        let mut register = Register::new();
        let inputs = [
            (139, false),
            (972, true),
            (742, true),
            (243, false),
            (64, false)
        ];
        let mut previous = register.out();
        for &(input, load) in &inputs {
            register.clock(int2word(input), load);
            assert_eq!(word2int(register.out()), if load { input } else { word2int(previous) });
            previous = register.out();
        }
    }

    #[test]
    fn test_RAM8() {
        let mut ram = RAM8::new();
        let values = [72, 45, 29, 836, 4582, 279];
        for &a0 in &[false, true] {
        for &a1 in &[false, true] {
        for &a2 in &[false, true] {
            let address = [a0, a1, a2];
            assert_eq!(word2int(ram.out(address)), 0);
            for &x in &values {
                ram.clock(address, int2word(x), true);
                assert_eq!(word2int(ram.out(address)), x);
            }
        } } }
    }

    #[test]
    fn test_RAM64() {
        let mut ram = RAM64::new();
        let values = [72, 45, 29, 836, 4582, 279];
        for &a0 in &[false, true] {
        for &a1 in &[false, true] {
        for &a2 in &[false, true] {
        for &a3 in &[false, true] {
        for &a4 in &[false, true] {
        for &a5 in &[false, true] {
            let address = [a0, a1, a2, a3, a4, a5];
            assert_eq!(word2int(ram.out(address)), 0);
            for &x in &values {
                ram.clock(address, int2word(x), true);
                assert_eq!(word2int(ram.out(address)), x);
            }
        } } } } } }
    }

    #[test]
    fn test_RAM512() {
        let mut ram = RAM512::new();
        let values = [72, 45, 29, 836, 4582, 279];
        for i in 0 .. 9 {
            let mut address = [false; 9];
            address[i] = true;
            assert_eq!(word2int(ram.out(address)), 0);
            for &x in &values {
                ram.clock(address, int2word(x), true);
                assert_eq!(word2int(ram.out(address)), x);
            }
        }
    }

    #[test]
    fn test_RAM16K() {
        let mut ram = RAM16K::new();
        let values = [72, 45, 29];
        for i in 9 .. 14 {
            let mut address = [false; 14];
            address[i] = true;
            assert_eq!(word2int(ram.out(address)), 0);
            for &x in &values {
                ram.clock(address, int2word(x), true);
                assert_eq!(word2int(ram.out(address)), x);
            }
        }
    }

    #[test]
    fn test_counter() {
        let mut counter = Counter::new();
        for i in 0 .. 10 {
            assert_eq!(word2int(counter.out()), i);
            counter.clock(int2word(0), true, false, false);
        }
        assert_eq!(word2int(counter.out()), 10);

        counter.clock(int2word(0), false, false, true);
        assert_eq!(word2int(counter.out()), 0);

        counter.clock(int2word(123), false, true, false);
        assert_eq!(word2int(counter.out()), 123);
    }
}
