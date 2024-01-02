use super::{utils, value, value::Value, ErrorType, VM};

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

    fn peek_next_char(&self) -> u8 {
        if self.peek_char() == b'\0' {
            b'0'
        } else {
            self.source[self.current_char + 1]
        }
    }

    fn next_char(&self) -> u8 {
        let c = self.peek_char();
        self.current_char += 1;
        if c == b'\n' {
            self.current_line += 1;
        }

        c
    }

    fn match_char(&self, c: u8) -> bool {
        if self.peek_char() != c {
            false
        } else {
            self.next_char();
            true
        }
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

    fn two_char_token(&self, c: u8, two: TokenType, one: TokenType) {
        self.make_token(if self.match_char(c) { two } else { one })
    }

    fn skip_line_comment(&self) {
        while self.peek_char() != b'\n' && self.peek_char() != b'\0' {
            self.next_char();
        }
    }

    fn skip_block_comment(&self) {
        let mut nesting = 1;
        while nesting > 0 {
            if self.peek_char() == b'\0' {
                self.lex_error("Unterminated block comment.");
                return;
            }

            if self.peek_char() == b'/' && self.peek_next_char() == b'*' {
                self.next_char();
                self.next_char();
                nesting += 1;
                continue;
            }

            if self.peek_char() == b'*' && self.peek_next_char() == b'/' {
                self.next_char();
                self.next_char();
                nesting -= 1;
                continue;
            }

            self.next_char();
        }
    }

    fn read_raw_string(&self) {
        let tok_type = TokenType::String;
        let string = utils::Buffer::<u8>::new();

        self.next_char();
        self.next_char();

        let mut skip_start = 0;
        let mut first_newline = -1;

        let mut skip_end = -1;
        let mut last_newline = -1;

        loop {
            let c = self.next_char();
            let c1 = self.peek_char();
            let c2 = self.peek_next_char();

            if c == b'\r' {
                continue;
            }

            if c == b'\n' {
                last_newline = string.count;
                skip_end = last_newline;
                first_newline = if first_newline == -1 {
                    string.count
                } else {
                    first_newline
                };
            }

            if c == b'"' && c1 == b'"' && c2 == b'"' {
                break;
            }

            let is_whitespace = c == b' ' || c == b'\t';
            skip_end = if c == b'\n' || is_whitespace {
                skip_end
            } else {
                -1
            };

            let skippable = skip_start != 1 && is_whitespace && first_newline == -1;
            skip_start = if skippable {
                string.count + 1
            } else {
                skip_start
            };

            if first_newline == -1 && !is_whitespace && c != b'\n' {
                skip_start = -1;
            }

            if c == b'\0' || c1 == b'\0' || c2 == b'\0' {
                self.lex_error("Unterminated raw string.");

                self.current_char -= 1;
                break;
            }

            string.write(self.vm, c);
        }

        self.next_char();
        self.next_char();

        let mut offset = 0;
        let mut count = string.count;

        if first_newline != -1 && skip_start == first_newline {
            offset = first_newline + 1;
        }
        if last_newline != -1 && skip_end == last_newline {
            count = last_newline;
        }

        count -= if offset > count { count } else { offset };

        self.next.value = self
            .vm
            .new_string_length(string.data.unwrap()[(count as usize)..], count);

        string.clear(self.vm);
        self.make_token(tok_type);
    }

    fn read_string(&self) {
        let tok_type = TokenType::String;
        let string = utils::Buffer::<u8>::new();

        loop {
            let c = self.next_char();
            if c == b'"' {
                break;
            }
            if c == b'\r' {
                continue;
            }

            if c == b'\0' {
                self.lex_error("Unterminated string.");

                self.current_char -= 1;
                break;
            }

            if c == b'%' {
                if self.num_parens < MAX_INTERPOLATION_NESTING {
                    if self.next_char() != b'(' {
                        self.lex_error("Expect '(' after '%'.");
                    }

                    self.num_parens += 1;
                    self.parens[self.num_parens] = 1;
                    tok_type = TokenType::Interpolation;
                    break;
                }

                self.lex_error(format!(
                    "Interpolation may only nest {MAX_INTERPOLATION_NESTING} levels deep."
                ));
            }

            if c == b'\\' {
                match self.next_char() {
                    b'"' => string.write(self.vm, b'"'),
                    b'\\' => string.write(self.vm, b'\\'),
                    b'%' => string.write(self.vm, b'%'),
                    b'0' => string.write(self.vm, b'\0'),
                    b'a' => string.write(self.vm, b'\x07'),
                    b'b' => string.write(self.vm, b'\x08'),
                    b'e' => string.write(self.vm, b'\x1B'),
                    b'f' => string.write(self.vm, b'\x0C'),
                    b'n' => string.write(self.vm, b'\n'),
                    b'r' => string.write(self.vm, b'\r'),
                    b't' => string.write(self.vm, b'\t'),
                    b'u' => self.read_unicode_escape(&string, 4),
                    b'U' => self.read_unicode_escape(&string, 8),
                    b'v' => string.write(self.vm, b'\x0B'),
                    b'x' => string.write(self.vm, self.read_hex_escape(2, "byte") as u8),
                    invalid_byte => {
                        let invalid_char = invalid_byte as char;
                        self.lex_error(format!("Invalid escape character '{invalid_char}'"));
                    }
                }
            } else {
                string.write(self.vm, c);
            }
        }

        self.next.value = self
            .vm
            .new_string_length(string.data.unwrap(), string.count);

        string.clear(self.vm);
        self.make_token(tok_type);
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

    fn print_error(&self, line: i32, label: String, msg: String) {
        self.has_error = true;
        if !self.print_errors {
            return;
        }

        if self.vm.config.error_fn.is_none() {
            return;
        }

        let module = self.module.name;
        let module_name = if module { module.value } else { "<unknown>" }; // TODO make the if condition here real

        self.vm.config.error_fn.unwrap()(
            self.vm,
            ErrorType::WrenErrorCompile,
            module_name,
            line,
            format!("{label}: {msg}"),
        );
    }

    fn lex_error<S: Into<String>>(&self, msg: S) {
        self.print_error(self.current_line, "Error".into(), msg.into());
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
