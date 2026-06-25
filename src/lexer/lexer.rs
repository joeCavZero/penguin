use crate::core::*;
use crate::lexer::*;

#[derive(Debug)]
pub enum LexerCallbackResponse {
    Token(String),
    String(String),
}

pub fn lex_file(
    file_path: String,
    file_id: usize,
) -> Result<Vec<PengPositionedToken>, PengError> {

    let source = match std::fs::read_to_string(file_path) {
        Ok(src) => src,
        Err(e) => return Err(
            PengError::SyntaxError(e.to_string())
        ),
    };

    let mut ptokens: Vec<PengPositionedToken> = Vec::new();
    let mut err_to_return: Option<PengError> = None;
    lex_source_fn(
        source,
        |res, line, column| {
            let pos = PengPosition::new_file(
                file_id,
                line,
                column,
            );

            match res {
                LexerCallbackResponse::Token(t) => {
                    match PengPositioned::<PengToken>::from_string(t, pos.clone()) {
                        Ok(ptk) => ptokens.push(ptk),
                        Err(e) => {
                            err_to_return = Some(
                                PengError::new_positioned_error(
                                    e,
                                    pos.clone(),
                                )
                            )
                        }
                    }
                }
                LexerCallbackResponse::String(s) => {
                    ptokens.push(PengPositioned::<PengToken>::new_string(s, pos))
                }
            }
        }
    );
    if let Some(err) = err_to_return {
        return Err(err);
    }
    Ok(ptokens)
}

pub fn lex_source(
    source: String,
) -> Result<Vec<PengPositionedToken>, PengError> {

    let mut ptokens: Vec<PengPositionedToken> = Vec::new();
    let mut err_to_return: Option<PengError> = None;
    lex_source_fn(
        source,
        |res, line, column| {
            let pos = PengPosition::new_source(
                line,
                column,
            );

            match res {
                LexerCallbackResponse::Token(t) => {
                    match PengPositioned::<PengToken>::from_string(t, pos.clone()) {
                        Ok(ptk) => ptokens.push(ptk),
                        Err(e) => {
                            err_to_return = Some(
                                PengError::new_positioned_error(
                                    e,
                                    pos.clone(),
                                )
                            )
                        }
                    }
                }
                LexerCallbackResponse::String(s) => {
                    ptokens.push(PengPositioned::<PengToken>::new_string(s, pos))
                }
            }
        }
    );
    if let Some(err) = err_to_return {
        return Err(err);
    }
    Ok(ptokens)
}

