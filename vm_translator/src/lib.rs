mod writer;
use writer::*;

#[derive(Debug)]
enum Command<'a> {
    // arithmetic commands
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),

    // logical commands
    LogicalOp(Condition), // eq, gt, lt

    // memory access commands
    Push{ segment: &'a str, index: i16 },
    Pop { segment: &'a str, index: i16 },

    // program flow commands
    Label(&'a str),
    Goto(&'a str),
    IfGoto(&'a str),

    // function calling commands
    Function{ funcname: &'a str, nlocals: i16 },
    Call{ funcname: &'a str, nargs: i16 },
    Return
}

fn line_to_command(line: &str) -> Command {
    assert!(!line.is_empty());
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    assert!(tokens.len() != 0);
    match tokens[0] {
        "neg" | "not" => {
            let op = match tokens[0] {
                "neg" => UnaryOp::Neg,
                "not" => UnaryOp::Not,
                _ => panic!("unknown unary operation: {}", tokens[0])
            };
            Command::UnaryOp(op)
        },
        "add" | "sub" | "and" | "or" => {
            let op = match tokens[0] {
                "add" => BinaryOp::Add,
                "sub" => BinaryOp::Sub,
                "and" => BinaryOp::And,
                "or"  => BinaryOp::Or,
                _ => panic!("unknown binary operation: {}", tokens[0])
            };
            Command::BinaryOp(op)
        },
        "eq" | "gt" | "lt" => {
            let cond = match tokens[0] {
                "eq" => Condition::Eq,
                "gt" => Condition::Gt,
                "lt" => Condition::Lt,
                _ => panic!("unknown logical operation: {}", tokens[0])
            };
            Command::LogicalOp(cond)
        },
        "push"      => Command::Push{ segment: tokens[1], index: tokens[2].parse::<i16>().unwrap() },
        "pop"       => Command::Pop{ segment: tokens[1], index: tokens[2].parse::<i16>().unwrap() },
        "label"     => Command::Label(tokens[1]),
        "if-goto"   => Command::IfGoto(tokens[1]),
        "goto"      => Command::Goto(tokens[1]),
        "function"  => Command::Function{ funcname: tokens[1], nlocals: tokens[2].parse::<i16>().unwrap() },
        "call"      => Command::Call{ funcname: tokens[1], nargs: tokens[2].parse::<i16>().unwrap() },
        "return"    => Command::Return,
        _ => panic!("unknown command: {}", line)
    }
}

fn translate_command(out: &mut AsmWriter, command: Command) {
    //out.write(&format!("// <{:?}>", command));
    match command {
        Command::UnaryOp(op) => {
            out.unary_op(op);
            out.push();
        },
        Command::BinaryOp(op) => {
            out.binary_op(op);
            out.push();
        },
        Command::LogicalOp(cond) => {
            out.logical_op(cond);
            out.push();
        },
        Command::Push{ segment, index } => {
            out.push_segment(segment, index);
        },
        Command::Pop{ segment, index } => {
            out.pop_segment(segment, index);
        },
        Command::Label(symbol) => {
            out.label(symbol);
        },
        Command::Goto(symbol) => {
            out.goto(symbol);
        },
        Command::IfGoto(symbol) => {
            out.pop();
            out.if_goto(symbol);
        },
        Command::Function{ funcname, nlocals } => {
            out.func_begin(funcname, nlocals);
        },
        Command::Return => {
            out.func_return();
        },
        Command::Call{ funcname, nargs } => {
            out.func_call(funcname, nargs);
        }
    }
    //out.write(&format!("// </{:?}>", command));
}

fn translate_vm_source(out: &mut AsmWriter, source: &str) {
    let commands = source.split("\n")
        .map(|line| if let Some(i) = line.find("//") { &line[..i] } else { line })  // remove comment
        .map(|line| line.trim())  // remove white spaces of head and tail
        .filter(|line| !line.is_empty())    // filter empty line
        .map(line_to_command)
        .collect::<Vec<_>>();
    for command in commands {
        translate_command(out, command);
    }
}

pub fn compile(out: &mut std::fmt::Write, source_filename: &str, source: &str) {
    let mut out = AsmWriter::new(out, source_filename);
    let _ = out.call_sys_init();
    translate_vm_source(&mut out, source);
}

#[cfg(test)]
mod tests {
    extern crate machine;
    extern crate asm;
    use machine::*;
    use super::*;

    fn run_machine(asm_source: &str, max_clock: usize) -> i16 {
        println!("{}", asm_source);
        let bin = asm::asm(&asm_source).unwrap();
        let mut machine = Machine::new(&bin);
        let mut nclock = 0;
        machine.print_status_header();
        while !machine.is_terminated() {
            //println!("{}", inst::Instruction::decode(machine.next_instruction()));
            machine.clock(false);
            //println!("SP = {}, STACK TOP = {}", machine.read_memory(0), machine.read_memory(machine.read_memory(0) - 1));
            machine.print_status();
            println!();
            nclock += 1;
            assert!(nclock < max_clock);
        }
        machine.read_memory(machine.read_memory(0) - 1) // top of the stack
    }

    fn test(expected: i16, vm_source: &str) {
        let mut asm_source = String::new();
        {
            let mut out = AsmWriter::new(&mut asm_source, "test_file");
            out.set_ram("SP", 256);
            out.set_ram("LCL", 300);
            out.set_ram("ARG", 400);
            out.set_ram("THIS", 3000);
            out.set_ram("THAT", 3010);
            translate_vm_source(&mut out, vm_source);
        }
        let max_clock = 1000;
        assert_eq!(expected, run_machine(&asm_source, max_clock));
    }

    fn test2(expected: i16, vm_source: &str) {
        let mut asm_source = String::new();
        compile(&mut asm_source, "test_file", &vm_source);
        let max_clock = 1000;
        assert_eq!(expected, run_machine(&asm_source, max_clock));
    }

    #[test]
    fn add() {
        test(15, "
        push constant 7
        push constant 8
        add
        ");
    }

    #[test]
    fn sub() {
        test(327 - 193,"
        push constant 327
        push constant 193
        sub
        ");
    }

    #[test]
    fn and() {
        test(826 & 294, "
        push constant 826
        push constant 294
        and
        ");
    }

    #[test]
    fn or() {
        test(826 | 294, "
        push constant 826
        push constant 294
        or
        ");
    }

    #[test]
    fn eq() {
        test(-1, "
        push constant 123
        push constant 123
        eq
        ");
        test(0, "
        push constant 100
        push constant 200
        eq
        ");
    }

    #[test]
    fn gt() {
        test(-1, "
        push constant 826
        push constant 294
        gt
        ");
        test(0, "
        push constant 1
        push constant 2
        gt
        ");
        test(0, "
        push constant 3
        push constant 3
        gt
        ");
    }

    #[test]
    fn lt() {
        test(0, "
        push constant 826
        push constant 294
        lt
        ");
        test(-1, "
        push constant 1
        push constant 2
        lt
        ");
        test(0, "
        push constant 3
        push constant 3
        lt
        ");
    }

    #[test]
    fn neg() {
        test(-123, "
        push constant 123
        neg
        ");
        test(321, "
        push constant 321
        neg
        neg
        ");
    }

    #[test]
    fn not() {
        test(!0, "
        push constant 0
        not
        ");
        test(!123, "
        push constant 123
        not
        ");
    }

    #[test]
    fn arithmetic() {
        test(1 - (2 + 3), "
        push constant 1
        push constant 2
        push constant 3
        add
        sub
        ");
        test((1 + 2) - 3, "
        push constant 1
        push constant 2
        add
        push constant 3
        sub
        ");
        test((1 + 3) - (5 + 7), "
        push constant 1
        push constant 3
        add
        push constant 5
        push constant 7
        add
        sub
        ");
    }

    #[test]
    fn pointer_segment() {
        test(123, "
        push constant 123
        pop pointer 0
        push pointer 0
        ");
        test(456, "
        push constant 456
        pop pointer 1
        push pointer 1
        ");
    }

    #[test]
    fn this_segment() {
        test(789, "
        push constant 789
        pop this 0
        push this 0
        ");
        test(111, "
        push constant 111
        pop this 1
        push this 1
        ");
        test(789, "
        push constant 789
        pop this 1
        push pointer 0
        push constant 1
        add
        pop pointer 0
        push pointer 0
        push this 0
        ");
    }

    #[test]
    fn static_segment() {
        test(123, "
        push constant 123
        pop static 3
        push static 3
        ");
    }

    #[test]
    fn conditional() {
        let vm = "
        if-goto L1
        push constant 123
        goto L2
        label L1
        push constant 456
        label L2
        push constant 1
        add
        ";
        test(124, &format!("
        push constant 0
        {}
        ", vm));
        test(457, &format!("
        push constant 0
        not
        {}
        ", vm));
    }

    #[test]
    fn sum() {
        test(1 + 2 + 3 + 4 + 5, "
        // argument[0] = 5
        push constant 5
        pop argument 0

        // Computes the sum 1 + 2 + ... + argument[0] and pushes the 
        // result onto the stack.
        push constant 0    
        pop local 0         // initializes sum = 0
        label LOOP_START
        push argument 0    
        push local 0
        add
        pop local 0	        // sum = sum + counter
        push argument 0
        push constant 1
        sub
        pop argument 0      // counter--
        push argument 0
        if-goto LOOP_START  // If counter > 0, goto LOOP_START
        push local 0
        ");
    }

    #[test]
    fn sys_init() {
        test2(1234 + 5678, "
        function Sys.init 0
            push constant 1234
            push constant 5678
            add
            return
        ");
    }

    #[test]
    fn simple_function() {
        test2(3, "
        function test_add 0
            push argument 0
            push argument 1
            add
            return
        function Sys.init 0
            push constant 1
            push constant 2
            call test_add 2
        ");
    }
}