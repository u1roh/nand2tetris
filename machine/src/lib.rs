mod given;
mod gate;
mod adder;
mod alu;
mod ram;
mod cpu;
mod blackbox;
pub mod inst;

use given::*;
use gate::*;
use ram::*;
use cpu::*;
use blackbox::*;

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
        let [load_ram, load_not_ram] = dmux(load, address[14]);
        let [load_screen, _] = dmux(load_not_ram, address[13]);
        self.ram.clock(ram_addr, input, load_ram);
        self.screen.clock(screen_addr, input, load_screen);
    }
}

pub struct Machine {
    instruction_memory: Box<ROM32K>,
    data_memory: Box<Memory>,
    cpu: Cpu
}

impl Machine {
    pub fn new(instructions: &[i16]) -> Self {
        Self{
            instruction_memory: Box::new(ROM32K::new(instructions)),
            data_memory: Box::new(Memory::new()),
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
    pub fn next_instruction(&self) -> i16 {
        debug::word2int(self.instruction_memory.out(self.cpu.pc()))
    }
    pub fn is_terminated(&self) -> bool {
        self.instruction_memory.is_terminated(self.cpu.pc())
    }
    pub fn read_memory(&self, address: i16) -> i16 {
        debug::word2int(self.data_memory.out(debug::int2word(address)))
    }
    pub fn screen_image(&self) -> &[i16; 32 * 256] {
        self.data_memory.screen.raw_image()
    }
    pub fn keyboard_input(&mut self, key: i16) {
        self.data_memory.keyboard.input(key);
    }
    pub fn print_status_header(&self) {
        println!("{:4}: [{:4}] {:5}, [{:4}] {:5}, [{:4}] {:5}, [{:4}] {:5}, [{:4}] {:5}",
            "PC",
            "SP", "*SP",
            "LCL", "*LCL",
            "ARG", "*ARG",
            "THIS", "*THIS",
            "THAT", "*THAT");
    }
    pub fn print_status(&self) {
        let (sp, lcl, arg, this, that) = (self.read_memory(0), self.read_memory(1), self.read_memory(2), self.read_memory(3), self.read_memory(4));
        print!("{:4}: [{:>04x}] {:>5}, [{:04x}] {:>5}, [{:04x}] {:>5}, [{:04x}] {:>5}, [{:04x}] {:>5}\r",
            debug::word2int(self.cpu.pc()),
            sp, self.read_memory(sp),
            lcl, self.read_memory(lcl),
            arg, self.read_memory(arg),
            this, self.read_memory(this),
            that, self.read_memory(that));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inst::*;

    fn run_machine(asm: &[Instruction], nclock: usize, address: i16) -> i16 {
        let bin = asm.iter().map(|inst| inst.encode()).collect::<Vec<_>>();
        let mut machine = Machine::new(&bin);
        for _ in 0 .. nclock {
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
        assert_eq!(run_machine(&asm, asm.len(), address), 1);
    }

    #[test]
    fn test_123_plus_456() {
        use Computation::*;
        let sum = 0b10000;  // address of symbol 'sum'
        let asm = [
            /* @123     */  AInstruction(123),
            /* D=A      */  CInstruction(X(false), dest::D, Jump::Null),
            /* @sum     */  AInstruction(sum),
            /* M=D      */  CInstruction(D, dest::M, Jump::Null),
            /* @456     */  AInstruction(456),
            /* D=A      */  CInstruction(X(false), dest::D, Jump::Null),
            /* @sum     */  AInstruction(sum),
            /* M=D+M    */  CInstruction(DPlusX(true), dest::M, Jump::Null),
        ];
        assert_eq!(run_machine(&asm, asm.len(), sum), 123 + 456);
    }

    #[test]
    fn test_sum_1_to_10() {
        use Computation::*;
        let i   = 0b10000;  // address of symbol 'i'
        let sum = 0b10001;  // address of symbol 'sum'
        let asm = [
            /* @i       */  AInstruction(i),
            /* M=1      */  CInstruction(One, dest::M, Jump::Null),
            /* @sum     */  AInstruction(sum),
            /* M=0      */  CInstruction(Zero, dest::M, Jump::Null),
            /* (LOOP)   */  // address is 4
            /* @i       */  AInstruction(i),
            /* D=M      */  CInstruction(X(true), dest::D, Jump::Null),
            /* @10      */  AInstruction(10),
            /* D=D-A    */  CInstruction(DMinusX(false), dest::D, Jump::Null),
            /* @END     */  AInstruction(18),
            /* D;JGT    */  CInstruction(D, 0, Jump::JGT),
            /* @i       */  AInstruction(i),
            /* D=M      */  CInstruction(X(true), dest::D, Jump::Null),
            /* @sum     */  AInstruction(sum),
            /* M=D+M    */  CInstruction(DPlusX(true), dest::M, Jump::Null),
            /* @i       */  AInstruction(i),
            /* M=M+1    */  CInstruction(XPlusOne(true), dest::M, Jump::Null),
            /* @LOOP    */  AInstruction(4),
            /* 0;JMP    */  CInstruction(Zero, 0, Jump::JMP),
            /* (END)    */  // address = 18
            /* @END     */  AInstruction(18),
            /* 0;JMP    */  CInstruction(Zero, 0, Jump::JMP),
        ];
        assert_eq!(run_machine(&asm, asm.len() * 10, sum), 10 * (10 + 1) / 2);
    }
}