fn lex_source_fn<F>(source: String, mut f: F)
where
    F: FnMut(LexerCallbackResponse, usize, Option<usize>),
{
    let mut special_tokens = vec![
        "=", ".", "..", "...", ":", ";", ",",
        "(", ")", "{", "}", "[", "]", 
        "<", ">", "<=", ">=", "==", "!=", 
        "+", "-", "*", "**", "/", "%", 
        "+=", "-=", "*=", "**=", "/=", "%=", 
        "&", "|", "&&", "||", "!", "?", "->"
    ];
    let mut string_separators = vec![("\"", "\""), ("#\"", "\"#")];

    const LINE_COMMENT: &str = "//";
    const MULTI_LINE_COMMENT: (&str, &str) = ("/*", "*/");
    const SPACE: char = ' ';
    const NEW_LINE: char = '\n';
    const SCAPE_CHARACTER: char = '\\';

    special_tokens.sort_by(|a, b| b.len().cmp(&a.len()));
    string_separators.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let src = source.replace("\r", "");
    let bytes_len = src.len();

    let mut i: usize = 0;

    let mut actual_line: usize = 1;
    let mut actual_column: Option<usize> = Some(1);

    let mut line_has_tab_indent = false;

    let mut token_acc = String::new();
    let mut token_start_column: Option<usize> = Some(1);

    while i < bytes_len {
        let ch = src[i..].chars().next().unwrap_or('\0');
        let ch_len = ch.len_utf8();

        if token_acc.is_empty() {
            token_start_column = actual_column;
        }

        if ch == '\t' {
            line_has_tab_indent = true;
            actual_column = None;
            i += ch_len;
            continue;
        }

        if ch == NEW_LINE {
            flush_token(
                &mut token_acc,
                token_start_column,
                line_has_tab_indent,
                actual_line,
                &mut f,
            );
            token_start_column = Some(1);
            token_acc.clear();
            actual_line += 1;
            actual_column = Some(1);
            line_has_tab_indent = false;
            i += ch_len;
            continue;
        }

        if ch == SPACE {
            flush_token(
                &mut token_acc,
                token_start_column,
                line_has_tab_indent,
                actual_line,
                &mut f,
            );
            token_start_column = actual_column;
            bump_col(&mut actual_column, 1);
            i += ch_len;
            continue;
        }

        if starts_with_at(&src, i, LINE_COMMENT) {
            flush_token(
                &mut token_acc,
                token_start_column,
                line_has_tab_indent,
                actual_line,
                &mut f,
            );
            token_acc.clear();
            token_start_column = actual_column;

            i += LINE_COMMENT.len();
            bump_col(&mut actual_column, LINE_COMMENT.len());

            while i < bytes_len {
                let c = src[i..].chars().next().unwrap_or('\0');
                let l = c.len_utf8();
                i += l;

                if c == '\t' {
                    line_has_tab_indent = true;
                    actual_column = None;
                } else if c == NEW_LINE {
                    actual_line += 1;
                    actual_column = Some(1);
                    line_has_tab_indent = false;
                    break;
                } else {
                    bump_col(&mut actual_column, 1);
                }
            }
            continue;
        }

        if starts_with_at(&src, i, MULTI_LINE_COMMENT.0) {
            flush_token(
                &mut token_acc,
                token_start_column,
                line_has_tab_indent,
                actual_line,
                &mut f,
            );
            token_acc.clear();
            token_start_column = actual_column;

            let start = i;
            i += MULTI_LINE_COMMENT.0.len();

            while i < bytes_len {
                if starts_with_at(&src, i, MULTI_LINE_COMMENT.1) {
                    i += MULTI_LINE_COMMENT.1.len();
                    break;
                }
                let c = src[i..].chars().next().unwrap_or('\0');
                i += c.len_utf8();
            }

            let consumed = &src[start..i.min(bytes_len)];
            advance_position_by_str(
                consumed,
                &mut actual_line,
                &mut actual_column,
                &mut line_has_tab_indent,
            );
            continue;
        }

        if let Some((open, close)) = find_separator_at(&src, i, &string_separators) {
            flush_token(
                &mut token_acc,
                token_start_column,
                line_has_tab_indent,
                actual_line,
                &mut f,
            );
            token_acc.clear();

            let start_line = actual_line;
            let start_col = if line_has_tab_indent {
                None
            } else {
                actual_column
            };

            if let Some(end) = read_delimited_end(&src, i, open, close, SCAPE_CHARACTER) {
                let consumed = &src[i..end];
                advance_position_by_str(
                    consumed,
                    &mut actual_line,
                    &mut actual_column,
                    &mut line_has_tab_indent,
                );

                let inner = &src[i + open.len()..end - close.len()];
                let mut content = unescape_string(inner, SCAPE_CHARACTER);
                if content.len() >= 2 && content.starts_with('"') && content.ends_with('"') {
                    content = content[1..content.len() - 1].to_string();
                }
                f(
                    LexerCallbackResponse::String(content),
                    start_line,
                    start_col,
                );

                i = end;
                continue;
            } else {
                let consumed = &src[i..];
                advance_position_by_str(
                    consumed,
                    &mut actual_line,
                    &mut actual_column,
                    &mut line_has_tab_indent,
                );

                let inner = &src[i + open.len()..];
                let mut content = unescape_string(inner, SCAPE_CHARACTER);
                if content.len() >= 2 && content.starts_with('"') && content.ends_with('"') {
                    content = content[1..content.len() - 1].to_string();
                }
                f(
                    LexerCallbackResponse::String(content),
                    start_line,
                    start_col,
                );
                break;
            }
        }

        if token_acc.is_empty() {
            match read_number_at(&src, i) {
                Some((num, consumed)) => {
                    let col = if line_has_tab_indent {
                        None
                    } else {
                        actual_column
                    };
                    f(
                        LexerCallbackResponse::Token(num.clone()),
                        actual_line,
                        col,
                    );

                    i += consumed;
                    bump_col(&mut actual_column, num.chars().count());
                    continue;
                }
                None => {}
            }
        }

        if let Some(tok) = find_special_at(&src, i, &special_tokens) {
            flush_token(
                &mut token_acc,
                token_start_column,
                line_has_tab_indent,
                actual_line,
                &mut f,
            );
            token_acc.clear();

            let col = if line_has_tab_indent {
                None
            } else {
                actual_column
            };
            f(
                LexerCallbackResponse::Token(tok.to_string()),
                actual_line,
                col,
            );

            i += tok.len();
            bump_col(&mut actual_column, tok.len());
            continue;
        }

        token_acc.push(ch);
        bump_col(&mut actual_column, 1);
        i += ch_len;
    }

    flush_token(
        &mut token_acc,
        token_start_column,
        line_has_tab_indent,
        actual_line,
        &mut f,
    );
}

