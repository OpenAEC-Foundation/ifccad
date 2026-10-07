use super::*;

/// TEXT is a separate source grammar: braces and ordinary backslashes are
/// literal, not MTEXT scopes or font/color controls.
pub fn lex_text(
    source: &str,
    limits: MarkupLimits,
) -> Result<Vec<LocatedMarkupToken>, TextMarkupError> {
    if source.len() > limits.max_bytes {
        return Err(TextMarkupError::new(
            TextMarkupErrorCode::ResourceLimit,
            0,
            "input exceeds byte budget",
        ));
    }
    let mut lexer = Lexer {
        source,
        pos: 0,
        scopes: Vec::new(),
        tokens: Vec::new(),
    };
    while lexer.pos < source.len() {
        let start = lexer.pos;
        let c = lexer.take().unwrap();
        let kind = match c {
            '%' => lexer.percent(start)?,
            '\\' if lexer.source[lexer.pos..].starts_with("U+") => {
                lexer.pos += 2;
                lexer.unicode(start)?
            }
            '\\' if matches!(lexer.peek(), Some('P' | 'p')) => {
                lexer.take();
                MarkupTokenKind::ParagraphBreak
            }
            '\\' => MarkupTokenKind::Literal("\\".into()),
            c if forbidden_literal(c) => {
                return Err(lexer.error(
                    TextMarkupErrorCode::StructuralControl,
                    start,
                    "Text source contains structural controls",
                ))
            }
            _ => {
                while lexer
                    .peek()
                    .is_some_and(|c| !matches!(c, '%' | '\\') && !forbidden_literal(c))
                {
                    lexer.take();
                }
                MarkupTokenKind::Literal(source[start..lexer.pos].into())
            }
        };
        if lexer.tokens.len() == limits.max_tokens {
            return Err(lexer.error(
                TextMarkupErrorCode::ResourceLimit,
                start,
                "token count exceeds budget",
            ));
        }
        lexer.tokens.push(LocatedMarkupToken {
            kind,
            range: start..lexer.pos,
        });
    }
    Ok(lexer.tokens)
}

pub fn lex(source: &str, limits: MarkupLimits) -> Result<Vec<LocatedMarkupToken>, TextMarkupError> {
    if source.len() > limits.max_bytes {
        return Err(TextMarkupError::new(
            TextMarkupErrorCode::ResourceLimit,
            0,
            "input exceeds byte budget",
        ));
    }
    let mut lexer = Lexer {
        source,
        pos: 0,
        scopes: Vec::new(),
        tokens: Vec::new(),
    };
    while lexer.pos < source.len() {
        let start = lexer.pos;
        let c = lexer
            .take()
            .expect("pos is a valid nonempty UTF-8 boundary");
        let token = match c {
            '{' => {
                if lexer.scopes.len() == limits.max_depth {
                    return Err(lexer.error(
                        TextMarkupErrorCode::ResourceLimit,
                        start,
                        "scope depth exceeds budget",
                    ));
                }
                lexer.scopes.push(start);
                MarkupTokenKind::ScopeStart
            }
            '}' => {
                lexer.scopes.pop().ok_or_else(|| {
                    lexer.error(
                        TextMarkupErrorCode::UnbalancedScope,
                        start,
                        "closing scope has no opener",
                    )
                })?;
                MarkupTokenKind::ScopeEnd
            }
            '\\' => lexer.code(start)?,
            '%' => lexer.percent(start)?,
            '^' => match lexer.peek() {
                Some('I') => {
                    lexer.take();
                    MarkupTokenKind::Tab
                }
                Some('J') => {
                    lexer.take();
                    MarkupTokenKind::CaretControl(CaretControl::LineFeed)
                }
                Some('M') => {
                    lexer.take();
                    MarkupTokenKind::CaretControl(CaretControl::CarriageReturn)
                }
                _ => MarkupTokenKind::Literal("^".into()),
            },
            '\t' => MarkupTokenKind::Tab,
            '\n' => MarkupTokenKind::RawLineFeed,
            '\r' => {
                if lexer.peek() == Some('\n') {
                    lexer.take();
                    MarkupTokenKind::RawCarriageReturnLineFeed
                } else {
                    MarkupTokenKind::RawCarriageReturn
                }
            }
            c if forbidden_literal(c) => {
                return Err(lexer.error(
                    TextMarkupErrorCode::StructuralControl,
                    start,
                    "source contains an unsupported structural control",
                ))
            }
            _ => {
                while let Some(next) = lexer.peek() {
                    if matches!(next, '{' | '}' | '\\' | '%' | '^' | '\t' | '\n' | '\r')
                        || forbidden_literal(next)
                    {
                        break;
                    }
                    lexer.take();
                }
                MarkupTokenKind::Literal(source[start..lexer.pos].into())
            }
        };
        if lexer.tokens.len() == limits.max_tokens {
            return Err(lexer.error(
                TextMarkupErrorCode::ResourceLimit,
                start,
                "token count exceeds budget",
            ));
        }
        lexer.tokens.push(LocatedMarkupToken {
            kind: token,
            range: start..lexer.pos,
        });
    }
    if let Some(start) = lexer.scopes.last() {
        return Err(lexer.error(
            TextMarkupErrorCode::UnbalancedScope,
            *start,
            "scope has no closing brace",
        ));
    }
    Ok(lexer.tokens)
}

