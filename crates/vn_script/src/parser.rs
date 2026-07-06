use std::path::Path;
use thiserror::Error;
use vn_core::{
    AssignOp, BinaryOp, Choice, Expr, Script, SourcePos, Stmt, StmtKind, TextEffect, Transition,
    UnaryOp, Value,
};

/// Script parse error.
#[derive(Debug, Error, Eq, PartialEq)]
#[error("{}:{}:{}: {message}", pos.file, pos.line, pos.column)]
pub struct ParseError {
    pub pos: SourcePos,
    pub message: String,
}

/// Parses a source string with a synthetic file name.
pub fn parse_source(file: impl Into<String>, source: &str) -> Result<Script, ParseError> {
    Parser::new(file.into(), source).parse_script()
}

/// Parses a source string using a path as file name.
pub fn parse_file(path: &Path, source: &str) -> Result<Script, ParseError> {
    parse_source(path.display().to_string(), source)
}

#[derive(Clone, Debug)]
struct Line {
    number: usize,
    indent: usize,
    text: String,
}

struct Parser {
    file: String,
    lines: Vec<Line>,
    index: usize,
}

impl Parser {
    fn new(file: String, source: &str) -> Self {
        let lines = source
            .lines()
            .enumerate()
            .filter_map(|(idx, raw)| {
                let without_comment = strip_comment(raw);
                if without_comment.trim().is_empty() {
                    return None;
                }
                let indent = without_comment.chars().take_while(|ch| *ch == ' ').count();
                Some(Line {
                    number: idx + 1,
                    indent,
                    text: without_comment[indent..].trim_end().to_string(),
                })
            })
            .collect();
        Self {
            file,
            lines,
            index: 0,
        }
    }

    fn parse_script(mut self) -> Result<Script, ParseError> {
        let statements = self.parse_block(0)?;
        if let Some(line) = self.current() {
            return Err(self.error(line, "unexpected indentation"));
        }
        Ok(Script { statements })
    }

