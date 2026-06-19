#[derive(Clone, Debug, PartialEq)]
pub enum PengToken {
    Nil,
    Int,
    Uint,
    Float32,
    Float64,
    String,
    Byte,

    Type,

    As,
    Is,

    Equals,
    Dot,
    Comma,
    Colon,
    Semicolon,

    RightParenthesis,
    LeftParenthesis,
    RightCurlyBrace,
    LeftCurlyBrace,
    RightBracket,
    LeftBracket,

    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    DoubleEquals,
    ExclamationEquals,

    Plus,
    Minus,
    Asterisk,
    DoubleAsterisk,
    Slash,
    Percent,

    PlusEquals,
    MinusEquals,
    AsteriskEquals,
    DoubleAsteriskEquals,
    SlashEquals,
    PercentEquals,

    DoubleDot,

    TripleDot,

    Ampersand,
    DoubleAmpersand,
    Pipe,
    DoublePipe,

    Exclamation,
    Arrow,

    Var,
    Func,
    Mod,
    Match,
    Return,
    If,
    Else,

    For,
    Each,
    While,
    Loop,
    In,

    Break,
    Continue,

    Identifier(String),

    // ( all numbers discards '_' ; needs to start with numbers (no underline at start))
    NumberLiteral(String), // used for generic numbers :: 1, 10, 2138, 1_00_0 (1000 (discards '_'))
    IntLiteral(isize), // 1i, 2i 4_0i (i suffix) (discards '_')
    UintLiteral(usize), // 10u suffix u
    ByteLiteral(u8), // 10b
    // dotted numbers are f32 by default
    Float32Literal(f32), // 1_._0 (1.0), 3.14, 3.33f32 (f32 suffix), 1.0__f32
    Float64Literal(f64), // the same as f32, but with f64 suffix

    StringLiteral(String),

    True,
    False,
}

impl PengToken {
    pub fn from_string(source: String) -> Result<Self, String> {

        let fixed = match source.as_str() {
            "nil" => Some(Self::Nil),
            "int" => Some(Self::Int),
            "uint" => Some(Self::Uint),
        
            "f32" => Some(Self::Float32),
            "f64" => Some(Self::Float64),

            "string" => Some(Self::String),
            "byte" => Some(Self::Byte),

            "type" => Some(Self::Type),

            "=" => Some(Self::Equals),
            "." => Some(Self::Dot),
            ":" => Some(Self::Colon),
            "," => Some(Self::Comma),
            ";" => Some(Self::Semicolon),

            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "*" => Some(Self::Asterisk),
            "**" => Some(Self::DoubleAsterisk),
            "/" => Some(Self::Slash),
            "%" => Some(Self::Percent),

            "+=" => Some(Self::PlusEquals),
            "-=" => Some(Self::MinusEquals),
            "*=" => Some(Self::AsteriskEquals),
            "**=" => Some(Self::DoubleAsteriskEquals),
            "/=" => Some(Self::SlashEquals),
            "%=" => Some(Self::PercentEquals),

            ".." => Some(Self::DoubleDot),

            "..." => Some(Self::TripleDot),

            "&" => Some(Self::Ampersand),
            "&&" => Some(Self::DoubleAmpersand),
            "|" => Some(Self::Pipe),
            "||" => Some(Self::DoublePipe),

            "!" => Some(Self::Exclamation),
            "->" => Some(Self::Arrow),

            "<" => Some(Self::LessThan),
            ">" => Some(Self::GreaterThan),
            "<=" => Some(Self::LessThanEquals),
            ">=" => Some(Self::GreaterThanEquals),

            "==" => Some(Self::DoubleEquals),
            "!=" => Some(Self::ExclamationEquals),

            "(" => Some(Self::LeftParenthesis),
            ")" => Some(Self::RightParenthesis),
            "{" => Some(Self::LeftCurlyBrace),
            "}" => Some(Self::RightCurlyBrace),
            "[" => Some(Self::LeftBracket),
            "]" => Some(Self::RightBracket),

            "var" => Some(Self::Var),
            "func" => Some(Self::Func),
            "mod" => Some(Self::Mod),
            "match" => Some(Self::Match),
            "return" => Some(Self::Return),
            "if" => Some(Self::If),
            "else" => Some(Self::Else),

            "as" => Some(Self::As),
            "is" => Some(Self::Is),

            "for" => Some(Self::For),
            "each" => Some(Self::Each),
            "while" => Some(Self::While),
            "loop" => Some(Self::Loop),
            "in" => Some(Self::In),

            "break" => Some(Self::Break),
            "continue" => Some(Self::Continue),

            "true" => Some(Self::True),
            "false" => Some(Self::False),

            _ => None,
        };

        if let Some(token) = fixed {
            return Ok(token);
        }

        if let Some(number_token) = Self::parse_number_literal(&source)? {
            return Ok(number_token);
        }

        Ok(Self::Identifier(source))
    }

    fn parse_number_literal(source: &str) -> Result<Option<Self>, String> {
        if !source.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Ok(None);
        }

        let clean = source.replace('_', "");

        if clean.is_empty() {
            return Err(format!("invalid number literal: {}", source));
        }

        if let Some(body) = clean.strip_suffix("f64") {
            return Self::parse_float64(body, source).map(Some);
        }

        if let Some(body) = clean.strip_suffix("f32") {
            return Self::parse_float32(body, source).map(Some);
        }

        if clean.contains('.') {
            return Self::parse_float32(&clean, source).map(Some);
        }

        if let Some(body) = clean.strip_suffix('i') {
            return Self::parse_int::<isize>(body, source).map(Self::IntLiteral).map(Some);
        }

        if let Some(body) = clean.strip_suffix('u') {
            return Self::parse_int::<usize>(body, source).map(Self::UintLiteral).map(Some);
        }

        if let Some(body) = clean.strip_suffix('b') {
            let value = Self::parse_int::<u8>(body, source)?;

            return Ok(Some(Self::ByteLiteral(value)));
        }

        if Self::is_digits(&clean) {
            return Ok(Some(Self::NumberLiteral(clean)));
        }

        Err(format!("invalid number literal: {}", source))
    }

    fn parse_int<T>(body: &str, original: &str) -> Result<T, String>
    where
        T: std::str::FromStr,
    {
        if !Self::is_digits(body) {
            return Err(format!("invalid number literal: {}", original));
        }

        body.parse::<T>()
            .map_err(|_| format!("invalid number literal: {}", original))
    }

    fn parse_float32(body: &str, original: &str) -> Result<Self, String> {
        if !Self::is_float_body(body) {
            return Err(format!("invalid number literal: {}", original));
        }

        body.parse::<f32>()
            .map(Self::Float32Literal)
            .map_err(|_| format!("invalid number literal: {}", original))
    }

    fn parse_float64(body: &str, original: &str) -> Result<Self, String> {
        if !Self::is_float_body(body) {
            return Err(format!("invalid number literal: {}", original));
        }

        body.parse::<f64>()
            .map(Self::Float64Literal)
            .map_err(|_| format!("invalid number literal: {}", original))
    }

    fn is_digits(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
    }

    fn is_float_body(s: &str) -> bool {
        let Some((left, right)) = s.split_once('.') else {
            return false;
        };

        Self::is_digits(left) && right.chars().all(|c| c.is_ascii_digit())
    }

    pub fn new_string(string: String) -> Self {
        Self::StringLiteral(string)
    }
}