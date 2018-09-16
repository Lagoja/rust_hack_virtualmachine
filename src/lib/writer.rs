use lib::parser::Command;
use lib::symbol_table::{SymbolTable, Address};
use lib::tokenizer::TokenType;

#[derive(Debug)]
pub struct AsmWriter {
    pointer: u16,
    symbol_table: SymbolTable,
}

impl AsmWriter {
    pub fn from(symbol_table: SymbolTable) -> AsmWriter {
        AsmWriter {
            pointer: 0,
            symbol_table,
        }
    }

    pub fn write_command(&self, command: Command) -> Result<String, &'static str> {
        match command {
            Command::Push { segment, index } => self.write_push(segment, index),
            Command::Pop { segment, index } => self.write_pop(segment, index),
            Command::Arithmetic(token_type) => self.write_arithmetic(token_type),
        }
    }

    fn write_push(&self, segment: String, index: u16) -> Result<String, &'static str> {
        let mut stepvec: Vec<String> = vec![];
        let seg: Address;
        if segment == "constant" {
            stepvec = vec![AsmWriter::constant_to_a(index), AsmWriter::push_from_a()];
        } else {
            seg = match self.symbol_table.get_address(&segment) {
                Some(address) => *address,
                None => return Err("Invalid segment provided"),
            };
            match seg {
                Address::Relative(addr) => stepvec = vec![
                AsmWriter::value_from_segment_to_a(addr, index),
                AsmWriter::push_from_a(),
            ],
                Address::Absolute(addr) => stepvec = vec![
                    String::from(format!("@{}\nA=M\n", addr + index)),
                    AsmWriter::push_from_a(),
                ]
            }
;
        }
        Ok(stepvec.join(""))
    }

    fn write_pop(&self, segment: String, index: u16) -> Result<String, &'static str> {
        let mut stepvec: Vec<String> = vec![];
        let seg: Address;
        if segment == "constant" {
            return Err("Cannot pop to constant");
        } else {
            seg = match self.symbol_table.get_address(&segment) {
                Some(address) => *address,
                None => return Err("Invalid segment provided"),
            };
            match seg {
                Address::Relative(addr) => stepvec = vec![
                AsmWriter::save_segment_addr_to_r13(addr, index),
                AsmWriter::write_pop_to_d(),
                AsmWriter::save_d_to_r13_segment_address(),
            ],
                Address::Absolute(addr) => stepvec = vec![
                    AsmWriter::write_pop_to_d(),
                    String::from(format!("@{}\nM=D\n", addr + index))
                ]
            }

        }
        Ok(stepvec.join(""))
    }

    pub fn write_arithmetic(&self, token_type: TokenType) -> Result<String, &'static str> {
        match token_type {
            TokenType::Add => Ok(AsmWriter::add()),
            TokenType::Subtract => Ok(AsmWriter::subtract()),
            TokenType::And => Ok(AsmWriter::and()),
            TokenType::Or => Ok(AsmWriter::or()),
            TokenType::Negate => Ok(AsmWriter::negate()),
            _ => Err("Invalid arithmetic command"),
        }
    }

    fn get_operands() -> String {
        let stepvec = vec![
            AsmWriter::write_pop_to_d(),
            AsmWriter::peek_next_value(),
        ];
        stepvec.join("")
    }

    fn add() -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&format!("D=D+M\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn and() -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&format!("D=D&M\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn or() -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&format!("D=D|M\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn subtract() -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&format!("D=M-D\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn negate() -> String {
        let mut out = AsmWriter::write_pop_to_d();
        out.push_str(&format!("D=-D"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn value_from_segment_to_a(segment: &str, index: u16) -> String {
        //Puts the value in A
        format!("@{}\nD=M\n@{}\nA=D+A\nA=M\n", segment, index)
    }

    fn constant_to_a(index: u16) -> String {
        //Puts a constant value in A
        format!("@{}\n", index)
    }

    fn save_segment_addr_to_r13(segment: &str, index: u16) -> String {
        //Takes an indexed segment address and stores it in R13
        format!("@{}\nD=M\n@{}\nD=D+A\n@R13\nM=D\n", segment, index)
    }

    fn save_d_to_r13_segment_address() -> String {
        //Assumes a value has been popped to D
        String::from("@R13\nA=M\nM=D\n")
    }

    fn push_from_a() -> String {
        //Assumes that the pushed value is in A
        String::from("D=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n")
    }

    fn push_from_d() -> String {
        //Assumes that the pushed value is in D
        String::from("@SP\nA=M\nM=D\n@SP\nM=M+1\n")
    }

    fn write_pop_to_d() -> String {
        //Puts the value in D
        String::from("@SP\nAM=M-1\nD=M\nM=0\n")
    }

    fn peek_next_value() -> String {
        String::from("@SP\nAM=M-1\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_segment_addr() {
        assert_eq!(
            AsmWriter::save_segment_addr_to_r13("LCL", 2),
            String::from("@LCL\nD=M\n@2\nD=D+A\n@R13\nM=D\n")
        );
    }

    #[test]
    fn test_add() {
        let mut st = SymbolTable::new();
        st.load_starting_table();
        let writer = AsmWriter::from(st);
        let out = writer.write_command(Command::Arithmetic(TokenType::Add));
        assert_eq!(
            out.unwrap(),
            String::from(
                "@SP
AM=M-1
D=M
M=0
@SP
AM=M-1
D=D+M
@SP
A=M
M=D
@SP
M=M+1
"
            )
        );
    }

    #[test]
    fn test_add_writer() {
        assert_eq!(
            AsmWriter::add(),
            String::from(
                "@SP
AM=M-1
D=M
M=0
@SP
AM=M-1
D=D+M
@SP
A=M
M=D
@SP
M=M+1
"
            )
        );
    }
}
