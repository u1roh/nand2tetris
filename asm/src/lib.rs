extern crate machine;
use machine::inst::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum AsmError<'a> {
    InvalidAInstruction(&'a str),
    EmptyComputation,
    EmptyLabel,
    InvalidLine(&'a str),
    InvalidComputation(&'a str),
    InvalidDestination(&'a str),
    InvalidJump(&'a str),
}

pub use AsmError::*;
pub type Result<'a, T> = std::result::Result<T, AsmError<'a>>;

#[derive(PartialEq, Eq, Debug)]
enum Command<'a> {
    AValue(i16),
    ASymbol(&'a str),
    Label(&'a str),
    Computation{ comp: &'a str, dest: Option<&'a str>, jump: Option<&'a str> }
}

fn line_to_command(line: &str) -> Result<Command> {
    assert!(!line.is_empty());
    if let Some(i) = line.find('@') {
        if i != 0 { return Err(InvalidLine(line)); }
        let a = &line[i+1 ..];
        if a.len() == 0 { Err(InvalidAInstruction(line)) } else {
            match line[i+1..].parse::<i16>() {
                Ok(a) => Ok(Command::AValue(a)),
                _ => Ok(Command::ASymbol(&line[i+1..]))
            }
        }
    }
    else if let Some(i) = line.find("(") {
        let j = line.find(")").ok_or(InvalidLine(line))?;
        if i > j { return Err(InvalidLine(line)) }
        let label = line[i+1 .. j].trim();
        if label.is_empty() { Err(EmptyLabel) } else { Ok(Command::Label(label)) }
    }
    else {
        let (dest, comp, jump) = match (line.find('='), line.find(';')) {
            (Some(i), Some(j)) => (Some(line[..i].trim()), line[i+1..j].trim(), Some(line[j+1..].trim())),
            (Some(i), None) => (Some(line[..i].trim()), line[i+1..].trim(), None),
            (None, Some(j)) => (None, line[..j].trim(), Some(line[j+1..].trim())),
            (None, None) => (None, line, None)
        };
        if comp.is_empty() { Err(EmptyComputation) } else {
            Ok(Command::Computation{ comp: comp, dest: dest, jump: jump })
        }
    }
}

// translate assembly program into machine language
pub fn asm(program: &str) -> Result<Vec<i16>> {
    let commands = program.split("\n")
        .map(|line| if let Some(i) = line.find("//") { &line[..i] } else { line })  // remove comment
        .map(|line| line.trim())  // remove white spaces of head and tail
        .filter(|line| !line.is_empty())    // filter empty line
        .map(line_to_command)
        .collect::<Result<Vec<_>>>()?;

    // predefined symbols
    let mut symbols = [
            ("SP",      0),
            ("LCL",     1),
            ("ARG",     2),
            ("THIS",    3),
            ("THAT",    4),
            ("SCREEN",  0x4000),
            ("KBD",     0x6000)
        ].iter().cloned().collect::<HashMap<&str, i16>>();
    let r = ["R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7", "R8", "R9", "R10", "R11", "R12", "R13", "R14", "R15"];
    for i in 0 .. 16 {
        symbols.insert(r[i], i as i16);
    }

    // 1st pass: add labels to symbol table
    let mut rom_address = 0;
    for command in &commands {
        match command {
            Command::Label(label) => { symbols.insert(label, rom_address); },
            _ => rom_address += 1
        }
    }

    // 2nd pass:
    let mut ram_address = 0x10;
    commands.iter().filter_map(|command| {
        match command {
            Command::Label(_) => None,  // skip label, since label is pseudo command.
            Command::AValue(a) => Some(Ok(AInstruction(*a))),
            Command::ASymbol(a) => {
                let a = *symbols.entry(*a).or_insert_with(|| {
                    ram_address += 1;
                    ram_address - 1
                });
                Some(Ok(AInstruction(a)))
            },
            Command::Computation{ comp, dest, jump } => {
                use Computation::*;
                let comp = match *comp {
                    "0"   => Zero, "1" => One, "-1" => MinusOne,
                    "D"   => D, "A" => X(false), "M" => X(true),
                    "!D"  => NotD,   "!A" => NotX(false),   "!M" => NotX(true),
                    "-D"  => MinusD, "-A" => MinusX(false), "-M" => MinusX(true),
                    "D+1" => DPlusOne,  "A+1" => XPlusOne(false),  "M+1" => XPlusOne(true),
                    "D-1" => DMinusOne, "A-1" => XMinusOne(false), "M-1" => XMinusOne(true),
                    "D+A" => DPlusX(false),  "D+M" => DPlusX(true),
                    "D-A" => DMinusX(false), "D-M" => DMinusX(true),
                    "A-D" => XMinusD(false), "M-D" => XMinusD(true),
                    "D&A" => DAndX(false),   "D&M" => DAndX(true),
                    "D|A" => DOrX(false),    "D|M" => DOrX(true),
                    _ => return Some(Err(InvalidComputation(comp))),
                };
                let dest = match *dest {
                    None => 0,
                    Some("A")   => dest::A,
                    Some("D")   => dest::D,
                    Some("M")   => dest::M,
                    Some("MD")  => dest::M | dest::D,
                    Some("AM")  => dest::A | dest::M,
                    Some("AD")  => dest::A | dest::D,
                    Some("AMD") => dest::A | dest::M | dest::D,
                    Some(dest) => return Some(Err(InvalidDestination(dest))),
                };
                let jump = match *jump {
                    None => Jump::Null,
                    Some("JGT") => Jump::JGT,
                    Some("JEQ") => Jump::JEQ,
                    Some("JGE") => Jump::JGE,
                    Some("JLT") => Jump::JLT,
                    Some("JNE") => Jump::JNE,
                    Some("JLE") => Jump::JLE,
                    Some("JMP") => Jump::JMP,
                    Some(jump) => return Some(Err(InvalidJump(jump)))
                };
                Some(Ok(CInstruction(comp, dest, jump)))
            }
        }
    }).map(|inst| inst.map(|inst| inst.encode())).collect::<Result<Vec<_>>>()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    
    #[test]
    fn test_to_lines() {
        let lines = "abc\ndef\nghi".split("\n").collect::<Vec<_>>();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "abc");
        assert_eq!(lines[1], "def");
        assert_eq!(lines[2], "ghi");
    }

    #[test]
    fn test_remove_comment() {
        let line = "hoge/piyo//comment";
        let comment_removed = if let Some(i) = line.find("//") { &line[..i] } else { line };
        assert_eq!(comment_removed, "hoge/piyo");
    }

    #[test]
    fn test_trim_whitespace() {
        let line = "  hoge  piyo      ";
        assert_eq!(line.trim(), "hoge  piyo");
    }

    #[test]
    fn test_line_to_command() {
        assert_eq!(line_to_command("@123"), Ok(Command::AValue(123)));
        assert_eq!(line_to_command("@foo"), Ok(Command::ASymbol("foo")));
        assert!(line_to_command("@").is_err());
        assert_eq!(line_to_command("D"), Ok(Command::Computation{ comp: "D", dest: None, jump: None }));
        assert_eq!(line_to_command("D=A"), Ok(Command::Computation{ comp: "A", dest: Some("D"), jump: None }));
        assert_eq!(line_to_command("M;JGT"), Ok(Command::Computation{ comp: "M", dest: None, jump: Some("JGT") }));
        assert_eq!(line_to_command("(BUZZ)"), Ok(Command::Label("BUZZ")));
    }

    fn run_machine(program: &str, nclock: usize, address: i16) -> i16 {
        let bin = asm(program).unwrap();
        let mut machine = Machine::new(&bin);
        for _ in 0 .. nclock {
            machine.clock(false);
        }
        machine.read_memory(address)
    }

    #[test]
    fn test_2_plus_3() {
        let program = "
// This file is part of www.nand2tetris.org
// and the book \"The Elements of Computing Systems\"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/add/Add.asm

// Computes R0 = 2 + 3  (R0 refers to RAM[0])

@2
D=A
@3
D=D+A
@0
M=D
";
        assert_eq!(run_machine(program, 6, 0), 2 + 3);
    }

    #[test]
    fn test_max() {
        let program = "
// This file is part of www.nand2tetris.org
// and the book \"The Elements of Computing Systems\"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/max/Max.asm

// Computes R2 = max(R0, R1)  (R0,R1,R2 refer to RAM[0],RAM[1],RAM[2])

   @R0
   D=M              // D = first number
   @R1
   D=D-M            // D = first number - second number
   @OUTPUT_FIRST
   D;JGT            // if D>0 (first is greater) goto output_first
   @R1
   D=M              // D = second number
   @OUTPUT_D
   0;JMP            // goto output_d
(OUTPUT_FIRST)
   @R0             
   D=M              // D = first number
(OUTPUT_D)
   @R2
   M=D              // M[2] = D (greatest number)
(INFINITE_LOOP)
   @INFINITE_LOOP
   0;JMP            // infinite loop
";
        fn set_value(i: usize, value: i16) -> String {
            format!("@{}\nD=A\n@R{}\nM=D\n", value, i)
        }
        let program = set_value(0, 123) + &set_value(1, 456) + program;
        assert_eq!(run_machine(&program, 20, 2), 456);
    }
}