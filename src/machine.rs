use crate::given::*;
use crate::gate::*;
use crate::ram::*;
use crate::cpu::*;
use crate::blackbox::*;

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
    pub fn read_memory(&self, address: i16) -> i16 {
        word2int(self.data_memory.out(int2word(address)))
    }
}