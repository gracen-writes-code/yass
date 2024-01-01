use std::any::Any;

use super::{value, value::Value, VM};

const MAX_INTERPOLATION_NESTING: usize = 8;

enum TokenType {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Colon,
    Dot,
    DotDot,
    DotDotDot,
    Comma,
    Star,
    Slash,
    Percent,
    Hash,
    Plus,
    Minus,
    LTLT,
    GTGT,
    Pipe,
    PipePipe,
    Caret,
    Amp,
    AmpAmp,
    Bang,
    Tilde,
    Question,
    EQ,
    LT,
    GT,
    LTEQ,
    GTEQ,
    EQEQ,
    BangEQ,

    Break,
    Continue,
    Class,
    Construct,
    Else,
    False,
    For,
    Foreign,
    If,
    Import,
    As,
    In,
    Is,
    Null,
    Return,
    Static,
    Super,
    This,
    True,
    Var,
    While,

    Field,
    StaticField,
    Name,
    Number,

    String,

    Interpolation,

    Line,

    Error,
    EOF,
}

struct Token<'a> {
    tok_type: TokenType,
    source: &'a [u8],
    start_idx: usize,
    length: usize,
    line: i32,
    value: Option<Value>,
}

impl Token<'_> {
    fn null<'a>(source: &'a [u8]) -> Self {
        Token {
            source,
            tok_type: TokenType::Error,
            start_idx: 0,
            length: 0,
            line: 0,
            value: None,
        }
    }
}

struct Parser<'a> {
    vm: &'a VM,
    module: &'a ObjModule,
    source: &'a [u8],
    token_start: usize,
    current_char: usize,
    current_line: i32,

    next: Token<'a>,
    current: Token<'a>,
    previous: Token<'a>,

    parens: [i32; MAX_INTERPOLATION_NESTING],
    num_parens: usize,

    print_errors: bool,
    has_error: bool,
}

impl Parser<'_> {
    fn new<'a>(vm: &VM, module: &ObjModule, source: &[u8], print_errors: bool) -> Self {
        Self {
            vm,
            module,
            source,
            token_start: 0,
            current_char: 0,
            current_line: 1,
            parens: [0; MAX_INTERPOLATION_NESTING],
            num_parens: 0,
            next: Token::null(source),
            current: Token::null(source),
            previous: Token::null(source),
            print_errors,
            has_error: false,
        }
    }

    fn peek_char(&self) -> u8 {
        self.source[self.current_char]
    }

    fn next_char(&self) -> u8 {
        let c = self.peek_char();
        self.current_char += 1;
        if c == b'\n' {
            self.current_line += 1;
        }

        c
    }

    fn make_token(&self, tok_type: TokenType) {
        self.next.tok_type = tok_type;
        self.next.start_idx = self.token_start;
        self.next.length = self.current_char - self.token_start;
        self.next.line = self.current_line;

        if matches!(tok_type, TokenType::Line) {
            self.next.line -= 1;
        }
    }

    fn next_token(&self) {
        self.previous = self.current;
        self.current = self.next;

        if matches!(self.next.tok_type, TokenType::EOF)
            || matches!(self.current.tok_type, TokenType::EOF)
        {
            return;
        }

        while self.peek_char() != b'\0' {
            self.token_start = self.current_char;

            let c = self.next_char();
            match c {
                b'(' => {
                    if self.num_parens > 0 {
                        self.parens[self.num_parens - 1] += 1;
                    }
                    self.make_token(TokenType::LeftParen);
                }
                b')' => {
                    if self.num_parens > 0 {
                        self.parens[self.num_parens - 1] -= 1;
                        if self.parens[self.num_parens - 1] == 0 {
                            self.num_parens -= 1;
                            self.read_string();
                            return;
                        }
                    }

                    self.make_token(TokenType::RightParen);
                }
                b'[' => self.make_token(TokenType::LeftBracket),
                b']' => self.make_token(TokenType::RightBracket),
                b'{' => self.make_token(TokenType::LeftBrace),
                b'}' => self.make_token(TokenType::RightBrace),
                b':' => self.make_token(TokenType::Colon),
                b',' => self.make_token(TokenType::Comma),
                b'*' => self.make_token(TokenType::Star),
                b'%' => self.make_token(TokenType::Percent),
                b'#' => {
                    if self.current_line == 1
                        && self.peek_char() == b'!'
                        && self.peek_next_char() == b'/'
                    {
                        self.skip_line_comment();
                        continue;
                    }

                    self.make_token(TokenType::Hash);
                }
                b'^' => self.make_token(TokenType::Caret),
                b'+' => self.make_token(TokenType::Plus),
                b'-' => self.make_token(TokenType::Minus),
                b'~' => self.make_token(TokenType::Tilde),
                b'?' => self.make_token(TokenType::Question),

                b'|' => self.two_char_token(b'|', TokenType::PipePipe, TokenType::Pipe),
                b'&' => self.two_char_token(b'&', TokenType::AmpAmp, TokenType::Amp),
                b'=' => self.two_char_token(b'=', TokenType::EQEQ, TokenType::EQ),

                b'.' => {
                    if self.match_char(b'.') {
                        self.two_char_token(b'.', TokenType::DotDotDot, TokenType::DotDot);
                        return;
                    }

                    self.make_token(TokenType::Dot);
                }
                b'/' => {
                    if self.match_char(b'/') {
                        self.skip_line_comment();
                        continue;
                    }

                    if self.match_char(b'*') {
                        self.skip_block_comment();
                        continue;
                    }

                    self.make_token(TokenType::Slash);
                }
                b'<' => {
                    if self.match_char(b'<') {
                        self.make_token(TokenType::LTLT);
                    } else {
                        self.two_char_token(b'=', TokenType::LTEQ, TokenType::LT);
                    }
                }
                b'>' => {
                    if self.match_char(b'>') {
                        self.make_token(TokenType::GTGT);
                    } else {
                        self.two_char_token(b'=', TokenType::GTEQ, TokenType::GT);
                    }
                }
                b'\n' => self.make_token(TokenType::Line),

                b' ' | b'\r' | b'\t' => {
                    while self.peek_char() == b' '
                        || self.peek_char() == b'\r'
                        || self.peek_char() == b'\t'
                    {
                        self.next_char();
                    }
                    continue;
                }

                b'"' => {
                    if self.peek_char() == b'"' && self.peek_next_char() == b'"' {
                        self.read_raw_string();
                        return;
                    }
                    self.read_string();
                }
                b'_' => self.read_name(
                    if self.peek_char() == b'_' {
                        TokenType::StaticField
                    } else {
                        TokenType::Field
                    },
                    c,
                ),

                b'0' => {
                    if self.peek_char() == b'x' {
                        self.read_hex_number();
                        return;
                    }

                    self.read_number();
                }
                _ => {
                    if is_name(c) {
                        self.read_name(TokenType::Name, c);
                    } else if is_digit(c) {
                        self.read_number();
                    } else {
                        if c >= 32 && c <= 126 {
                            let c_char = c as char;
                            self.lex_error(format!("Invalid character: '{c}'."));
                        } else {
                            self.lex_error(format!("Invalid byte 0x{c}."));
                        }
                        self.next.tok_type = TokenType::Error;
                        self.next.length = 0;
                    }
                }
            };
            return;
        }

        self.token_start = self.current_char;
        self.make_token(TokenType::EOF);
    }
}

