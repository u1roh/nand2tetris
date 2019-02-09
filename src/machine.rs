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
        let (load_ram, load_not_ram) = dmux(load, address[14]);
        let (_, load_screen) = dmux(load_not_ram, address[13]);
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
        self.data_memory.clock(self.cpu.addressM(), cpu_out.outM, cpu_out.writeM);
        self.cpu.clock(cpu_input);
    }
    pub fn read_memory(&self, address: i16) -> i16 {
        debug::word2int(self.data_memory.out(debug::int2word(address)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asm::*;

    fn run_machine(asm: &[Instruction], address: i16) -> i16 {
        let bin = asm.iter().map(|inst| inst.encode()).collect::<Vec<_>>();
        let mut machine = Machine::new(&bin);
        for _ in 0 .. asm.len() {
            machine.clock(false);
        }
        machine.read_memory(address)
    }

    #[test]
    fn test_set_value_to_memory() {
        let address = 0b0000000000010000;
        let asm = [
            AInstruction(address),
            CInstruction(Computation::One, dest::M, Jump::Null)
        ];
        assert_eq!(run_machine(&asm, address), 1);
    }
}