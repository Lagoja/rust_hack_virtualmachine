use lib::tokenizer::{Token, TokenList, TokenType};
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Command {
    Push { segment: String, index: u16, class_name: String },
    Pop { segment: String, index: u16, class_name: String},
    Arithmetic(TokenType),
    Goto(String),
    If(String),
    Label(String),
    Function { symbol: String, nvars: u16 },
    Call { symbol: String, nargs: u16 },
    Return,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<TokenList>,
    next_command: u16,
    total_commands: u16,
    class_name: String
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokens: vec![],
            next_command: 0,
            total_commands: 10,
            class_name: String::new()
        }
    }

    pub fn from(tokens: Vec<TokenList>, class_name: String) -> Parser {
        let l = tokens.len() as u16;
        Parser {
            tokens,
            next_command: 0,
            total_commands: l,
            class_name
        }
    }

    pub fn has_more_commands(&self) -> bool {
        println!("Total Commands: {}, Next Command {}", self.total_commands, self.next_command);
        self.total_commands - self.next_command > 0
    }

    pub fn advance(&mut self) -> Result<Option<Command>, Box<Error>> {
        let token_list: TokenList = self.tokens.get(self.next_command as usize).unwrap().to_vec();
        self.next_command += 1;
        self.parse(token_list)
    }

    fn parse(&mut self, token_list: TokenList) -> Result<Option<Command>, Box<Error>> {
        let mut t_iter = token_list.iter();
        //Empty lines or comments should return Ok(None), so the writer knows to skip them. Bad input or syntax should return an Error, so that we can interrupt parsing.
        let mut result: Option<Command> = None;
        //Need to handle empty lines
        let c: &Token = match t_iter.next() {
            Some(x) => x,
            None => return Ok(result),
        };

        //Need to handle full line comments first.
        if c.token_type == TokenType::Comment {
            return Ok(result);
        }

        //First word should always be a keyword or command. Throw an error if not
        if !c.is_keyword {
            return Err(Box::new(KeywordError {
                line_number: self.next_command,
            }));
        };

        //Now we can start parsing the tokens. Use the first token to identify the command type, and route accordingly
        result = match c.token_type {
            TokenType::Pop | TokenType::Push => {
                let arg1 = t_iter.next().unwrap();
                let arg2 = t_iter.next().unwrap();
                match Parser::mem_access_parse(c, arg1, arg2, self.class_name.clone()) {
                    Some(comm) => Some(comm),
                    None => {
                        return Err(Box::new(ArgumentError {
                            command_type: String::from("Memory Access"),
                            line_number: self.next_command,
                        }))
                    }
                }
            }

            TokenType::Label | TokenType::If | TokenType::Goto => {
                let arg1 = t_iter.next().unwrap();
                match Parser::control_flow_parse(c, arg1) {
                    Some(comm) => Some(comm),
                    None => {
                        return Err(Box::new(ArgumentError {
                            command_type: String::from("Control Flow"),
                            line_number: self.next_command,
                        }))
                    }
                }
            }
            // At this stage, any remaining commands should be Arithmetic
            TokenType::Call | TokenType::Function => {
                let arg1 = t_iter.next().unwrap();
                let arg2 = t_iter.next().unwrap();
                match Parser::function_command_parse(c, arg1, arg2) {
                    Some(comm) => Some(comm),
                    None => {
                        return Err(Box::new(ArgumentError {
                            command_type: String::from("Function"),
                            line_number: self.next_command,
                        }))
                    }
                }
            }

            TokenType::Return => Some(Command::Return),

            _ => match Parser::arithmetic_parse(c) {
                Some(comm) => Some(comm),
                None => {
                    return Err(Box::new(ArgumentError {
                        command_type: String::from("Function"),
                        line_number: self.next_command,
                    }))
                }
            },
        };
        // self.next_command += 1;

        Ok(result)
    }

    fn mem_access_parse(c: &Token, arg1: &Token, arg2: &Token, class_name: String) -> Option<Command> {
        if arg1.token_type == TokenType::Symbol && arg2.token_type == TokenType::Index {
            match c.token_type {
                TokenType::Push => Some(Command::Push {
                    segment: String::from(arg1.token.clone()),
                    index: arg2.token.parse::<u16>().unwrap(),
                    class_name
                }),
                TokenType::Pop => Some(Command::Pop {
                    segment: String::from(arg1.token.clone()),
                    index: arg2.token.parse::<u16>().unwrap(),
                    class_name
                }),
                _ => return None,
            }
        } else {
            None
        }
    }

    fn control_flow_parse(c: &Token, arg1: &Token) -> Option<Command> {
        if arg1.token_type == TokenType::Symbol {
            match c.token_type {
                TokenType::Label => Some(Command::Label(arg1.token.clone())),
                TokenType::Goto => Some(Command::Goto(arg1.token.clone())),
                TokenType::If => Some(Command::If(arg1.token.clone())),
                _ => None,
            }
        } else {
            None
        }
    }

    fn function_command_parse(c: &Token, arg1: &Token, arg2: &Token) -> Option<Command> {
        if arg1.token_type == TokenType::Symbol && arg2.token_type == TokenType::Index {
            match c.token_type {
                TokenType::Function => Some(Command::Function {
                    symbol: arg1.token.clone(),
                    nvars: arg2.token.parse::<u16>().unwrap(),
                }),
                TokenType::Call => Some(Command::Call {
                    symbol: arg1.token.clone(),
                    nargs: arg2.token.parse::<u16>().unwrap(),
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    fn arithmetic_parse(c: &Token) -> Option<Command> {
        Some(Command::Arithmetic(c.token_type))
    }

    //Add another method for processing the leftover tokens, warn on syntax violations
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn memory_access_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![
            Token::from(String::from("push"), TokenType::Push, true),
            Token::from(String::from("local"), TokenType::Symbol, false),
            Token::from(String::from("0"), TokenType::Index, false),
        ];

        let output = parser.parse(input);

        assert_eq!(
            output.unwrap(),
            Some(Command::Push {
                segment: String::from("local"),
                index: 0,
                class_name: String::new()
            })
        );
    }

    #[test]
    fn arithmetic_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![Token::from(String::from("add"), TokenType::Add, true)];

        let output = parser.parse(input);
        assert_eq!(output.unwrap(), Some(Command::Arithmetic(TokenType::Add)));
    }

    #[test]
    fn comment_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![
            Token::from(String::from("//"), TokenType::Comment, false),
            Token::from(String::from("hello"), TokenType::Symbol, false),
        ];

        let output = parser.parse(input);
        assert_eq!(output.unwrap(), None);
    }

    #[test]
    fn inline_comment_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![
            Token::from(String::from("add"), TokenType::Add, true),
            Token::from(String::from("//"), TokenType::Comment, false),
        ];

        let output = parser.parse(input);
        assert_eq!(output.unwrap(), Some(Command::Arithmetic(TokenType::Add)));
    }

    #[test]
    fn no_tokens_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![];

        let output = parser.parse(input);
        assert_eq!(output.unwrap(), None);
    }

}

// #[derive(Debug)]
// enum ParserError {
//     ArgumentError(ArgumentError),
//     KeywordError(KeywordError),
// }

#[derive(Debug)]
struct ArgumentError {
    command_type: String,
    line_number: u16,
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Improper arguments for {} command at line {}",
            self.command_type, self.line_number
        )
    }
}

impl Error for ArgumentError {}

#[derive(Debug)]
struct KeywordError {
    line_number: u16,
}

impl fmt::Display for KeywordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected keyword at line {}", self.line_number)
    }
}

impl Error for KeywordError {}
