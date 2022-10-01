use crate::html::position::Position;
use std::str::Chars;
use crate::html::tokenizer::TokenKind::{Decimal, Eof, Integer, ReservedSymbol, Text, Whitespace};

#[derive(Debug, Clone)]
pub enum TokenKind {
    Illegal,
    Eof,
    Whitespace,

    ReservedSymbol(String),
    Text(String),

    String(String),
    Integer(i64),
    Decimal(f64),
}

#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenKind,
    pos: Position,
    next: Option<Box<Token>>,
}

impl Token {
    pub fn new(kind: TokenKind, pos: Position) -> Token {
        return Token { kind, pos, next: None };
    }
}

fn is_alphanum_(c: char) -> bool {
    return c.is_alphanumeric() || c == '_';
}

fn is_number(c: char) -> bool {
    return c == '0'
        || c == '1'
        || c == '2'
        || c == '3'
        || c == '4'
        || c == '5'
        || c == '6'
        || c == '7'
        || c == '8'
        || c == '9';
}

fn is_ws(c: char) -> bool {
    return c == '\n' || c == '\t' || c == ' ';
}

fn is_reserved_symbol(c: char) -> bool {
    let symbols: Vec<&str> = vec!["<", ">", "!", "=", "-", "/", "&"];
    for s in symbols {
        if s == c.to_string() {
            return true;
        }
    }
    return false;
}

pub struct Tokenizer {
    target: String,
    pos: Position,
}

impl Tokenizer {
    pub fn new(target: &str) -> Tokenizer {
        return Tokenizer {
            target: target.to_string(),
            pos: Position::new(1, 0, 0),
        };
    }

    fn is_eof(&self) -> bool {
        return self.pos.at_whole >= self.target.len() as u32;
    }

    fn move_horizon(&mut self, n: u32) {
        self.pos.at_line += n;
        self.pos.at_whole += n;
    }

    fn next_line(&mut self) {
        self.pos.at_whole += 1;
        self.pos.line_no += 1;
        self.pos.at_line = 0;
    }

    fn current_char(&self) -> char {
        return self.target.chars().nth(self.pos.at_whole as usize).unwrap();
    }

    fn peek(&self, n: u32) -> char {
        return self
            .target
            .chars()
            .nth((self.pos.at_whole + n) as usize)
            .unwrap();
    }

    fn start_with(&self, word: String) -> bool {
        let chars: Chars = word.chars();
        for (i, c) in chars.enumerate() {
            if self.peek(i as u32) != c {
                return false;
            }
        }
        return true;
    }

    fn consume_string(&mut self, is_single: bool) -> String {
        let mut s: String = "".to_string();

        // consume start single/double quotation
        self.move_horizon(1);

        while !self.is_eof() {
            let cur = self.current_char();
            if cur == '\'' && is_single {
                break;
            }
            if cur == '"' && !is_single {
                break;
            }
            s += &*cur.to_string();
            self.move_horizon(1);
        }

        // consume end single/double quotation
        self.move_horizon(1);

        return s;
    }

    fn consume_numeric(&mut self) -> (f64, bool) {
        let mut s: String = "".to_string();
        let mut include_dot: bool = false;

        while !self.is_eof() {
            if is_number(self.current_char()) {
                s += &*self.current_char().to_string()
            } else if self.current_char() == '.' {
                s += &*self.current_char().to_string();
                include_dot = true;
            } else {
                break;
            }
            self.move_horizon(1);
        }

        return (s.parse().unwrap(), include_dot);
    }

    fn consume_ws(&mut self) -> String {
        let mut s: String = "".to_string();

        while !self.is_eof() {
            if is_ws(self.current_char()) && self.current_char() != '\n' {
                s += &*self.current_char().to_string();
                self.move_horizon(1);
            } else if self.current_char() == '\n' {
                s += &*self.current_char().to_string();
                self.next_line();
            } else {
                break;
            }
        }

        return s;
    }

    fn consume_symbol(&mut self) -> String {
        let s: String = self.current_char().to_string();
        self.move_horizon(1);
        return s;
    }