    fn parse_block(&mut self, indent: usize) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while let Some(line) = self.current() {
            if line.indent < indent {
                break;
            }
            if line.indent > indent {
                return Err(self.error(line, "unexpected indented block"));
            }
            let statement = self.parse_statement(indent)?;
            let opens_label_block = matches!(statement.kind, StmtKind::Label { .. });
            statements.push(statement);
            if opens_label_block
                && let Some(next) = self.current()
                && next.indent > indent
            {
                statements.extend(self.parse_block(next.indent)?);
            }
        }
        Ok(statements)
    }

    fn parse_statement(&mut self, indent: usize) -> Result<Stmt, ParseError> {
        let line = self
            .current()
            .cloned()
            .expect("current line checked by caller");
        self.index += 1;
        let pos = self.pos(&line, 1);
        let text = line.text.as_str();
        let kind = if let Some(name) = text
            .strip_prefix("label ")
            .and_then(|tail| tail.strip_suffix(':'))
        {
            StmtKind::Label {
                name: parse_name(name.trim(), &line, self)?,
            }
        } else if let Some(image) = text.strip_prefix("scene ") {
            let (image, transition) = parse_visual_tail(image.trim(), &line, self)?;
            StmtKind::Scene { image, transition }
        } else if let Some(rest) = text.strip_prefix("show ") {
            parse_show(rest, &line, self)?
        } else if let Some(tag) = text.strip_prefix("hide ") {
            StmtKind::Hide {
                tag: parse_name(tag.trim(), &line, self)?,
            }
        } else if let Some(path) = text.strip_prefix("play music ") {
            StmtKind::PlayMusic {
                path: parse_quoted(path.trim(), &line, self)?,
            }
        } else if text == "stop music" {
            StmtKind::StopMusic
        } else if text == "menu:" {
            StmtKind::Menu {
                choices: self.parse_menu(indent + 4)?,
            }
        } else if let Some(label) = text.strip_prefix("jump ") {
            StmtKind::Jump {
                label: parse_name(label.trim(), &line, self)?,
            }
        } else if text == "end" {
            StmtKind::End
        } else if let Some(rest) = text.strip_prefix('$') {
            parse_assignment(rest.trim(), &line, self)?
        } else if let Some(rest) = text
            .strip_prefix("if ")
            .and_then(|tail| tail.strip_suffix(':'))
        {
            let then_body = self.parse_block(indent + 4)?;
            let else_body = if self
                .current()
                .is_some_and(|next| next.indent == indent && next.text == "else:")
            {
                self.index += 1;
                self.parse_block(indent + 4)?
            } else {
                Vec::new()
            };
            StmtKind::If {
                cond: parse_expr(rest.trim(), &line, self)?,
                then_body,
                else_body,
            }
        } else if text.starts_with('"') {
            let (text, effect) = parse_text_effect(text, &line, self)?;
            StmtKind::Say {
                speaker: None,
                text,
                effect,
            }
        } else if let Some((speaker, quoted)) = split_say(text) {
            let (text, effect) = parse_text_effect(quoted, &line, self)?;
            StmtKind::Say {
                speaker: Some(parse_name(speaker, &line, self)?),
                text,
                effect,
            }
        } else {
            return Err(self.error(&line, "unknown statement"));
        };
        Ok(Stmt { kind, pos })
    }

    fn parse_menu(&mut self, indent: usize) -> Result<Vec<Choice>, ParseError> {
        let mut choices = Vec::new();
        while let Some(line) = self.current() {
            if line.indent < indent {
                break;
            }
            if line.indent != indent {
                return Err(self.error(line, "menu choice indentation must match menu block"));
            }
            let line = line.clone();
            let Some(choice_text) = line.text.strip_suffix(':') else {
                return Err(self.error(&line, "menu choice must end with ':'"));
            };
            let (choice_text, condition) = parse_choice_header(choice_text, &line, self)?;
            let text = parse_quoted(choice_text, &line, self)?;
            self.index += 1;
            let body = self.parse_block(indent + 4)?;
            if body.is_empty() {
                return Err(self.error(&line, "menu choice body cannot be empty"));
            }
            choices.push(Choice {
                text,
                condition,
                body,
                pos: self.pos(&line, 1),
            });
        }
        if choices.is_empty() {
            let pos = self
                .current()
                .map(|line| self.pos(line, 1))
                .unwrap_or_else(|| SourcePos {
                    file: self.file.clone(),
                    line: 1,
                    column: 1,
                });
            return Err(ParseError {
                pos,
                message: "menu requires at least one choice".to_string(),
            });
        }
        Ok(choices)
    }

    fn current(&self) -> Option<&Line> {
        self.lines.get(self.index)
    }

    fn pos(&self, line: &Line, column: usize) -> SourcePos {
        SourcePos {
            file: self.file.clone(),
            line: line.number,
            column: line.indent + column,
        }
    }

    fn error(&self, line: &Line, message: impl Into<String>) -> ParseError {
        ParseError {
            pos: self.pos(line, 1),
            message: message.into(),
        }
    }
}

fn strip_comment(raw: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &raw[..idx],
            _ => {}
        }
    }
    raw
}

fn parse_show(rest: &str, line: &Line, parser: &Parser) -> Result<StmtKind, ParseError> {
    let (rest, transition) = parse_visual_tail(rest, line, parser)?;
    let parts = rest.split_whitespace().collect::<Vec<_>>();
    if parts.is_empty() {
        return Err(parser.error(line, "show requires image name"));
    }
    let at_index = parts.iter().position(|part| *part == "at");
    let (image_parts, position) = if let Some(index) = at_index {
        if index + 1 >= parts.len() {
            return Err(parser.error(line, "show at requires a position"));
        }
        (&parts[..index], parts[index + 1].to_string())
    } else {
        (&parts[..], "center".to_string())
    };
    if image_parts.is_empty() {
        return Err(parser.error(line, "show requires image name"));
    }
    Ok(StmtKind::Show {
        tag: image_parts[0].to_string(),
        attrs: image_parts[1..]
            .iter()
            .map(|part| (*part).to_string())
            .collect(),
        position,
        transition,
    })
}

