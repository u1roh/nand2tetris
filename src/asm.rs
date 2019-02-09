
// translate assembly program into machine language
pub fn asm(program: &str) -> Vec<i16> {
    Vec::new()
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
}