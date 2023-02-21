use std::env;
use std::fs::File;
use std::io::Read;

#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    TEXT(String),
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
    RETRUNS,   // returns
    RETURN,    // return
    LPAREN,    // (
    RPAREN,    // )
    COMMA,     // ,
    CHANGABLE, // changable
    ARRAY,     // array
    OF,        // of
    LBRACKET,  // [
    RBRACKET,  // ]
    NOTHING,   // nothing
    PROGRAM,   // program
    QUIT,      // quit
    LINK,      // link
    TO,        // to
    UNLINK,    // unlink
    BREAK,     // break
    NUMTYPE,   // number
    TXTTYPE,   // text
    IF,        // if
    ELSE,      // else
    WHILE,     // while
    REPEAT,    // repeat
    TIMES,     // times
    FOR,       // for
    ALL,       // all
    AND,       // and
    OR,        // or
    BOR,       // bit-or
    BXOR,      // bit-xor
    BAND,      // bit-and
    BSL,       // bit-sl
    BSR,       // bit-sr
    BNOT,      // bit-not
    LCURLY,    // {
    RCURLY,    // }
    PERIOD,    // .
    QUOTE,     // '
    DQUOTE,    // "
    EOF,
    INVALID,
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    lexeme: Option<String>,
    row: u32,
    col: u32,
}

#[derive(Debug)]
struct Lexer {
    curr_row: u32,
    curr_col: u32,
    curr_lex: String,
    curr_char: char,
    curr_token: Token,
    raw_text: String,
}

impl Lexer {
    // Create a new lexer structure from a provided code String
    // The code String ('text') is expected to have at least
    //  one character in length
    // This function also loads in the first character into the
    //  'curr_char' buffer for processing
    pub fn new(text: String) -> Result<Self, &'static str> {
        if text.len() <= 0 {
            return Err("Code string cannot be null");
        }
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

    // Consumes a single character from the lexer's raw text
    // Throws an error if non-ASCII characters are consumed
    pub fn consume(&mut self) -> Result<char, &'static str> {
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
        while self.curr_char.is_whitespace() {
            if self.curr_char == '\0' {
                return Ok(());
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
        else if self.lex_other()? {
            Ok(self.curr_token.clone())
        }
        else {
            self.curr_token = self.create_token(self.curr_row, self.curr_col, TokenType::INVALID);
            self.consume()?;
            Ok(self.curr_token.clone())
        }
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
            '\'' => TokenType::QUOTE,
            '"' => TokenType::DQUOTE,
            _ => TokenType::INVALID,
        };

        if t_type == TokenType::INVALID {
            Ok(false)
        }
        else {
            self.curr_lex.push(self.curr_char);
            let token: Token = self.create_token(self.curr_row, self.curr_col, t_type);
            self.consume()?;
            Ok(true)
        }
    }

    // Attempt to lex from a multi-character, but fixed, set of tokens
    // This only includes sigils, not letters or keywords
    pub fn lex_multi_fixed(&mut self) -> Result<bool, &'static str> {
        let lex: String = self.curr_char.to_string();
        let start_row = self.curr_row;
        let start_col = self.curr_col;

        let multi_fixed_tokens = [
            ("<", TokenType::LT),
            ("<=", TokenType::LE),
            (">", TokenType::GT),
            (">=", TokenType::GE),
            ("!=", TokenType::NE),
        ]

        let mut matches = multi_fixed_tokens.iter().filter(|item| item[0].starts_with(lex)).clone().collect();
        while matches.len() >= 1 {
            self.consume()?;
            if matches.len() == 1 {
                // make sure its a complete match and make the token
            }
            lex.push(self.curr_char);
            matches = matches.iter().filter(|item| item[0].starts_with(lex)).clone().collect();
        }

        Ok(false)
    }

    // Attempt to create a token for numbers, variables (id),
    //  text, and keywords
    pub fn lex_other(&mut self) -> Result<bool, &'static str> {

        if self.curr_char.is_numeric() || self.curr_char == '.' {
            return self.lex_number();
        }
        else if self.curr_char.is_alphabetic() {
            return self.lex_id();
        }
        else {
            Ok(false)
        }
    }

    // Lex all concurrent letters together into a single id
    // This stops at whitespace
    pub fn lex_id(&mut self) -> Result<bool, &'static str> {
        if !self.curr_char.is_alphabetic() {
            return Ok(false);
        }
        let start_row = self.curr_row;
        let start_col = self.curr_col;
        while self.curr_char.is_alphabetic() {
            self.curr_lex.push(self.curr_char);
            self.consume()?;
        }
        self.create_token(start_row, start_col, TokenType::TEXT(self.curr_lex.clone()));
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

    let mut code: String = String::new();
    if args.len() > 1 {
        let fname = &args[1];
        let mut file = File::open(fname).expect("Could not open file");
        file.read_to_string(&mut code).expect("Could not read from file");
    }
    else {
        code = "hello, world + 123 - 11.4491 = 12333 test.file \n hello again".to_string();
    }
    
    let mut lex: Lexer = (Lexer::new(code)).expect("Could not create lexer");
    loop {
        lex.next();
        println!("{:?}", lex.curr_token);
        if lex.curr_char == '\0' {
            break;
        }
    }
    lex.next();
    println!("{:?}", lex.curr_token);
}
