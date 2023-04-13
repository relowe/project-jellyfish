#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::Read;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    TEXT(String),
    ID(String),
    NUMBER(f64),
    ADD,       // +
    SUB,       // -
    MUL,       // *
    DIV,       // /
    MOD,       // mod
    POW,       // ^
    COLON,     // :
    DEF,       // definitions
    END,       // end
    STRUCT,    // structure
    EQ,        // =
    NE,        // !=
    LT,        // <
    LE,        // <=
    GT,        // >
    GE,        // >=
    IS,        // is
    WORDNOT,   // not
    FUN,       // function
    RETURNS,   // returns
    RETURN,    // return
    LPAREN,    // (
    RPAREN,    // )
    COMMA,     // ,
    CHANGEABLE, // changeable
    ARRAY,     // array
    OF,        // of
    LBRACKET,  // [
    RBRACKET,  // ]
    NOTHING,   // nothing
    PROGRAM,   // program
    QUIT,      // quit
    LINK,      // link
    LINKED,    // linked
    UNLINK,    // unlink
    TO,        // to
    BREAK,     // break
    CONTINUE,  // continue
    NUMTYPE,   // number
    TEXTTYPE,  // text
    IF,        // if
    THEN,      // then
    ELSE,      // else
    WHILE,     // while
    REPEAT,    // repeat
    FOREVER,   // forever
    TIMES,     // times
    FOR,       // for
    ALL,       // all
    IN,        // in
    AND,       // and
    OR,        // or
    BOR,       // bit_or
    BXOR,      // bit_xor
    BAND,      // bit_and
    BSL,       // bit_sl
    BSR,       // bit_sr
    BNOT,      // bit_not
    LCURLY,    // {
    RCURLY,    // }
    PERIOD,    // .
    EOF, // end of file
    INVALID,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Option<String>,
    pub row: u32,
    pub col: u32,
}

#[derive(Debug)]
pub struct Lexer {
    curr_row: u32,
    curr_col: u32,
    curr_lex: String,
    curr_char: char,
    pub curr_token: Token,
    raw_text: String,
}

