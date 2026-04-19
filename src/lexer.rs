#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Keywords
    Val,
    Mut,
    Fn,
    Pub,
    Type,
    Union,
    Enum,
    If,
    Elif,
    Else,
    For,
    In,
    Return,
    Break,
    Match,
    Some,
    None,
    Ok,
    Err,
    Use,
    Trust,
    Assert,
    When,
    Test,
    Extern,
    Pre,
    Post,
    Guarded,
    Invariant,
    Linux,
    Darwin,
    Windows,
    Posix,
    As,
    Ident(String),
    Number(String),
    String(String),
    RawString(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Semicolon,
    Comma,
    Dot,
    Arrow,     // =>
    Question,  // ?
    Equal,     // =
    EqualEq,   // ==
    NotEq,     // !=
    Lt,        // <
    Gt,        // >
    LtEq,      // <=
    GtEq,      // >=
    LtLt,      // <<
    GtGt,      // >>
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    Bang,
    Tilde,
    Caret,
    AmpAmp,    // &&
    PipePipe,  // ||
    PlusEq,    // +=
    MinusEq,   // -=
    StarEq,    // *=
    SlashEq,   // /=
    At,        // @ (builtin prefix)

    // Comments
    Comment(String),

    // Special
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        if self.pos + offset < self.input.len() {
            Some(self.input[self.pos + offset])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self, quote: char) -> String {
        let mut result = String::new();
        self.advance(); // skip opening quote

        while let Some(ch) = self.current() {
            if ch == quote {
                self.advance();
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current() {
                    result.push('\\');
                    result.push(escaped);
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        result
    }

    fn read_raw_string(&mut self) -> String {
        let mut result = String::new();
        self.advance(); // skip first \
        self.advance(); // skip second \

        while let Some(ch) = self.current() {
            if ch == '\\' && self.peek(1) == Some('\\') {
                // End of raw string
                self.advance();
                self.advance();
                break;
            } else {
                result.push(ch);
                self.advance();
            }
        }

        result
    }

    fn read_ident(&mut self) -> String {
        let mut result = String::new();

        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        result
    }

    fn read_number(&mut self) -> String {
        let mut result = String::new();

        if self.current() == Some('0') && (self.peek(1) == Some('x') || self.peek(1) == Some('X')) {
            result.push('0');
            self.advance();
            result.push(self.current().unwrap());
            self.advance();
            while let Some(ch) = self.current() {
                if ch.is_ascii_hexdigit() || ch == '_' {
                    result.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            return result;
        }

        while let Some(ch) = self.current() {
            if ch.is_numeric() || ch == '.' || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        result
    }

    fn read_comment(&mut self) -> String {
        let mut result = String::new();
        self.advance(); // skip first /
        self.advance(); // skip second /

        while let Some(ch) = self.current() {
            if ch == '\n' {
                break;
            }
            result.push(ch);
            self.advance();
        }

        result
    }

    fn keyword_or_ident(ident: &str) -> Token {
        match ident {
            "val" => Token::Val,
            "mut" => Token::Mut,
            "fn" => Token::Fn,
            "pub" => Token::Pub,
            "type" => Token::Type,
            "union" => Token::Union,
            "enum" => Token::Enum,
            "if" => Token::If,
            "elif" => Token::Elif,
            "else" => Token::Else,
            "for" => Token::For,
            "in" => Token::In,
            "return" => Token::Return,
            "break" => Token::Break,
            "match" => Token::Match,
            "some" => Token::Some,
            "none" => Token::None,
            "ok" => Token::Ok,
            "err" => Token::Err,
            "use" => Token::Use,
            "trust" => Token::Trust,
            "assert" => Token::Assert,
            "when" => Token::When,
            "test" => Token::Test,
            "extern" => Token::Extern,
            "pre" => Token::Pre,
            "post" => Token::Post,
            "guarded" => Token::Guarded,
            "invariant" => Token::Invariant,
            "linux" => Token::Linux,
            "darwin" => Token::Darwin,
            "windows" => Token::Windows,
            "posix" => Token::Posix,
            "as" => Token::As,
            _ => Token::Ident(ident.to_string()),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current() {
            None => Token::Eof,
            Some('(') => {
                self.advance();
                Token::LParen
            }
            Some(')') => {
                self.advance();
                Token::RParen
            }
            Some('{') => {
                self.advance();
                Token::LBrace
            }
            Some('}') => {
                self.advance();
                Token::RBrace
            }
            Some('[') => {
                self.advance();
                Token::LBracket
            }
            Some(']') => {
                self.advance();
                Token::RBracket
            }
            Some(':') => {
                self.advance();
                Token::Colon
            }
            Some(';') => {
                self.advance();
                Token::Semicolon
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some('.') => {
                self.advance();
                Token::Dot
            }
            Some('?') => {
                self.advance();
                Token::Question
            }
            Some('@') => {
                self.advance();
                Token::At
            }
            Some('=') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::EqualEq
                } else if self.current() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Equal
                }
            }
            Some('!') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::NotEq
                } else {
                    Token::Bang
                }
            }
            Some('<') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::LtEq
                } else if self.current() == Some('<') {
                    self.advance();
                    Token::LtLt
                } else {
                    Token::Lt
                }
            }
            Some('>') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::GtEq
                } else if self.current() == Some('>') {
                    self.advance();
                    Token::GtGt
                } else {
                    Token::Gt
                }
            }
            Some('+') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::PlusEq
                } else {
                    Token::Plus
                }
            }
            Some('-') => {
                self.advance();
                Token::Minus
            }
            Some('*') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::StarEq
                } else {
                    Token::Star
                }
            }
            Some('/') => {
                if self.peek(1) == Some('/') {
                    let comment = self.read_comment();
                    Token::Comment(comment)
                } else {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        Token::SlashEq
                    } else {
                        Token::Slash
                    }
                }
            }
            Some('%') => {
                self.advance();
                Token::Percent
            }
            Some('&') => {
                self.advance();
                if self.current() == Some('&') {
                    self.advance();
                    Token::AmpAmp
                } else {
                    Token::Ampersand
                }
            }
            Some('|') => {
                self.advance();
                if self.current() == Some('|') {
                    self.advance();
                    Token::PipePipe
                } else {
                    Token::Pipe
                }
            }
            Some('^') => {
                self.advance();
                Token::Caret
            }
            Some('~') => {
                self.advance();
                Token::Tilde
            }
            Some('"') => {
                let s = self.read_string('"');
                Token::String(s)
            }
            Some('\'') => {
                let s = self.read_string('\'');
                Token::String(s)
            }
            Some('\\') if self.peek(1) == Some('\\') => {
                let raw = self.read_raw_string();
                Token::RawString(raw)
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_ident();
                Lexer::keyword_or_ident(&ident)
            }
            Some(ch) if ch.is_numeric() => {
                let num = self.read_number();
                Token::Number(num)
            }
            Some(ch) => {
                self.advance();
                Token::Ident(ch.to_string())
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("val mut fn pub type");
        assert_eq!(lexer.next_token(), Token::Val);
        assert_eq!(lexer.next_token(), Token::Mut);
        assert_eq!(lexer.next_token(), Token::Fn);
        assert_eq!(lexer.next_token(), Token::Pub);
        assert_eq!(lexer.next_token(), Token::Type);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("= == != < > <= >= + - * / %");
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::EqualEq);
        assert_eq!(lexer.next_token(), Token::NotEq);
        assert_eq!(lexer.next_token(), Token::Lt);
        assert_eq!(lexer.next_token(), Token::Gt);
        assert_eq!(lexer.next_token(), Token::LtEq);
        assert_eq!(lexer.next_token(), Token::GtEq);
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Star);
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::Percent);
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello""#);
        if let Token::String(s) = lexer.next_token() {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string token");
        }
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("// this is a comment\nval x");
        if let Token::Comment(c) = lexer.next_token() {
            assert_eq!(c, " this is a comment");
        } else {
            panic!("Expected comment token");
        }
        assert_eq!(lexer.next_token(), Token::Val);
    }
}