fn parse_visual_tail(
    text: &str,
    line: &Line,
    parser: &Parser,
) -> Result<(String, Option<Transition>), ParseError> {
    let Some((head, effect)) = text.rsplit_once(" with ") else {
        return Ok((text.to_string(), None));
    };
    Ok((
        head.trim().to_string(),
        Some(parse_transition(effect.trim(), line, parser)?),
    ))
}

fn parse_text_effect(
    text: &str,
    line: &Line,
    parser: &Parser,
) -> Result<(String, TextEffect), ParseError> {
    let Some((quoted, effect)) = text.rsplit_once(" with ") else {
        return Ok((parse_quoted(text, line, parser)?, TextEffect::Instant));
    };
    Ok((
        parse_quoted(quoted.trim(), line, parser)?,
        parse_effect(effect.trim(), line, parser)?,
    ))
}

fn parse_transition(text: &str, line: &Line, parser: &Parser) -> Result<Transition, ParseError> {
    let (kind, args) = parse_call(text);
    if !matches!(kind, "fade" | "dissolve") {
        return Err(parser.error(line, "unknown transition"));
    }
    Ok(Transition {
        kind: kind.to_string(),
        duration_ms: parse_duration_ms(args, 0, line, parser)?,
    })
}

fn parse_effect(text: &str, line: &Line, parser: &Parser) -> Result<TextEffect, ParseError> {
    let (kind, args) = parse_call(text);
    match kind {
        "typewriter" => Ok(TextEffect::Typewriter {
            chars_per_second: parse_speed(args, 30, line, parser)?,
        }),
        "instant" => Ok(TextEffect::Instant),
        _ => Err(parser.error(line, "unknown text effect")),
    }
}

fn parse_call(text: &str) -> (&str, &str) {
    if let Some((kind, args)) = text.strip_suffix(')').and_then(|text| text.split_once('(')) {
        (kind.trim(), args.trim())
    } else {
        (text.trim(), "")
    }
}

fn parse_duration_ms(
    args: &str,
    default_ms: u32,
    line: &Line,
    parser: &Parser,
) -> Result<u32, ParseError> {
    let Some(value) = parse_named_arg(args, "duration") else {
        return Ok(default_ms);
    };
    let seconds = value
        .parse::<f32>()
        .map_err(|_| parser.error(line, "duration must be a number"))?;
    Ok((seconds * 1000.0).round() as u32)
}

fn parse_speed(args: &str, default: u16, line: &Line, parser: &Parser) -> Result<u16, ParseError> {
    let Some(value) = parse_named_arg(args, "speed") else {
        return Ok(default);
    };
    value
        .parse::<u16>()
        .map_err(|_| parser.error(line, "speed must be an integer"))
}

fn parse_named_arg<'a>(args: &'a str, name: &str) -> Option<&'a str> {
    args.split(',').find_map(|arg| {
        let (key, value) = arg.trim().split_once('=')?;
        (key.trim() == name).then_some(value.trim())
    })
}

fn parse_assignment(rest: &str, line: &Line, parser: &Parser) -> Result<StmtKind, ParseError> {
    let (var, op, value) = if let Some((var, value)) = rest.split_once("+=") {
        (var, AssignOp::Add, value)
    } else if let Some((var, value)) = rest.split_once("-=") {
        (var, AssignOp::Sub, value)
    } else if let Some((var, value)) = rest.split_once('=') {
        (var, AssignOp::Set, value)
    } else {
        return Err(parser.error(line, "assignment requires '='"));
    };
    Ok(StmtKind::Set {
        var: parse_name(var.trim(), line, parser)?,
        op,
        value: parse_expr(value.trim(), line, parser)?,
    })
}

fn parse_choice_header<'a>(
    text: &'a str,
    line: &Line,
    parser: &Parser,
) -> Result<(&'a str, Option<Expr>), ParseError> {
    let Some((choice, condition)) = text.rsplit_once(" if ") else {
        return Ok((text, None));
    };
    Ok((
        choice.trim_end(),
        Some(parse_expr(condition.trim(), line, parser)?),
    ))
}

fn parse_expr(text: &str, line: &Line, parser: &Parser) -> Result<Expr, ParseError> {
    ExprParser::new(text, line, parser).parse()
}