impl Lexer {
    // Create a new lexer structure from a provided code String
    // This function also loads in the first character into the
    //  'curr_char' buffer for processing
    pub fn new(text: String) -> Result<Self, &'static str> {
        let mut lex = Lexer {
            raw_text: text.chars().rev().collect::<String>(),
            curr_row: 1,
            curr_col: 0,
            curr_lex: "".to_string(),
            curr_char: '\0',
            // the initial token will be an invalid token
            curr_token: Token{
                token_type: TokenType::INVALID,
                lexeme: None,
                row: 0,
                col: 0,
            }
        };
        lex.consume()?;
        Ok(lex)
    }

    // Try to load a string into the lexer from a file
    // This acts as an alternative to 'new', since it creates
    //  its own lexer structure and returns that
    pub fn from_file(file: String) -> Result<Self, &'static str> {
        let result = File::open(file);
        if result.is_err() {
            return Err("Could not open file");
        }
        let mut file_obj = result.unwrap();
        let mut code: String = String::new();
        if file_obj.read_to_string(&mut code).is_err() {
            return Err("Could not read from file");
        }
        Lexer::new(code)
    }

    // Consumes a single character from the lexer's raw text
    // Throws an error if non-ASCII characters are consumed
    pub fn consume(&mut self) -> Result<char, &'static str> {
        if self.raw_text.len() < 1 {
            self.curr_char = '\0';
            return Ok(self.curr_char);
        }

        self.curr_char = self.raw_text.pop().unwrap_or('\0');
        self.curr_col += 1;
        if self.curr_char == '\n' {
            self.curr_col = 0;
            self.curr_row += 1;
        }
        if !self.curr_char.is_ascii() {
            return Err("Only printable ASCII characters are allowed");
        }
        Ok(self.curr_char)
    }

    // Consume all whitespace characters, if the current character
    //  is not whitespace, or is EOF, this does nothing
    // Will consume all trailing whitespace until EOF
    pub fn consume_whitespace(&mut self) -> Result<(), &'static str> {
        while self.curr_char.is_whitespace() || self.curr_char == '#' {
            if self.curr_char == '\0' {
                return Ok(());
            }
            if self.curr_char == '#' {
                while !(self.curr_char == '\n' || self.curr_char == '\0') {
                    self.consume()?;
                }
            }
            self.consume()?;
        }
        return Ok(());
    }

    // Consume and create the next token, store it in curr_token,
    //  and return it. If EOF token is already created, return
    //  it instead, without lexing any further.
    pub fn next(&mut self) -> Result<Token, &'static str> {
        if self.curr_token.token_type == TokenType::EOF {
            return Ok(self.curr_token.clone());
        }
        else if self.curr_char == '\0' {
            self.create_token(self.curr_row, self.curr_col, TokenType::EOF);
            return Ok(self.curr_token.clone());
        }

        self.consume_whitespace()?;

        if self.lex_single()? {
            Ok(self.curr_token.clone())
        }
        else if self.lex_multi_fixed()? {
            Ok(self.curr_token.clone())
        }
        else if self.lex_other()? {
            Ok(self.curr_token.clone())
        }
        else {
            self.curr_lex = self.curr_char.to_string();
            self.curr_token = self.create_token(self.curr_row, self.curr_col, TokenType::INVALID);
            self.consume()?;
            Ok(self.curr_token.clone())
        }
    }

    // Return a boolean for if the current token is EOF
    pub fn is_done(&self) -> bool {
        self.curr_token.token_type == TokenType::EOF
    }

    // Attempt to create a token for single character tokens
    pub fn lex_single(&mut self) -> Result<bool, &'static str> {

        let t_type : TokenType = match self.curr_char {
            '+' => TokenType::ADD,
            '-' => TokenType::SUB,
            '*' => TokenType::MUL,
            '/' => TokenType::DIV,
            '^' => TokenType::POW,
            ':' => TokenType::COLON,
            '=' => TokenType::EQ,
            '(' => TokenType::LPAREN,
            ')' => TokenType::RPAREN,
            '{' => TokenType::LCURLY,
            '}' => TokenType::RCURLY,
            '[' => TokenType::LBRACKET,
            ']' => TokenType::RBRACKET,
            ',' => TokenType::COMMA,
            '.' => TokenType::PERIOD,
            _ => TokenType::INVALID,
        };

        if t_type == TokenType::INVALID {
            Ok(false)
        }
        else {
            self.curr_lex.push(self.curr_char);
            self.create_token(self.curr_row, self.curr_col, t_type);
            self.consume()?;
            Ok(true)
        }
    }

    // Attempt to lex from a multi-character, but fixed, set of tokens
    // This only includes sigils, not letters or keywords
    pub fn lex_multi_fixed(&mut self) -> Result<bool, &'static str> {
        let mut lex: String = self.curr_char.to_string();
        let start_row = self.curr_row;
        let start_col = self.curr_col;

        let multi_fixed_tokens = [
            ("<", TokenType::LT),
            ("<=", TokenType::LE),
            (">", TokenType::GT),
            (">=", TokenType::GE),
            ("!=", TokenType::NE),
        ];

        let mut matches: Vec<&(&str, TokenType)> = multi_fixed_tokens.iter().filter(|item| (item.0).starts_with(&lex)).clone().collect();
        let mut new_matches: Vec<&(&str, TokenType)>;
        // Keep looping to find 1 (or zero) potential matches
        while matches.len() > 1 {
            self.consume()?;
            lex.push(self.curr_char);
            new_matches = matches.clone().into_iter().filter(|item| (item.0).starts_with(&lex)).collect();
            // If this sets us to 0 matches, undo the new character
            //  and remove the too-long lexes that didn't match
            if new_matches.len() == 0 {
                lex.pop();
                matches = matches.into_iter().filter(|item| (item.0).len() == lex.len()).collect();
            }
            else {
                matches = new_matches;
            }
        }

        // 'matches' has only one (or zero) values now, if zero return
        //  an invalid token
        if lex.len() >= 1 && matches.len() > 0 {
            // We have at least part of a matching fixed lex
            //  keep consuming token until we get a complete match
            //  or a conflicting caracter.
            let match_lex = matches[0].0.clone();
            let match_token = matches[0].1.clone();

            while lex.len() < match_lex.len() && match_lex.starts_with(&lex) {
                self.consume()?;
                lex.push(self.curr_char);
            }

            if lex == match_lex {
                // We actually got a match, make the token and consume the character
                self.consume()?;

                self.curr_lex = lex.clone();
                self.create_token(start_row, start_col, match_token);
            }
            else {
                // We didn't get a match, don't consume the current token
                lex.pop();
                self.curr_lex = lex.to_string().clone();
                self.create_token(start_row, start_col, TokenType::INVALID);
            }
            Ok(true)
        }
        else if lex.len() > 1 {
            // We have parts of a lex that did match at some
            //  point, but now form an invalid token
            lex.pop();
            self.curr_lex = lex.to_string().clone();
            self.create_token(start_row, start_col, TokenType::INVALID);
            Ok(true)
        }
        else {
            // We didn't have any lexemes to compare
            Ok(false)
        }
    }

    // Attempt to create a token for numbers, variables (id),
    //  text, and keywords
    pub fn lex_other(&mut self) -> Result<bool, &'static str> {

        if self.curr_char.is_numeric() || self.curr_char == '.' {
            return self.lex_number();
        }
        if self.curr_char == '"' || self.curr_char == '\'' {
            return self.lex_text();
        }
        else if self.curr_char.is_alphabetic() || self.curr_char == '_' {
            return self.lex_id();
        }
        else {
            Ok(false)
        }
    }

    // Lex all concurrent letters (and underscores) together into a single id
    // This stops at whitespace (or a non-letter)
    pub fn lex_id(&mut self) -> Result<bool, &'static str> {
        let start_row = self.curr_row;
        let start_col = self.curr_col;
        while self.curr_char.is_alphabetic() || self.curr_char == '_' {
            self.curr_lex.push(self.curr_char);
            self.consume()?;
        }

        let lex_val = self.curr_lex.clone().to_lowercase();

        let token_type = match lex_val.as_str() {
            "definitions" => TokenType::DEF,
            "end" => TokenType::END,
            "structure" => TokenType::STRUCT,
            "is" => TokenType::IS,
            "not" => TokenType::WORDNOT,
            "function" => TokenType::FUN,
            "returns" => TokenType::RETURNS,
            "return" => TokenType::RETURN,
            "changeable" => TokenType::CHANGEABLE,
            "array" => TokenType::ARRAY,
            "of" => TokenType::OF,
            "nothing" => TokenType::NOTHING,
            "program" => TokenType::PROGRAM,
            "quit" => TokenType::QUIT,
            "link" => TokenType::LINK,
            "linked" => TokenType::LINKED,
            "unlink" => TokenType::UNLINK,
            "to" => TokenType::TO,
            "break" => TokenType::BREAK,
            "continue" => TokenType::CONTINUE,
            "number" => TokenType::NUMTYPE,
            "text" => TokenType::TEXTTYPE,
            "if" => TokenType::IF,
            "then" => TokenType::THEN,
            "else" => TokenType::ELSE,
            "while" => TokenType::WHILE,
            "repeat" => TokenType::REPEAT,
            "forever" => TokenType::FOREVER,
            "times" => TokenType::TIMES,
            "for" => TokenType::FOR,
            "all" => TokenType::ALL,
            "in" => TokenType::IN,
            "and" => TokenType::AND,
            "or" => TokenType::OR,
            "bit_or" => TokenType::BOR,
            "bit_xor" => TokenType::BXOR,
            "bit_and" => TokenType::BAND,
            "bit_sl" => TokenType::BSL,
            "bit_sr" => TokenType::BSR,
            "bit_not" => TokenType::BNOT,
            "mod" => TokenType::MOD,
            _ => TokenType::ID(lex_val),
        };

        self.create_token(start_row, start_col, token_type);
        Ok(true)
    }

    // Lex all of the characters inside of a string
    pub fn lex_text(&mut self) -> Result<bool, &'static str> {
        let end_char = self.curr_char;
        let start_col = self.curr_col;
        let start_row = self.curr_row;
        self.curr_lex = String::new();
        self.curr_lex.push(self.curr_char);
        self.consume()?;

        while self.curr_char != end_char && self.curr_char != '\0' {
            if self.curr_char == '\\' {
                self.consume()?;

                let cleaned_char = match self.curr_char {
                    '\t' | '\n' | '\0' => ' ',
                    _ => self.curr_char,
                };

                let escape_char = match cleaned_char {
                    't' => '\t',
                    'n' => '\n',
                    '"' => '"',
                    '\'' => '\'',
                    '\\' => '\\',
                    _ => cleaned_char,
                };

                if escape_char == cleaned_char && cleaned_char != '\\' {
                    self.curr_lex.push('\\');
                }

                self.curr_lex.push(escape_char);
            }
            else {
                let cleaned_char = match self.curr_char {
                    '\t' | '\n' | '\0' => ' ',
                    _ => self.curr_char,
                };
                self.curr_lex.push(cleaned_char);
            }

            self.consume()?;
        }

        // If the string didn't close, make an invalid token
        if self.curr_char != end_char {
            self.create_token(start_row, start_col, TokenType::INVALID);
        }
        else {
            self.curr_lex.push(self.curr_char);
            self.consume()?;
            let mut lex_val = self.curr_lex.chars();
            lex_val.next();
            lex_val.next_back();
            self.create_token(start_row, start_col, TokenType::TEXT(lex_val.as_str().to_string()));
        }

        Ok(true)
    }

    // Lexes all concurrent numbers, creates a token, and returns
    //  true if the token was make successfully
    pub fn lex_number(&mut self) -> Result<bool, &'static str> {
        if !(self.curr_char.is_numeric() || self.curr_char == '.') {
            return Ok(false);
        }
        let start_row = self.curr_row;
        let start_col = self.curr_col;
        while self.curr_char.is_numeric() {
            self.curr_lex.push(self.curr_char);
            self.consume()?;
        }
        if self.curr_char == '.' {
            self.curr_lex.push(self.curr_char);
            self.consume()?;
        }
        while self.curr_char.is_numeric() {
            self.curr_lex.push(self.curr_char);
            self.consume()?;
        }
        self.create_token(start_row, start_col, TokenType::NUMBER(self.curr_lex.parse::<f64>().unwrap()));
        Ok(true)
    }

    // Create a token given the provided start row and column
    // Clears out the currently stored lexeme
    pub fn create_token(&mut self, row: u32, col: u32, token_type: TokenType) -> Token {
        let lexeme: Option<String> = match self.curr_lex.len() {
            0 => None,
            _ => Some(self.curr_lex.clone())
        };
        
        self.curr_token = Token{
            token_type: token_type,
            lexeme: lexeme,
            row: row,
            col: col,
        };
        self.curr_lex = "".to_string();
        self.curr_token.clone()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut lex: Lexer;
    let res: Result<Lexer, &'static str>;
    if args.len() > 1 {
        let fname = &args[1];
        res = Lexer::from_file(fname.to_string());
    }
    else {
        res = Lexer::new("hello, world + 123 - 11.4491 = 12333 test.file \n hello again".to_string());
    }

    lex = match res {
        Ok(val) => val,
        Err(val) => panic!("{}", val),
    };
    
    while !lex.is_done() {
        println!("{:?}", lex.next());
    }
}