fn bump_col(col: &mut Option<usize>, n: usize) {
    if let Some(c) = *col {
        *col = Some(c + n);
    }
}

fn starts_with_at(s: &str, idx: usize, pat: &str) -> bool {
    s.as_bytes()
        .get(idx..idx + pat.len())
        .map(|b| b == pat.as_bytes())
        .unwrap_or(false)
}

fn find_special_at<'a>(s: &str, idx: usize, specials: &'a [&'a str]) -> Option<&'a str> {
    for tok in specials {
        if starts_with_at(s, idx, tok) {
            return Some(*tok);
        }
    }
    None
}

fn find_separator_at<'a>(
    s: &str,
    idx: usize,
    seps: &'a [(&'a str, &'a str)],
) -> Option<(&'a str, &'a str)> {
    for (open, close) in seps {
        if starts_with_at(s, idx, open) {
            return Some((*open, *close));
        }
    }
    None
}

fn read_delimited_end(
    s: &str,
    start: usize,
    open: &str,
    close: &str,
    escape: char,
) -> Option<usize> {
    if !starts_with_at(s, start, open) {
        return None;
    }

    let mut i = start + open.len();
    let mut escaped = false;

    while i < s.len() {
        let ch = s[i..].chars().next().unwrap_or('\0');
        let l = ch.len_utf8();

        if escaped {
            escaped = false;
            i += l;
            continue;
        }

        if ch == escape {
            escaped = true;
            i += l;
            continue;
        }

        if starts_with_at(s, i, close) {
            return Some(i + close.len());
        }

        i += l;
    }

    None
}

fn advance_position_by_str(
    consumed: &str,
    line: &mut usize,
    col: &mut Option<usize>,
    line_has_tab_indent: &mut bool,
) {
    for ch in consumed.chars() {
        if ch == '\n' {
            *line += 1;
            *col = Some(1);
            *line_has_tab_indent = false;
        } else if ch == '\t' {
            *line_has_tab_indent = true;
            *col = None;
        } else {
            bump_col(col, 1);
        }
    }
}

fn flush_token<F>(
    token_acc: &mut String,
    token_start_column: Option<usize>,
    line_has_tab_indent: bool,
    actual_line: usize,
    f: &mut F,
) where
    F: FnMut(LexerCallbackResponse, usize, Option<usize>),
{
    if token_acc.is_empty() {
        return;
    }
    let col = if line_has_tab_indent {
        None
    } else {
        token_start_column
    };
    let tok = std::mem::take(token_acc);
    f(LexerCallbackResponse::Token(tok), actual_line, col);
}

fn unescape_string(s: &str, escape: char) -> String {
    let mut out = String::new();
    let mut it = s.chars();

    while let Some(ch) = it.next() {
        if ch != escape {
            out.push(ch);
            continue;
        }

        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some(other) => out.push(other),
            None => break,
        }
    }

    out
}

fn is_ascii_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

fn peek_char_at(s: &str, idx: usize) -> Option<char> {
    match s.get(idx..) {
        Some(slice) => slice.chars().next(),
        None => None,
    }
}


fn read_number_at(s: &str, start: usize) -> Option<(String, usize)> {
    let first = match peek_char_at(s, start) {
        Some(v) => v,
        None => return None,
    };

    // Number literal precisa começar com número.
    // Portanto: .5 NÃO é número aqui.
    if !is_ascii_digit(first) {
        return None;
    }

    let mut i = start;
    let mut out = String::new();

    // parte inteira: 1, 10, 1_000, 1___
    while let Some(ch) = peek_char_at(s, i) {
        if is_ascii_digit(ch) || ch == '_' {
            out.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }

    // parte decimal: 1.0, 1_._0, 1.0__
    let has_decimal_dot = match peek_char_at(s, i) {
        Some('.') => {
            match peek_char_at(s, i + 1) {
                Some('.') => false,
                _ => true,
            }
        }
        _ => false,
    };

    if has_decimal_dot {
        out.push('.');
        i += '.'.len_utf8();

        while let Some(ch) = peek_char_at(s, i) {
            if is_ascii_digit(ch) || ch == '_' {
                out.push(ch);
                i += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    // sufixo: i, i8, i16, u, u32, f32, f64...
    //
    // Aqui o lexer consome letras e números depois do literal.
    // Assim, "10abc" vira um único token "10abc",
    // e o Token::from_string pode retornar:
    // invalid number literal: 10abc
    while let Some(ch) = peek_char_at(s, i) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }

    Some((out, i - start))
}