struct Compiler {}

impl Compiler {
    fn new(parser: &Parser, parent: Option<&Compiler>, is_method: bool) -> Self {
        todo!() // initCompiler [wren_compiler.c]
    }

    fn match_token(&self, expected: TokenType) -> bool {
        if self.peek() != expected {
            return false;
        }

        self.parser.next_token();
        return true;
    }

    fn match_line(&self) -> bool {
        if !self.match_token(TokenType::Line) {
            return false;
        }

        while self.match_token(TokenType::Line) {}
        return true;
    }

    fn ignore_newlines(&self) {
        self.match_line();
    }
}

impl super::VM {
    pub(super) fn compile(
        &self,
        module: &ObjModule,
        source: &[u8],
        is_expression: bool,
        print_errors: bool,
    ) -> ObjFn {
        source.starts_with("\u{feff}".as_bytes());

        let parser = Parser::new(self, module, source, print_errors);

        parser.next_token();
        parser.next_token();

        let num_existing_variables = module.variables.count;

        let compiler = Compiler::new(&parser, None, false);
        compiler.ignore_newlines();

        if is_expression {
            compiler.expression();
            compiler.consume(TokenType::EOF, "Expect end of expression.");
        } else {
            while !compiler.match_token(TokenType::EOF) {
                compiler.definition();

                if !compiler.match_line() {
                    compiler.consume(TokenType::EOF, "Expect end of file.");
                    break;
                }
            }

            compiler.emit_op(Code::EndModule);
        }

        compiler.emit_op(Code::Return);

        for i in num_existing_variables..parser.module.variables.count {
            if value::is_num(parser.module.variables.data[i]) {
                parser.previous.tok_type = TokenType::Name;
                parser.previous.start = parser.module.variable_names.data[i].value; // TODO fix this to make it start_idx
                parser.previous.length = parser.module.variable_names.data[i].length;
                parser.previous.line = value::as_num(parser.module.varaibles.data[i]);
                compiler.error("Variable is used but not defined.");
            }
        }

        compiler.end_compiler("(script)", 8)
    }
}