fn forbidden_literal(c: char) -> bool {
    c.is_control() || matches!(c, '\u{2028}' | '\u{2029}')
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    scopes: Vec<usize>,
    tokens: Vec<LocatedMarkupToken>,
}

impl Lexer<'_> {
    fn error(
        &self,
        code: TextMarkupErrorCode,
        offset: usize,
        message: &'static str,
    ) -> TextMarkupError {
        TextMarkupError::new(code, offset, message)
    }
    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }
    fn take(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn literal(&self, c: char, start: usize) -> Result<MarkupTokenKind, TextMarkupError> {
        if forbidden_literal(c) {
            Err(self.error(
                TextMarkupErrorCode::StructuralControl,
                start,
                "escaped scalar is a structural control",
            ))
        } else {
            Ok(MarkupTokenKind::Literal(c.to_string()))
        }
    }
    fn body(&mut self, start: usize) -> Result<&str, TextMarkupError> {
        let begin = self.pos;
        while let Some(c) = self.take() {
            if c == '%' && self.peek() == Some('<') {
                return Err(self.error(
                    TextMarkupErrorCode::DynamicField,
                    self.pos - 1,
                    "dynamic fields inside code bodies require a separate semantic model",
                ));
            }
            if c == ';' {
                return Ok(&self.source[begin..self.pos - 1]);
            }
            if forbidden_literal(c) {
                return Err(self.error(
                    TextMarkupErrorCode::StructuralControl,
                    start,
                    "control code body contains a structural control",
                ));
            }
            if c == '\\' {
                match self.take() {
                    Some(';' | '\\' | '{' | '}') => {}
                    Some('U') if self.peek() == Some('+') => {
                        self.take();
                        self.unicode(start)?;
                    }
                    _ => {
                        return Err(self.error(
                            TextMarkupErrorCode::MalformedCode,
                            start,
                            "invalid escape within control code body",
                        ))
                    }
                }
            }
        }
        Err(self.error(
            TextMarkupErrorCode::MalformedCode,
            start,
            "control code requires a terminating semicolon",
        ))
    }
    fn unicode_unit(&mut self, start: usize) -> Result<u16, TextMarkupError> {
        let begin = self.pos;
        for _ in 0..4 {
            if !self.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                return Err(self.error(
                    TextMarkupErrorCode::InvalidUnicode,
                    start,
                    "Unicode escape requires four hex digits",
                ));
            }
            self.take();
        }
        u16::from_str_radix(&self.source[begin..self.pos], 16).map_err(|_| {
            self.error(
                TextMarkupErrorCode::InvalidUnicode,
                start,
                "invalid Unicode escape",
            )
        })
    }
    fn unicode(&mut self, start: usize) -> Result<MarkupTokenKind, TextMarkupError> {
        let high = self.unicode_unit(start)?;
        let scalar = if (0xd800..=0xdbff).contains(&high) {
            if !self.source[self.pos..].starts_with(r"\U+") {
                return Err(self.error(
                    TextMarkupErrorCode::InvalidUnicode,
                    start,
                    "high surrogate requires a following low surrogate",
                ));
            }
            self.pos += 3;
            let low = self.unicode_unit(start)?;
            if !(0xdc00..=0xdfff).contains(&low) {
                return Err(self.error(
                    TextMarkupErrorCode::InvalidUnicode,
                    start,
                    "invalid low surrogate",
                ));
            }
            0x10000 + ((u32::from(high) - 0xd800) << 10) + (u32::from(low) - 0xdc00)
        } else {
            u32::from(high)
        };
        let c = char::from_u32(scalar).ok_or_else(|| {
            self.error(
                TextMarkupErrorCode::InvalidUnicode,
                start,
                "invalid Unicode scalar",
            )
        })?;
        self.literal(c, start)
    }
    fn code(&mut self, start: usize) -> Result<MarkupTokenKind, TextMarkupError> {
        let c = self.take().ok_or_else(|| {
            self.error(
                TextMarkupErrorCode::MalformedCode,
                start,
                "trailing backslash",
            )
        })?;
        match c {
            '\\' | '{' | '}' | ';' => self.literal(c, start),
            '~' => self.literal('\u{a0}', start),
            'U' if self.peek() == Some('+') => {
                self.take();
                self.unicode(start)
            }
            'P' => Ok(MarkupTokenKind::ParagraphBreak),
            'N' => Ok(MarkupTokenKind::ColumnBreak),
            'L' | 'l' | 'O' | 'o' | 'K' | 'k' => Ok(MarkupTokenKind::Decoration {
                decoration: match c {
                    'L' | 'l' => MarkupDecoration::Underline,
                    'O' | 'o' => MarkupDecoration::Overline,
                    _ => MarkupDecoration::StrikeThrough,
                },
                enabled: c.is_uppercase(),
                toggle: false,
            }),
            'H' | 'W' | 'w' | 'T' | 'Q' => {
                let property = match c {
                    'H' => MarkupScalarProperty::Height,
                    'W' | 'w' => MarkupScalarProperty::Width,
                    'T' => MarkupScalarProperty::Tracking,
                    _ => MarkupScalarProperty::ObliqueDegrees,
                };
                let body = self.body(start)?;
                syntax::scalar(property, body, start)
            }
            'C' | 'c' | 'A' => {
                let body = self.body(start)?;
                let value = body.parse::<u32>().map_err(|_| {
                    self.error(
                        TextMarkupErrorCode::InvalidNumber,
                        start,
                        "invalid integer code",
                    )
                })?;
                if c == 'C' {
                    let remaining = &self.source.as_bytes()[self.pos..];
                    let count = remaining
                        .iter()
                        .take_while(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'-'))
                        .count();
                    if count > 0
                        && remaining.get(count) == Some(&b';')
                        && remaining[..count].iter().any(u8::is_ascii_digit)
                    {
                        return Err(self.error(TextMarkupErrorCode::UnsupportedCode, start,
                            "secondary indexed color syntax requires an unsupported gradient or ambiguous source profile"));
                    }
                }
                match c {
                    'C' if value <= 256 => Ok(MarkupTokenKind::IndexedColor(value as u16)),
                    'c' if value <= 0xffffff => Ok(MarkupTokenKind::PackedBgrColor(value)),
                    'A' if value <= 2 => Ok(MarkupTokenKind::LineAlignment(value as u8)),
                    _ => Err(self.error(
                        TextMarkupErrorCode::InvalidNumber,
                        start,
                        "integer code is outside its range",
                    )),
                }
            }
            'f' | 'F' => {
                let shx = c == 'F' && self.peek() == Some('N');
                if shx {
                    self.take();
                }
                let body = self.body(start)?;
                if body.is_empty() || body.starts_with('|') {
                    return Err(self.error(
                        TextMarkupErrorCode::MalformedCode,
                        start,
                        "font name is required",
                    ));
                }
                Ok(MarkupTokenKind::Font {
                    body: body.into(),
                    shx,
                })
            }
            'p' => {
                let body = self.body(start)?;
                Ok(MarkupTokenKind::ParagraphProperties(body.into()))
            }
            'S' | 's' => {
                let body = self.body(start)?;
                let (upper, lower) = body.split_once(['/', '#', '^']).ok_or_else(|| {
                    TextMarkupError::new(
                        TextMarkupErrorCode::MalformedCode,
                        start,
                        "stack requires a separator",
                    )
                })?;
                if upper.is_empty() && lower.is_empty() {
                    return Err(self.error(
                        TextMarkupErrorCode::MalformedCode,
                        start,
                        "stack parts cannot both be empty",
                    ));
                }
                Ok(MarkupTokenKind::Stack(body.into()))
            }
            _ => Err(self.error(
                TextMarkupErrorCode::UnsupportedCode,
                start,
                "unsupported control code",
            )),
        }
    }
    fn percent(&mut self, start: usize) -> Result<MarkupTokenKind, TextMarkupError> {
        if self.peek() == Some('<') {
            return Err(self.error(
                TextMarkupErrorCode::DynamicField,
                start,
                "dynamic fields require a separate semantic model",
            ));
        }
        if self.peek() != Some('%') {
            return self.literal('%', start);
        }
        self.take();
        let c = self.take().ok_or_else(|| {
            self.error(
                TextMarkupErrorCode::MalformedCode,
                start,
                "incomplete percent code",
            )
        })?;
        match c {
            'd' | 'D' => self.literal('°', start),
            'p' | 'P' => self.literal('±', start),
            'c' | 'C' => self.literal('Ø', start),
            '%' => {
                if self.peek() == Some('%') {
                    self.take();
                }
                self.literal('%', start)
            }
            'u' | 'U' | 'o' | 'O' | 'k' | 'K' => Ok(MarkupTokenKind::Decoration {
                decoration: match c {
                    'u' | 'U' => MarkupDecoration::Underline,
                    'o' | 'O' => MarkupDecoration::Overline,
                    _ => MarkupDecoration::StrikeThrough,
                },
                enabled: false,
                toggle: true,
            }),
            '0'..='9' => {
                let a = self.take().filter(char::is_ascii_digit).ok_or_else(|| {
                    self.error(
                        TextMarkupErrorCode::MalformedCode,
                        start,
                        "decimal percent code requires three digits",
                    )
                })?;
                let b = self.take().filter(char::is_ascii_digit).ok_or_else(|| {
                    self.error(
                        TextMarkupErrorCode::MalformedCode,
                        start,
                        "decimal percent code requires three digits",
                    )
                })?;
                let scalar = c.to_digit(10).unwrap() * 100
                    + a.to_digit(10).unwrap() * 10
                    + b.to_digit(10).unwrap();
                let value = char::from_u32(scalar).ok_or_else(|| {
                    self.error(
                        TextMarkupErrorCode::InvalidUnicode,
                        start,
                        "invalid decimal percent scalar",
                    )
                })?;
                self.literal(value, start)
            }
            _ => Err(self.error(
                TextMarkupErrorCode::UnsupportedCode,
                start,
                "unsupported percent code",
            )),
        }
    }
}
