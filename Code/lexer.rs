use std::env;
use std::fs::File;
use std::io::{Error, Read};

enum TokenType {
    TEXT(String),
    NUMBER(f64),
    EOF,
}

struct Token {
    token_type: TokenType,
    lexeme: Option<String>,
    row: u32,
    col: u32,
}

#[derive(Debug)]
struct Lexer {
    file_name: String,
    file: File,
    curr_row: u32,
    curr_col: u32,
    curr_lex: String,
    curr_char: char,
}

impl Lexer {
    pub fn new(file_name: String) -> Result<Self, Error> {
        let file = File::open(file_name.clone())?;

        Ok(Lexer {
            file_name,
            file,
            curr_row: 0,
            curr_col: 0,
            curr_lex: "".to_string(),
            curr_char: ' ',
        })
    }

    pub fn next(&mut self) -> Result<char, Error> {
        let mut buf = [0; 1];
        let n = self.file.read(&mut buf)?;
        if n == 0 {
            self.curr_char = '\0';
            Ok('\0')
        } else {
            let c = buf[0] as char;
            if c == '\n' {
                self.curr_row += 1;
                self.curr_col = 0;
            } else {
                self.curr_col += 1;
            }
            self.curr_char = c;
            Ok(c)
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Error: No file provided");
        return;
    }

    let lex_res = Lexer::new(args[1].clone());
    println!("File: {:?}", lex_res);


    let mut lex = match lex_res {
        Ok(val) => val,
        Err(e) => {
            println!("Couldn't open file: {}", e);
            return;
        }
    };

    println!("File: {:?}", lex);
    
    while lex.curr_char != '\0' {
        println!("{}", lex.next().expect("No character found!"));
    }
}