fn parse_value(text: &str, line: &Line, parser: &Parser) -> Result<Value, ParseError> {
    if text == "true" || text == "True" {
        return Ok(Value::Bool(true));
    }
    if text == "false" || text == "False" {
        return Ok(Value::Bool(false));
    }
    if text.starts_with('"') {
        return parse_quoted(text, line, parser).map(Value::Str);
    }
    text.parse::<i64>()
        .map(Value::Int)
        .map_err(|_| parser.error(line, "unsupported literal"))
}

struct ExprParser<'a, 'b> {
    tokens: Vec<&'a str>,
    index: usize,
    line: &'b Line,
    parser: &'b Parser,
}

impl<'a, 'b> ExprParser<'a, 'b> {
    fn new(text: &'a str, line: &'b Line, parser: &'b Parser) -> Self {
        Self {
            tokens: tokenize_expr(text),
            index: 0,
            line,
            parser,
        }
    }

    fn parse(mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_or()?;
        if self.current().is_some() {
            return Err(self
                .parser
                .error(self.line, "unexpected token in expression"));
        }
        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_and()?;
        while self.eat("or") {
            expr = binary(expr, BinaryOp::Or, self.parse_and()?);
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_compare()?;
        while self.eat("and") {
            expr = binary(expr, BinaryOp::And, self.parse_compare()?);
        }
        Ok(expr)
    }

    fn parse_compare(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_add()?;
        while let Some(op) = self.current().and_then(compare_op) {
            self.index += 1;
            expr = binary(expr, op, self.parse_add()?);
        }
        Ok(expr)
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.current() {
                Some("+") => BinaryOp::Add,
                Some("-") => BinaryOp::Sub,
                _ => break,
            };
            self.index += 1;
            expr = binary(expr, op, self.parse_unary()?);
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.eat("not") {
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(self.parse_unary()?),
            });
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let Some(token) = self.current() else {
            return Err(self.parser.error(self.line, "expected expression"));
        };
        self.index += 1;
        parse_value(token, self.line, self.parser)
            .map(Expr::Value)
            .or_else(|_| parse_name(token, self.line, self.parser).map(Expr::Var))
    }

    fn eat(&mut self, token: &str) -> bool {
        if self.current() == Some(token) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn current(&self) -> Option<&'a str> {
        self.tokens.get(self.index).copied()
    }
}

fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}

fn compare_op(token: &str) -> Option<BinaryOp> {
    match token {
        "==" => Some(BinaryOp::Eq),
        "!=" => Some(BinaryOp::Ne),
        "<" => Some(BinaryOp::Lt),
        "<=" => Some(BinaryOp::Le),
        ">" => Some(BinaryOp::Gt),
        ">=" => Some(BinaryOp::Ge),
        _ => None,
    }
}

fn tokenize_expr(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

fn parse_name(text: &str, line: &Line, parser: &Parser) -> Result<String, ParseError> {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return Err(parser.error(line, "expected name"));
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(parser.error(line, "name must start with a letter or '_'"));
    }
    if chars.any(|ch| !(ch == '_' || ch.is_ascii_alphanumeric())) {
        return Err(parser.error(line, "name contains invalid characters"));
    }
    Ok(text.to_string())
}

fn parse_quoted(text: &str, line: &Line, parser: &Parser) -> Result<String, ParseError> {
    let bytes = text.as_bytes();
    if bytes.first() != Some(&b'"') {
        return Err(parser.error(line, "expected quoted string"));
    }
    let mut output = String::new();
    let mut escaped = false;
    for (idx, ch) in text[1..].char_indices() {
        if escaped {
            match ch {
                'n' => output.push('\n'),
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                other => output.push(other),
            }
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => {
                if !text[idx + 2..].trim().is_empty() {
                    return Err(parser.error(line, "unexpected text after string"));
                }
                return Ok(output);
            }
            other => output.push(other),
        }
    }
    Err(parser.error(line, "unterminated string"))
}

fn split_say(text: &str) -> Option<(&str, &str)> {
    let quote = text.find('"')?;
    let speaker = text[..quote].trim();
    if speaker.is_empty() {
        return None;
    }
    Some((speaker, text[quote..].trim()))
}