    fn consume_text(&mut self) -> String {
        let mut s: String = "".to_string();

        if !is_alphanum_(self.current_char()) {
            s = self.current_char().to_string();
            self.move_horizon(1);
            return s;
        }

        while !self.is_eof() {
            if is_alphanum_(self.current_char()) {
                s += &*self.current_char().to_string();
                self.move_horizon(1);
            } else {
                break;
            }
        }

        return s;
    }

    fn link_ws_token<'a>(&self, cur: &'a mut Token, pos: Position) -> &'a mut Box<Token> {
        let tok: Token = Token::new(Whitespace, pos);
        cur.next = Some(Box::from(tok.clone()));
        return cur.next.as_mut().unwrap();
    }

    fn link_symbol_token<'a>(&self, cur: &'a mut Token, pos: Position, symbol: String) -> &'a mut Box<Token> {
        let tok: Token = Token::new(ReservedSymbol(symbol), pos);
        cur.next = Some(Box::from(tok.clone()));
        return cur.next.as_mut().unwrap();
    }

    fn link_decimal_token<'a>(&self, cur: &'a mut Token, pos: Position, f: f64) -> &'a mut Box<Token> {
        let tok: Token = Token::new(Decimal(f), pos);
        cur.next = Some(Box::from(tok.clone()));
        return cur.next.as_mut().unwrap();
    }

    fn link_integer_token<'a>(&self, cur: &'a mut Token, pos: Position, i: i64) -> &'a mut Box<Token> {
        let tok: Token = Token::new(Integer(i), pos);
        cur.next = Some(Box::from(tok.clone()));
        return cur.next.as_mut().unwrap();
    }

    fn link_string_token<'a>(&self, cur: &'a mut Token, pos: Position, s: String) -> &'a mut Box<Token> {
        let tok: Token = Token::new(TokenKind::String(s), pos);
        cur.next = Some(Box::from(tok.clone()));
        return cur.next.as_mut().unwrap();
    }

    fn link_text_token<'a>(&self, cur: &'a mut Token, pos: Position, s: String) -> &'a mut Box<Token> {
        let tok: Token = Token::new(Text(s), pos);
        cur.next = Some(Box::from(tok.clone()));
        return cur.next.as_mut().unwrap();
    }

    fn link_eof_token<'a>(&self, cur: &'a mut Token, pos: Position) -> &'a mut Box<Token> {
        let tok: Token = Token::new(Eof, pos);
        cur.next = Some(Box::from(tok.clone()));
        return cur.next.as_mut().unwrap();
    }

    fn tokenize(&mut self) -> Option<Box<Token>> {
        let mut head = Token::new(TokenKind::Illegal, self.pos.clone());
        let mut cur= &mut head;

        while !self.is_eof() {
            if is_ws(self.current_char()) {
                let _ws = self.consume_ws();
                cur = self.link_ws_token(cur, self.pos.clone());
                continue;
            }

            if is_reserved_symbol(self.current_char()) {
                let sym = self.consume_symbol();
                cur = self.link_symbol_token(cur, self.pos.clone(), sym);
                continue;
            }

            if self.current_char() == '\'' {
                let s = self.consume_string(true);
                cur = self.link_string_token(cur, self.pos.clone(), s);
                continue
            } else if self.current_char() == '"' {
                let s = self.consume_string(false);
                cur = self.link_string_token(cur, self.pos.clone(), s);
                continue
            }

            if is_number(self.current_char()) {
                let (f, include_dot) = self.consume_numeric();
                if include_dot {
                    cur = self.link_decimal_token(cur, self.pos.clone(), f);
                } else {
                    cur = self.link_integer_token(cur, self.pos.clone(), f as i64);
                }
                continue;
            }

            let t = self.consume_text();
            cur = self.link_text_token(cur, self.pos.clone(), t);
            continue;
        }

        let _cur = self.link_eof_token(cur, self.pos.clone());
        return head.next;
    }
}

#[cfg(test)]
mod tests {
    use crate::html::tokenizer::Tokenizer;
    #[test]
    fn tokenize() {
        let input = "<h1>hello, world</h1>";
        let mut tokenizer = Tokenizer::new(input);
        let token = tokenizer.tokenize();
        println!("{:#?}", token)
    }
}