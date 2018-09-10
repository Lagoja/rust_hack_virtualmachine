use lib::tokenizer::{Token, TokenList, TokenType};

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Push,
    Pop,
    Add,
    Subtract,
    Negate,
    Equal,
    LessThan,
    GreaterThan,
    And,
    Or,
    Not,
}

#[derive(Debug, PartialEq)]
pub struct Command {
    command_type: CommandType,
    pub arg1: Option<String>,
    pub arg2: Option<String>,
    pub is_arithmetic: bool,
}

impl Command {
    pub fn new(
        command_type: CommandType,
        arg1: Option<String>,
        arg2: Option<String>,
        is_arithmetic: bool,
    ) -> Command {
        Command {
            command_type,
            arg1,
            arg2,
            is_arithmetic,
        }
    }
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
        Parser{
            tokens,
            next_command: 0
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

    pub fn parse(&mut self, token_list: TokenList) -> Result<Option<Command>, &'static str> {
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
                    None => return Err("Improper arguments for Memory Access Command")
                }
            }
            // At this stage, any remaining commands should be Arithmetic
            _ => match Parser::al_parse(c) {
                Some(comm) => Some (comm),
                None => return Err("Improper arguments for Arthmetic Command")
            }
        };

        Ok(result)
    }

    fn mem_access_parse(c: &Token, arg1: &Token, arg2: &Token) -> Option<Command> {
        let command_type = match c.token_type {
            TokenType::Push => CommandType::Push,
            TokenType::Pop => CommandType::Pop,
            _ => return None,
        };
        if arg1.token_type == TokenType::Symbol && arg2.token_type == TokenType::Index {
            return Some(Command::new(
                command_type,
                Some(String::from(arg1.token.clone())),
                Some(String::from(arg2.token.clone())),
                false,
            ));
        } else {
            return None;
        }
    }

    fn al_parse(c: &Token) -> Option<Command> {
        let command_type = match c.token_type {
            TokenType::Add => CommandType::Add,
            TokenType::Subtract => CommandType::Subtract,
            TokenType::Negate => CommandType::Negate,
            TokenType::Equal => CommandType::Equal,
            TokenType::GreaterThan => CommandType::GreaterThan,
            TokenType::LessThan => CommandType::LessThan,
            TokenType::And => CommandType::And,
            TokenType::Or => CommandType::Or,
            TokenType::Not => CommandType::Not,
            _ => return None,
        };

        Some(Command::new(command_type, None, None, true))
    }
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

        assert_eq!(output.unwrap(), Some(Command::new(CommandType::Push, Some(String::from("local")), Some(String::from("0")),false)));

    }

    #[test]
    fn arithmetic_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![
            Token::from(String::from("add"), TokenType::Add, true),
        ];

        let output = parser.parse(input);
        assert_eq!(output.unwrap(), Some(Command::new(CommandType::Add, None, None,true)));
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
    fn no_tokens_parse_test() {
        let mut parser = Parser::new();
        let input: TokenList = vec![];

        let output = parser.parse(input);
        assert_eq!(output.unwrap(), None);
    }

}
