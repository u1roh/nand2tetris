use crate::given::{Word, int2word, word2int};

pub struct ROM32K {
    data: [i16; 32 * 1024]
}

impl ROM32K {
    pub fn new(data: &[i16]) -> Self {
        let mut rom = Self{ data: [0; 32 * 1024] };
        rom.data.copy_from_slice(data);
        rom
    }
    pub fn out(&self, address: Word) -> Word {
        int2word(self.data[word2int(address) as usize])
    }
}

pub struct Screen {
    data: [i16; 16 * 1024]
}

impl Screen {
    pub fn new() -> Self {
        Self{ data: [0; 16 * 1024] }
    }
    pub fn out(&self, address: [bool; 13]) -> Word {
        let i = {
            let mut a = [false; 16];
            a.copy_from_slice(&address);
            word2int(a) as usize
        };
        int2word(self.data[i])
    }
    pub fn clock(&mut self, address: [bool; 13], input: Word, load: bool) {
        if load {
            let i = {
                let mut a = [false; 16];
                a.copy_from_slice(&address);
                word2int(a) as usize
            };
            self.data[i] = word2int(input);
        }
    }
    pub fn raw_image(&self) -> &[i16; 16 * 1024] {
        &self.data
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
