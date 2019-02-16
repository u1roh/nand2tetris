use crate::given::Word;
use crate::given::debug::{int2word, word2int};

pub struct ROM32K {
    data: Box<[i16; 32 * 1024]>
}

impl ROM32K {
    pub fn new(data: &[i16]) -> Self {
        let mut rom = Self{ data: Box::new([0; 32 * 1024]) };
        for i in 0 .. data.len() { rom.data[i] = data[i]; }
        rom
    }
    pub fn out(&self, address: Word) -> Word {
        int2word(self.data[word2int(address) as usize])
    }
}

pub struct Screen {
    data: Box<[i16; 32 * 256]>
}

impl Screen {
    pub fn new() -> Self {
        Self{ data: Box::new([0x0f0f; 32 * 256]) }
    }
    pub fn out(&self, address: [bool; 13]) -> Word {
        let i = {
            let mut a = [false; 16];
            for i in 0 .. address.len() { a[i] = address[i]; }
            word2int(a) as usize
        };
        int2word(self.data[i])
    }
    pub fn clock(&mut self, address: [bool; 13], input: Word, load: bool) {
        if load {
            let i = {
                let mut a = [false; 16];
                for i in 0 .. address.len() { a[i] = address[i]; }
                word2int(a) as usize
            };
            self.data[i] = word2int(input);
        }
    }
    pub fn raw_image(&self) -> &[i16; 32 * 256] {
        &*self.data
    }
}

pub struct Keyboard {
    key: i16
}

impl Keyboard {
    pub fn new() -> Self {
        Self{ key: 0 }
    }
    pub fn out(&self) -> Word {
        int2word(self.key)
    }
}
