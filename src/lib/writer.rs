use lib::parser::Command;
use lib::symbol_table::{Address, SymbolTable};
use lib::tokenizer::TokenType;

#[derive(Debug)]
pub struct AsmWriter {
    line_count: u16,
    branch_count: u16,
    symbol_table: SymbolTable,
}

impl AsmWriter {
    pub fn from(symbol_table: SymbolTable) -> AsmWriter {
        AsmWriter {
            line_count: 0,
            branch_count: 0,
            symbol_table,
        }
    }

    pub fn write_init(&mut self) -> Result<String, &'static str> {
        let stepvec = vec![
            String::from("@256\nD=A\n@SP\nM=D\n"),
            self.write_call(String::from("Sys.init"), 0).unwrap(),
        ];
        Ok(stepvec.join(""))
    }

    pub fn write_command(&mut self, command: Command) -> Result<String, &'static str> {
        let mut outstr = format!("//Command #{}\n", self.line_count);
        let comm = match command {
            Command::Push {
                segment,
                index,
                class_name,
            } => self.write_push(segment, index, class_name)?,
            Command::Pop {
                segment,
                index,
                class_name,
            } => self.write_pop(segment, index, class_name)?,
            Command::Arithmetic(token_type) => self.write_arithmetic(token_type)?,
            Command::If(label) => self.write_if(label)?,
            Command::Goto(label) => self.write_goto(label)?,
            Command::Label(label) => self.write_label(label)?,
            Command::Call { symbol, nargs } => self.write_call(symbol, nargs)?,
            Command::Function { symbol, nvars } => self.write_function(symbol, nvars)?,
            Command::Return => self.write_return()?,
        };
        self.line_count += 1;
        outstr.push_str(&comm);
        Ok(outstr)
    }

    fn write_push(
        &self,
        segment: String,
        index: u16,
        class_name: String,
    ) -> Result<String, &'static str> {
        let stepvec: Vec<String>;
        let seg: Address;
        if segment == "constant" {
            stepvec = vec![AsmWriter::constant_to_a(index), AsmWriter::push_from_a()];
        } else if segment == "static" {
            stepvec = vec![
                String::from(format!("@{}.{}\nA=M\n", class_name, index)),
                AsmWriter::push_from_a(),
            ]
        } else {
            seg = match self.symbol_table.get_address(&segment) {
                Some(address) => *address,
                None => return Err("Invalid segment provided"),
            };
            match seg {
                Address::Relative(addr) => {
                    stepvec = vec![
                        AsmWriter::value_from_segment_to_a(addr, index),
                        AsmWriter::push_from_a(),
                    ]
                }
                Address::Absolute(addr) => {
                    stepvec = vec![
                        String::from(format!("@{}\nA=M\n", addr + index)),
                        AsmWriter::push_from_a(),
                    ]
                }
            };
        }
        Ok(stepvec.join(""))
    }

    fn write_pop(
        &self,
        segment: String,
        index: u16,
        class_name: String,
    ) -> Result<String, &'static str> {
        let stepvec: Vec<String>;
        let seg: Address;
        if segment == "constant" {
            return Err("Cannot pop to constant");
        } else if segment == "static" {
            stepvec = vec![
                AsmWriter::write_pop_to_d(),
                String::from(format!("@{}.{}\nM=D\n", class_name, index)),
            ]
        } else {
            seg = match self.symbol_table.get_address(&segment) {
                Some(address) => *address,
                None => return Err("Invalid segment provided"),
            };
            match seg {
                Address::Relative(addr) => {
                    stepvec = vec![
                        AsmWriter::save_segment_addr_to_r13(addr, index),
                        AsmWriter::write_pop_to_d(),
                        AsmWriter::save_d_to_r13_segment_address(),
                    ]
                }
                Address::Absolute(addr) => {
                    stepvec = vec![
                        AsmWriter::write_pop_to_d(),
                        String::from(format!("@{}\nM=D\n", addr + index)),
                    ]
                }
            }
        }
        Ok(stepvec.join(""))
    }

    fn write_arithmetic(&mut self, token_type: TokenType) -> Result<String, &'static str> {
        match token_type {
            TokenType::Add => Ok(self.add()),
            TokenType::Subtract => Ok(self.subtract()),
            TokenType::And => Ok(self.and()),
            TokenType::Or => Ok(self.or()),
            TokenType::Not => Ok(self.not()),
            TokenType::Negate => Ok(self.negate()),
            TokenType::Equal => Ok(self.equal()),
            TokenType::GreaterThan => Ok(self.greater_than()),
            TokenType::LessThan => Ok(self.less_than()),
            _ => Err("Invalid arithmetic command"),
        }
    }

    fn write_call(&mut self, symbol: String, nargs: u16) -> Result<String, &'static str> {
        let stepvec = vec![
            format!("@RET-{}${}\n", symbol, self.line_count),
            AsmWriter::push_from_a(),
            String::from("@LCL\n"),
            AsmWriter::push_from_m(),
            String::from("@ARG\n"),
            AsmWriter::push_from_m(),
            String::from("@THIS\n"),
            AsmWriter::push_from_m(),
            String::from("@THAT\n"),
            AsmWriter::push_from_m(),
            format!(
                "@SP\nD=M\n@{}\nD=D-A\n@ARG\nM=D\n@SP\nD=M\n@LCL\nM=D\n",
                nargs + 5
            ),
            self.write_goto(symbol.clone()).unwrap(),
            format!("(RET-{}${})\n", symbol, self.line_count),
        ];
        Ok(stepvec.join(""))
    }

    fn write_function(&self, symbol: String, mut nvars: u16) -> Result<String, &'static str> {
        let mut stepvec = vec![format!("({})\n", symbol)];
        while nvars > 0 {
            stepvec.push(
                self.write_push(String::from("constant"), 0, String::new())
                    .unwrap(),
            );
            nvars -= 1;
        }
        Ok(stepvec.join(""))
    }

    fn write_return(&self) -> Result<String, &'static str> {
        let stepvec = vec![String::from("@LCL\nD=M\n@R14\nM=D\n@5\nA=D-A\nD=M\n@R15\nM=D\n"),
        self.write_pop(String::from("argument"), 0, String::new()).unwrap(),
        String::from("@ARG\nD=M+1\n@SP\nM=D\n@R14\nAM=M-1\nD=M\n@THAT\nM=D\n@R14\nAM=M-1\nD=M\n@THIS\nM=D\n@R14\nAM=M-1\nD=M\n@ARG\nM=D\n@R14\nAM=M-1\nD=M\n@LCL\nM=D\n@R15\nA=M\n0;JMP\n")];

        Ok(stepvec.join(""))
    }

    fn write_label(&self, label: String) -> Result<String, &'static str> {
        Ok(format!("({})\n", &label))
    }

    fn write_goto(&self, label: String) -> Result<String, &'static str> {
        Ok(format!("@{}\n0;JMP\n", label))
    }

    fn write_if(&mut self, label: String) -> Result<String, &'static str> {
        let mut out = AsmWriter::write_pop_to_d();
        out.push_str(&format!("@{}\nD;JLT\n", label));
        Ok(out)
    }

    fn get_operands() -> String {
        // Puts y in d, and x in a
        let stepvec = vec![AsmWriter::write_pop_to_d(), AsmWriter::peek_next_value()];
        stepvec.join("")
    }

    fn equal(&mut self) -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&self.write_comparison("JEQ"));
        self.branch_count += 1;
        out
    }

    fn greater_than(&mut self) -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&self.write_comparison("JGT"));
        self.branch_count += 1;
        out
    }

    fn less_than(&mut self) -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&self.write_comparison("JLT"));
        self.branch_count += 1;
        out
    }

    fn write_comparison(&self, instruction: &str) -> String {
        let out = format!("D=M-D\n@BRANCH{bcount}\nD;{in}\nD=0\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@BRANCH{bcount}END\n0;JMP\n(BRANCH{bcount})\nD=-1\n@SP\nA=M\nM=D\n@SP\nM=M+1\n(BRANCH{bcount}END)\n",
        in=instruction, bcount=self.branch_count);
        String::from(out)
    }

    fn add(&self) -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&format!("D=D+M\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn and(&self) -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&format!("D=D&M\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn or(&self) -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&format!("D=D|M\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn subtract(&self) -> String {
        let mut out = AsmWriter::get_operands();
        out.push_str(&format!("D=M-D\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn not(&self) -> String {
        let mut out = AsmWriter::write_pop_to_d();
        out.push_str(&format!("D=!D\n"));
        out.push_str(&AsmWriter::push_from_d());
        out
    }

    fn negate(&self) -> String {
        let mut out = AsmWriter::write_pop_to_d();
        out.push_str(&format!("D=-D\n"));
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

    fn push_from_m() -> String {
        //Assumes that the pushed value is in A
        String::from("D=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n")
    }

    fn push_from_d() -> String {
        //Assumes that the pushed value is in D
        String::from("@SP\nA=M\nM=D\n@SP\nM=M+1\n")
    }

    fn write_pop_to_d() -> String {
        //Puts the value in D
        String::from("@SP\nAM=M-1\nD=M\n")
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
    fn test_push_static() {
        let st = SymbolTable::new();
        let mut writer = AsmWriter::from(st);
        let out = writer.write_command(Command::Push {
            segment: String::from("static"),
            index: 0,
            class_name: String::from("Main"),
        });
        assert_eq!(
            out.unwrap(),
            String::from("//Command #0\n@Main.0\nA=M\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n")
        );
    }

    #[test]
    fn test_pop_static() {
        let st = SymbolTable::new();
        let mut writer = AsmWriter::from(st);
        let out = writer.write_command(Command::Pop {
            segment: String::from("static"),
            index: 0,
            class_name: String::from("Main"),
        });
        assert_eq!(
            out.unwrap(),
            String::from("//Command #0\n@SP\nAM=M-1\nD=M\n@Main.0\nM=D\n")
        );
    }

    #[test]
    fn test_add() {
        let mut st = SymbolTable::new();
        st.load_starting_table();
        let mut writer = AsmWriter::from(st);
        let out = writer.write_command(Command::Arithmetic(TokenType::Add));
        assert_eq!(
            out.unwrap(),
            String::from(
                "//Command #0\n@SP
AM=M-1
D=M
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
        let st = SymbolTable::new();
        let writer = AsmWriter::from(st);
        assert_eq!(
            writer.add(),
            String::from(
                "@SP
AM=M-1
D=M
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
    fn test_equal_writer() {
        let st = SymbolTable::new();
        let mut writer = AsmWriter::from(st);
        let out = writer.write_command(Command::Arithmetic(TokenType::Equal));
        assert_eq!(
            out.unwrap(),
            String::from(
                "//Command #0\n@SP\nAM=M-1\nD=M\n@SP\nAM=M-1\nD=M-D
@BRANCH0
D;JEQ
D=0
@SP
A=M
M=D
@SP
M=M+1
@BRANCH0END
0;JMP
(BRANCH0)
D=-1
@SP
A=M
M=D
@SP
M=M+1
(BRANCH0END)
"
            )
        );
    }
}
