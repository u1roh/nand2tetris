use crate::inst::*;

#[derive(Debug, PartialEq, Eq)]
pub enum AsmError<'a> {
    InvalidAInstruction(&'a str),
    ComputationFieldEmpty,
    InvalidComputation(&'a str),
    InvalidDestination(&'a str),
    InvalidJump(&'a str),
    NotImplemented,
    UnknownError(String)
}

pub use AsmError::*;
pub type Result<'a, T> = std::result::Result<T, AsmError<'a>>;

// translate assembly program into machine language
pub fn asm(program: &str) -> Result<Vec<i16>> {
    let commands = program.split("\n")
        .map(|line| if let Some(i) = line.find("//") { &line[..i] } else { line })
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(line_to_command)
        .collect::<Result<Vec<_>>>()?;
    commands.iter().filter_map(|command| {
        match command {
            Command::AValue(a) => Some(Ok(AInstruction(*a))),
            Command::Variable(_) => None,
            Command::Label(_) => None,
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
                    Some("MD")  => dest::M | dest::A,
                    Some("AM")  => dest::A | dest::M,
                    Some("AD")  => dest::A | dest::D,
                    Some("AMD") => dest::A | dest::D | dest::M,
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

#[derive(PartialEq, Eq, Debug)]
enum Command<'a> {
    AValue(i16),
    Variable(&'a str),
    Label(&'a str),
    Computation{ comp: &'a str, dest: Option<&'a str>, jump: Option<&'a str> }
}

fn line_to_command(line: &str) -> Result<Command> {
    assert!(!line.is_empty());
    if let Some(i) = line.find('@') {
        let a = &line[i+1 ..];
        if a.len() == 0 { Err(InvalidAInstruction(line)) } else {
            match line[i+1..].parse::<i16>() {
                Ok(a) => Ok(Command::AValue(a)),
                _ => Ok(Command::Variable(&line[i+1..]))
            }
        }
    }
    else {
        let (dest, comp, jump) = match (line.find('='), line.find(';')) {
            (Some(i), Some(j)) => (Some(line[..i].trim()), line[i+1..j].trim(), Some(line[j+1..].trim())),
            (Some(i), None) => (Some(line[..i].trim()), line[i+1..].trim(), None),
            (None, Some(j)) => (None, line[..j].trim(), Some(line[j+1..].trim())),
            (None, None) => (None, line, None)
        };
        if comp.is_empty() { Err(ComputationFieldEmpty) } else {
            Ok(Command::Computation{ comp: comp, dest: dest, jump: jump })
        }
    }
}

fn to_lines(program: &str) -> Vec<&str> {
    program.split("\n").collect::<Vec<_>>()
}

fn remove_comment(line: &str) -> &str {
    if let Some(i) = line.find("//") { &line[..i] } else { line }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    
    #[test]
    fn test_to_lines() {
        let program = "abc\ndef\nghi";
        let lines = to_lines(program);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "abc");
        assert_eq!(lines[1], "def");
        assert_eq!(lines[2], "ghi");
    }

    #[test]
    fn test_remove_comment() {
        let line = "hoge/piyo//comment";
        assert_eq!(remove_comment(line), "hoge/piyo");
    }

    #[test]
    fn test_trim_whitespace() {
        let line = "  hoge  piyo      ";
        assert_eq!(line.trim(), "hoge  piyo");
    }

    #[test]
    fn test_line_to_command() {
        assert_eq!(line_to_command("@123"), Ok(Command::AValue(123)));
        assert_eq!(line_to_command("@foo"), Ok(Command::Variable("foo")));
        assert!(line_to_command("@").is_err());
        assert_eq!(line_to_command("D"), Ok(Command::Computation{ comp: "D", dest: None, jump: None }));
        assert_eq!(line_to_command("D=A"), Ok(Command::Computation{ comp: "A", dest: Some("D"), jump: None }));
        assert_eq!(line_to_command("M;JGT"), Ok(Command::Computation{ comp: "M", dest: None, jump: Some("JGT") }));
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
    fn test_add() {
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
}