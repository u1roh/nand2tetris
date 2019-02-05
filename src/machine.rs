use crate::given::*;
use crate::gate::*;
use crate::ram::*;
use crate::cpu::*;

struct ROM32K {
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

struct Screen {
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

struct Keyboard {
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


struct Memory {
    ram: RAM16K,
    screen: Screen,
    keyboard: Keyboard
}

impl Memory {
    pub fn new() -> Self {
        Memory{ ram: RAM16K::new(), screen: Screen::new(), keyboard: Keyboard::new() }
    }
    pub fn out(&self, address: Word) -> Word {
        let ram_addr = [
            address[ 0], address[ 1], address[ 2], address[ 3],
            address[ 4], address[ 5], address[ 6], address[ 7],
            address[ 8], address[ 9], address[10], address[11],
            address[12], address[13]
        ];
        let screen_addr = [
            address[ 0], address[ 1], address[ 2], address[ 3],
            address[ 4], address[ 5], address[ 6], address[ 7],
            address[ 8], address[ 9], address[10], address[11],
            address[12],
        ];
        mux16(
            self.ram.out(ram_addr),
            mux16(self.screen.out(screen_addr), self.keyboard.out(), address[13]),
            address[14])
    }
    pub fn clock(&mut self, address: Word, input: Word, load: bool) {
        let ram_addr = [
            address[ 0], address[ 1], address[ 2], address[ 3],
            address[ 4], address[ 5], address[ 6], address[ 7],
            address[ 8], address[ 9], address[10], address[11],
            address[12], address[13]
        ];
        let screen_addr = [
            address[ 0], address[ 1], address[ 2], address[ 3],
            address[ 4], address[ 5], address[ 6], address[ 7],
            address[ 8], address[ 9], address[10], address[11],
            address[12],
        ];
        let (load_not_ram, load_ram) = dmux(load, address[14]);
        let (load_screen, _) = dmux(load_not_ram, address[13]);
        self.ram.clock(ram_addr, input, load_ram);
        self.screen.clock(screen_addr, input, load_ram);
    }
}

struct Machine {
    instruction_memory: ROM32K,
    data_memory: Memory,
    cpu: Cpu
}

impl Machine {
    pub fn new(instructions: &[i16]) -> Self {
        Self{
            instruction_memory: ROM32K::new(instructions),
            data_memory: Memory::new(),
            cpu: Cpu::new()
        }
    }
    pub fn clock(&mut self, reset: bool) {
        let cpu_input = CpuInput{
            instruction: self.instruction_memory.out(self.cpu.pc()),
            inM: self.data_memory.out(self.cpu.addressM()),
            reset: reset
        };
        let cpu_out = self.cpu.out(cpu_input);
        self.data_memory.clock(cpu_out.addressM, cpu_out.outM, cpu_out.writeM);
        self.cpu.clock(cpu_input);
    }
}