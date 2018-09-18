use regex::Regex;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
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
    Symbol,
    Index,
    Comment,
    Label,
    If,
    Goto,
    Function,
    Call,
    Return,
    Undefined,
}

// Token Struct
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token: String,
    pub token_type: TokenType,
    pub is_keyword: bool,
}

impl Token {
    pub fn new(token_type: TokenType) -> Token {
        Token {
            token: String::new(),
            token_type,
            is_keyword: false
        }
    }

    pub fn from(token: String, token_type: TokenType, is_keyword: bool) -> Token {
        Token { token, token_type, is_keyword}
    }
}

pub type TokenList = Vec<Token>;

//MatchRule Struct
pub struct MatchRule {
    return_type: TokenType,
    rule: Regex,
    is_keyword: bool,
}

impl MatchRule {
    pub fn new(return_type: TokenType, rule: Regex, is_keyword: bool) -> MatchRule {
        MatchRule {
            return_type,
            rule,
            is_keyword,
        }
    }

    pub fn matches_str(&self, input: &str) -> bool {
        self.rule.is_match(input)
    }
}

//Tokenizer Struct
pub struct Tokenizer {
    match_rules: Vec<MatchRule>,
}

impl Tokenizer {
    pub fn from(match_rules: Vec<MatchRule>) -> Tokenizer {
        Tokenizer { match_rules }
    }

    pub fn add_rule(&mut self, match_rule: MatchRule) {
        self.match_rules.push(match_rule)
    }

    pub fn tokenize(&self, input: &str) -> Result<TokenList, &'static str> {
        let mut result: TokenList = Vec::new();
        let word_vec = input.trim().split_whitespace();
        for word in word_vec {
            let mut token = Token::new(TokenType::Undefined);
            for rule in &self.match_rules {
                if rule.matches_str(word) {
                    token = Token::from(String::from(word), rule.return_type, rule.is_keyword);
                    break;
                }
            }
            let t = token.token_type;
            result.push(token);
            // Stop tokenizing once we hit a comment
            if t == TokenType::Comment {
                break;
            }
        }
        Ok(result)
    }
}

pub fn default_ruleset() -> Vec<MatchRule> {
    vec![
        //Comments
        MatchRule::new(TokenType::Comment, Regex::new(r"^//").unwrap(), false),
        //Memory Access
        MatchRule::new(TokenType::Push, Regex::new("push").unwrap(), true),
        MatchRule::new(TokenType::Pop, Regex::new("pop").unwrap(), true),
        //Arthmetic 
        MatchRule::new(TokenType::Add, Regex::new("add").unwrap(), true),
        MatchRule::new(TokenType::Subtract, Regex::new("sub").unwrap(), true),
        MatchRule::new(TokenType::Negate, Regex::new("neg").unwrap(), true),
        MatchRule::new(TokenType::Equal, Regex::new("eq").unwrap(), true),
        MatchRule::new(TokenType::GreaterThan, Regex::new("gt").unwrap(), true),
        MatchRule::new(TokenType::LessThan, Regex::new("lt").unwrap(), true),
        MatchRule::new(TokenType::And, Regex::new("and").unwrap(), true),
        MatchRule::new(TokenType::Or, Regex::new("or").unwrap(), true),
        MatchRule::new(TokenType::Not, Regex::new("not").unwrap(), true),
        //Symbols
        MatchRule::new(TokenType::Label, Regex::new("label").unwrap(), true),
        MatchRule::new(TokenType::If, Regex::new("if-goto").unwrap(), true),
        MatchRule::new(TokenType::Goto, Regex::new("goto").unwrap(), true),
        MatchRule::new(TokenType::Function, Regex::new("function").unwrap(), true),
        MatchRule::new(TokenType::Call, Regex::new("call").unwrap(), true),
        MatchRule::new(TokenType::Return, Regex::new("return").unwrap(), true),
        MatchRule::new(TokenType::Symbol, Regex::new(r"[a-z_.]+").unwrap(), false),
        MatchRule::new(TokenType::Index, Regex::new(r"[0-9]+").unwrap(), false),
    ]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initialize_tokenizer() {
        let _ = Tokenizer::from(default_ruleset());
    }

    #[test]
    fn token_test1() {
        let t = Tokenizer::from(default_ruleset());
        let input = "add eq sub";
        let result = t.tokenize(input);
        let test_vec = vec![
            Token::from(String::from("add"), TokenType::Add, true),
            Token::from(String::from("eq"), TokenType::Equal, true),
            Token::from(String::from("sub"), TokenType::Subtract, true),
        ];
        assert_eq!(result.unwrap(), test_vec);
    }

    #[test]
    fn token_test_undefined() {
        let t = Tokenizer::from(default_ruleset());
        let input = "add eq %$^%";
        let result = t.tokenize(input);
        let test_vec = vec![
            Token::from(String::from("add"), TokenType::Add, true),
            Token::from(String::from("eq"), TokenType::Equal, true),
            Token::from(String::from(""), TokenType::Undefined, false),
        ];
        assert_eq!(result.unwrap(), test_vec);
    }

    #[test]
    fn token_test_empty_line() {
        let t = Tokenizer::from(default_ruleset());
        let input = "";
        let result = t.tokenize(input);
        let test_vec = vec![];
        assert_eq!(result.unwrap(), test_vec);
    }

    #[test]
    fn token_test_memory_command() {
        let t = Tokenizer::from(default_ruleset());
        let input = "push local 2";
        let result = t.tokenize(input);
        let test_vec = vec![
            Token::from(String::from("push"), TokenType::Push, true),
            Token::from(String::from("local"), TokenType::Symbol, false),
            Token::from(String::from("2"), TokenType::Index, false),
        ];
        assert_eq!(result.unwrap(), test_vec);
    }

    #[test]
    fn token_test_comment_line() {
        let t = Tokenizer::from(default_ruleset());
        let input = "//add eq test";
        let result = t.tokenize(input);
        let test_vec = vec![Token::from(String::from("//add"), TokenType::Comment, false)];
        assert_eq!(result.unwrap(), test_vec);
    }

    #[test]
    fn token_test_inline_comment() {
        let t = Tokenizer::from(default_ruleset());
        let input = "add eq //test inline doesn't read more symbols";
        let result = t.tokenize(input);
        let test_vec = vec![
            Token::from(String::from("add"), TokenType::Add, true),
            Token::from(String::from("eq"), TokenType::Equal, true),
            Token::from(String::from("//test"), TokenType::Comment, false),
        ];
        assert_eq!(result.unwrap(), test_vec);
    }


}
