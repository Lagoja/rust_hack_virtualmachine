use lib::tokenizer::{Token, TokenList, TokenType};

#[derive(Debug, PartialEq)]
pub enum Command {
    Push { segment: String, index: u16 },
    Pop { segment: String, index: u16 },
    Arithmetic(TokenType),
    Goto(String),
    If(String),
    Label(String),
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<TokenList>,
    next_command: usize,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokens: vec![],
            next_command: 0,
        }
    }

    pub fn from(tokens: Vec<TokenList>) -> Parser {
        Parser {
            tokens,
            next_command: 0,
        }
    }

    pub fn has_more_commands(&self) -> bool {
        &self.tokens.len() - &self.next_command > 0
    }

    pub fn advance(&mut self) -> Result<Option<Command>, &'static str> {
        let token_list: TokenList = self.tokens.get(self.next_command).unwrap().to_vec();
        self.next_command += 1;
        self.parse(token_list)
    }

    fn parse(&mut self, token_list: TokenList) -> Result<Option<Command>, &'static str> {
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
            return Err("Invalid command: Expected keyword");
        };

        //Now we can start parsing the tokens. Use the first token to identify the command type, and route accordingly
        result = match c.token_type {
            TokenType::Pop | TokenType::Push => {
                let arg1 = t_iter.next().unwrap();
                let arg2 = t_iter.next().unwrap();
                match Parser::mem_access_parse(c, arg1, arg2) {
                    Some(comm) => Some(comm),
                    None => return Err("Improper arguments for Memory Access Command"),
                }
            },
            TokenType::Label | TokenType::If | TokenType::Goto => {
                let arg1 = t_iter.next().unwrap();
                match Parser::control_flow_parse(c, arg1) {
                    Some(comm) => Some(comm),
                    None => return Err("Improper arguments for Control Flow Command")
                }
            },
            // At this stage, any remaining commands should be Arithmetic
            _ => match Parser::arithmetic_parse(c) {
                Some(comm) => Some(comm),
                None => return Err("Improper arguments for Arthmetic Command"),
            },
        };

        Ok(result)
    }

    fn mem_access_parse(c: &Token, arg1: &Token, arg2: &Token) -> Option<Command> {
        if arg1.token_type == TokenType::Symbol && arg2.token_type == TokenType::Index {
            match c.token_type {
                TokenType::Push => Some(Command::Push {
                    segment: String::from(arg1.token.clone()),
                    index: arg2.token.parse::<u16>().unwrap(),
                }),
                TokenType::Pop => Some(Command::Pop {
                    segment: String::from(arg1.token.clone()),
                    index: arg2.token.parse::<u16>().unwrap(),
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
                index: 0
            })
        );
    }

    #[test]
    fn arithmetic_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![Token::from(String::from("add"), TokenType::Add, true)];

        let output = parser.parse(input);
        assert_eq!(
            output.unwrap(),
            Some(Command::Arithmetic(TokenType::Add))
        );
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
        assert_eq!(
            output.unwrap(),
            Some(Command::Arithmetic(TokenType::Add))
        );
    }

    #[test]
    fn no_tokens_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![];

        let output = parser.parse(input);
        assert_eq!(output.unwrap(), None);
    }

}
