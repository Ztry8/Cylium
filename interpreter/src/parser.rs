// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{
    errors,
    file_handler::FileHandler,
    lexer::Token,
    types::{ReturnType, TypesCheck},
};

#[macro_export]
macro_rules! node {
    ($line:expr, $kind:expr) => {
        AstNode {
            line: $line,
            kind: $kind,
        }
    };
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub line: usize,
    pub kind: AstKind,
}

#[derive(Debug, Clone)]
pub enum AstKind {
    Ident(String),

    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    // Vector(Vec<AstKind>),
    Delete(String),
    Echo(Box<AstKind>),
    Exit(i32),

    Func {
        name: String,
        body: Vec<AstNode>,
        args: Vec<(String, TypesCheck)>,
        return_type: ReturnType,
    },
    VarDecl {
        name: String,
        type_: TypesCheck,
        value: Box<AstKind>,
        is_const: bool,
    },
    FuncCall {
        name: String,
        args: Vec<AstKind>,
    },
    Return(Option<Box<AstKind>>),
    Assign {
        name: String,
        op: Token,
        expr: Box<AstKind>,
        var_type: TypesCheck,
    },
    BinaryOp {
        left: Box<AstKind>,
        op: Token,
        right: Box<AstKind>,
        left_type: TypesCheck,
        right_type: TypesCheck,
    },
    UnaryOp {
        op: Token,
        expr: Box<AstKind>,
        expr_type: TypesCheck,
    },
    AsOp {
        expr: Box<AstKind>,
        op: Cast,
        src_type: TypesCheck,
    },
    Condition {
        expr: Box<AstKind>,
        yes: Vec<AstNode>,
        no: Option<ElseBlock>,
    },
    While {
        expr: Box<AstKind>,
        body: Vec<AstNode>,
    },
    For {
        var_name: String,
        start: Box<AstKind>,
        end: Box<AstKind>,
        step: Box<Option<AstKind>>,
        body: Vec<AstNode>,
    },
}

#[derive(Debug, Clone)]
pub enum ElseBlock {
    ElseIf(Box<AstNode>),
    Else(Vec<AstNode>),
}

#[derive(Debug, Clone)]
pub enum Cast {
    String,
    Number,
    Float,
    Boolean,
}

pub struct Parser {
    tokens: Vec<Vec<Token>>,
    line: usize,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Vec<Token>>) -> Self {
        Self {
            line: 0,
            pos: 0,
            tokens,
        }
    }

    pub fn start(&mut self, handler: &FileHandler) -> Vec<AstNode> {
        let mut tree = Vec::new();

        while self.line < self.tokens.len() {
            tree.push(
                self.parse()
                    .unwrap_or_else(|e| handler.show_error(self.line, &e)),
            );
        }

        tree
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens
            .get(self.line)
            .and_then(|line| line.get(self.pos))
    }

    fn next_line(&mut self) {
        self.line += 1;
        self.pos = 0;
    }

    fn parse(&mut self) -> Result<AstNode, String> {
        match self.current_token() {
            Some(Token::Func) => self.parse_func(),
            Some(Token::Const) => {
                let result = self.parse_var();
                self.next_line();
                result
            }
            _ => Err(errors::A20.to_owned()),
        }
    }

    fn parse_var(&mut self) -> Result<AstNode, String> {
        let is_const = self.current_token() == Some(&Token::Const);

        if is_const {
            self.pos += 1;
        }

        let type_ = match self.current_token() {
            Some(Token::Number) => TypesCheck::Number,
            Some(Token::Float) => TypesCheck::Float,
            Some(Token::String) => TypesCheck::String,
            Some(Token::Bool) => TypesCheck::Boolean,
            _ => return Err(errors::A15.to_owned()),
        };

        self.pos += 1;
        if let Some(Token::Ident(name)) = self.current_token().cloned() {
            self.pos += 1;

            if name.is_empty() || !name.is_ascii() || name.len() > 256 {
                return Err(errors::A05.to_owned());
            }

            if self.current_token() == Some(&Token::Assign) {
                self.pos += 1;

                Ok(node!(
                    self.line,
                    AstKind::VarDecl {
                        name,
                        type_,
                        value: Box::new(self.parse_value()?),
                        is_const,
                    }
                ))
            } else {
                Err(errors::A04.to_owned())
            }
        } else {
            Err(errors::A15.to_owned())
        }
    }

    pub fn parse_value(&mut self) -> Result<AstKind, String> {
        let first = self.parse_expr(0)?;

        if self.current_token() != Some(&Token::Comma) {
            if !matches!(
                self.current_token(),
                None | Some(Token::To) | Some(Token::Step) | Some(Token::CloseParen) 
            ) {
                return Err(errors::A15.to_owned());
            }

            return Ok(first);
        }

        if !matches!(
            first,
            AstKind::Number(_) | AstKind::Float(_) | AstKind::String(_) | AstKind::Boolean(_)
        ) {
            return Err(errors::A19.to_owned());
        }

        let mut values = vec![first];

        while self.current_token() == Some(&Token::Comma) {
            self.pos += 1;

            let new = self.parse_expr(0)?;

            if !matches!(
                new,
                AstKind::Number(_) | AstKind::Float(_) | AstKind::String(_) | AstKind::Boolean(_)
            ) {
                return Err(errors::A19.to_owned());
            }

            values.push(new);
        }

        if self.current_token().is_some() {
            return Err(errors::A15.to_owned());
        }

        // Ok(AstKind::Vector(values))
        todo!()
    }

    fn parse_expr(&mut self, min_prec: u8) -> Result<AstKind, String> {
        let mut left = match self.current_token().cloned() {
            Some(Token::Minus) | Some(Token::Not) | Some(Token::BitNot) => {
                let op = self.current_token().unwrap().clone();

                self.pos += 1;
                let expr = self.parse_expr(6)?;

                AstKind::UnaryOp {
                    op,
                    expr: Box::new(expr),
                    expr_type: TypesCheck::Number,
                }
            }
            Some(Token::OpenParen) => {
                self.pos += 1;

                let expr = self.parse_expr(0)?;

                match self.current_token() {
                    Some(Token::CloseParen) => self.pos += 1,
                    _ => return Err(errors::A10.to_owned()),
                }
                expr
            }
            Some(Token::Ident(name)) => {
                self.pos += 1;
                if self.current_token() == Some(&Token::OpenParen) {
                    let args = self.parse_call_args()?;
                    AstKind::FuncCall { name, args }
                } else {
                    AstKind::Ident(name)
                }
            }
            Some(Token::Number) => {
                self.pos += 1;
                AstKind::Ident("number".to_owned())
            }
            Some(Token::Float) => {
                self.pos += 1;
                AstKind::Ident("float".to_owned())
            }
            Some(Token::String) => {
                self.pos += 1;
                AstKind::Ident("string".to_owned())
            }
            Some(Token::Bool) => {
                self.pos += 1;
                AstKind::Ident("bool".to_owned())
            }
            Some(Token::NumberValue(v)) => {
                self.pos += 1;
                AstKind::Number(v)
            }
            Some(Token::FloatValue(v)) => {
                self.pos += 1;
                AstKind::Float(v)
            }
            Some(Token::StringValue(v)) => {
                self.pos += 1;
                AstKind::String(v)
            }
            Some(Token::BooleanValue(v)) => {
                self.pos += 1;
                AstKind::Boolean(v)
            }
            _ => return Err(errors::A15.to_owned()),
        };

        loop {
            let op_tok = match self.current_token() {
                Some(tok)
                    if Self::precedence(tok).is_some()
                        && Self::precedence(tok).unwrap() >= min_prec =>
                {
                    tok.clone()
                }
                _ => break,
            };

            let prec = Self::precedence(&op_tok).unwrap();
            self.pos += 1;

            let right = self.parse_expr(prec + 1)?;

            if op_tok == Token::As {
                left = AstKind::AsOp {
                    expr: Box::new(left),
                    op: if let AstKind::Ident(name) = right {
                        match name.as_str() {
                            "string" => Cast::String,
                            "number" => Cast::Number,
                            "float" => Cast::Float,
                            "bool" => Cast::Boolean,
                            _ => return Err(errors::A15.to_owned()),
                        }
                    } else {
                        return Err(errors::A15.to_owned());
                    },
                    src_type: TypesCheck::Number,
                };
            } else {
                left = AstKind::BinaryOp {
                    left: Box::new(left),
                    op: op_tok,
                    right: Box::new(right),
                    left_type: TypesCheck::Number,
                    right_type: TypesCheck::Number,
                };
            }
        }

        Ok(left)
    }

    fn precedence(tok: &Token) -> Option<u8> {
        match tok {
            Token::Or => Some(1),
            Token::And => Some(2),
            Token::Equal
            | Token::NotEqual
            | Token::Greater
            | Token::Less
            | Token::GreaterEqual
            | Token::LessEqual => Some(3),
            Token::Plus | Token::Minus => Some(4),
            Token::Multiply | Token::Divide | Token::Mod => Some(5),
            Token::As => Some(7),
            Token::BitAnd | Token::BitOr | Token::BitXor | Token::BitRight | Token::BitLeft => {
                Some(8)
            }
            _ => None,
        }
    }

    fn parse_func(&mut self) -> Result<AstNode, String> {
        let func_line = self.line;
        self.pos += 1;

        let name = if let Some(Token::Ident(n)) = self.current_token().cloned() {
            self.pos += 1;
            n
        } else {
            return Err(errors::A15.to_owned());
        };

        if self.current_token() != Some(&Token::OpenParen) {
            return Err(errors::A15.to_owned());
        }
        self.pos += 1;

        let mut args = Vec::new();

        while self.current_token() != Some(&Token::CloseParen) {
            let arg = if let Some(Token::Ident(a)) = self.current_token().cloned() {
                self.pos += 1;
                a
            } else {
                return Err(errors::A15.to_owned());
            };

            if self.current_token() != Some(&Token::Colon) {
                return Err(errors::A15.to_owned());
            }
            self.pos += 1;

            let type_ = match self.current_token() {
                Some(Token::Number) => TypesCheck::Number,
                Some(Token::Float) => TypesCheck::Float,
                Some(Token::String) => TypesCheck::String,
                Some(Token::Bool) => TypesCheck::Boolean,
                _ => return Err(errors::A15.to_owned()),
            };
            self.pos += 1;
            args.push((arg, type_));

            if self.current_token() == Some(&Token::Comma) {
                self.pos += 1;
            } else if self.current_token() != Some(&Token::CloseParen) {
                return Err(errors::A15.to_owned());
            }
        }

        self.pos += 1;

        if self.current_token() != Some(&Token::Arrow) {
            return Err(errors::A15.to_owned());
        }
        self.pos += 1;

        let return_type = match self.current_token() {
            Some(Token::Void) => ReturnType::Void,
            Some(Token::Number) => ReturnType::Number,
            Some(Token::Float) => ReturnType::Float,
            Some(Token::String) => ReturnType::String,
            Some(Token::Bool) => ReturnType::Boolean,
            _ => return Err(errors::A15.to_owned()),
        };
        self.pos += 1;

        let mut body = Vec::new();
        self.next_line();
        while self.current_token().cloned() != Some(Token::End) {
            body.push(self.parse_main()?);
            self.next_line();
        }
        self.next_line();

        Ok(node!(
            func_line,
            AstKind::Func {
                name,
                body,
                args,
                return_type
            }
        ))
    }

    fn parse_call_args(&mut self) -> Result<Vec<AstKind>, String> {
        self.pos += 1;

        let mut args = Vec::new();

        if self.current_token() != Some(&Token::CloseParen) {
            args.push(self.parse_expr(0)?);
            while self.current_token() == Some(&Token::Comma) {
                self.pos += 1;
                args.push(self.parse_expr(0)?);
            }
        }

        if self.current_token() != Some(&Token::CloseParen) {
            return Err(errors::A10.to_owned());
        }

        self.pos += 1;
        Ok(args)
    }

    fn parse_main(&mut self) -> Result<AstNode, String> {
        match self.current_token().cloned() {
            Some(Token::Echo) => {
                self.pos += 1;
                if self.current_token() != Some(&Token::OpenParen) {
                    return Err(errors::A15.to_owned());
                }
                self.pos += 1;
                let val = self.parse_value()?;
                if self.current_token() != Some(&Token::CloseParen) {
                    return Err(errors::A10.to_owned());
                }
                self.pos += 1;
                Ok(node!(self.line, AstKind::Echo(Box::new(val))))
            }
            Some(Token::Delete) => {
                self.pos += 1;
                if self.current_token() != Some(&Token::OpenParen) {
                    return Err(errors::A15.to_owned());
                }
                self.pos += 1;
                let name = if let Some(Token::Ident(n)) = self.current_token().cloned() {
                    self.pos += 1;
                    n
                } else {
                    return Err(errors::A29.to_owned());
                };
                if self.current_token() != Some(&Token::CloseParen) {
                    return Err(errors::A10.to_owned());
                }
                self.pos += 1;
                Ok(node!(self.line, AstKind::Delete(name)))
            }
            Some(Token::Return) => {
                self.pos += 1;
                if self.pos >= self.tokens[self.line].len() {
                    Ok(node!(self.line, AstKind::Return(None)))
                } else {
                    Ok(node!(
                        self.line,
                        AstKind::Return(Some(Box::new(self.parse_value()?)))
                    ))
                }
            }
            Some(Token::Exit) => {
                self.pos += 1;
                if self.current_token() != Some(&Token::OpenParen) {
                    return Err(errors::A15.to_owned());
                }
                self.pos += 1;
                let code = if let Some(Token::NumberValue(c)) = self.current_token().cloned() {
                    self.pos += 1;
                    c as i32
                } else {
                    return Err(errors::A26.to_owned());
                };
                if self.current_token() != Some(&Token::CloseParen) {
                    return Err(errors::A10.to_owned());
                }
                self.pos += 1;
                Ok(node!(self.line, AstKind::Exit(code)))
            }
            Some(Token::Number) | Some(Token::Float) | Some(Token::String) | Some(Token::Bool)
            | Some(Token::Const) => self.parse_var(),
            Some(Token::While) => {
                self.pos += 1;
                let expr = self.parse_value()?;
                self.next_line();

                let mut body = Vec::new();

                while self.current_token().cloned() != Some(Token::End) {
                    body.push(self.parse_main()?);
                    self.next_line();
                }

                Ok(node!(
                    self.line,
                    AstKind::While {
                        expr: Box::new(expr),
                        body,
                    }
                ))
            }
            Some(Token::For) => {
                self.pos += 1;

                let name = if let Some(Token::Ident(name)) = self.current_token().cloned() {
                    if name.is_empty() || !name.is_ascii() || name.len() > 256 {
                        Err(errors::A05.to_owned())
                    } else {
                        Ok(name)
                    }
                } else {
                    Err(errors::A15.to_owned())
                }?;

                self.pos += 1;
                if self.current_token() != Some(&Token::From) {
                    return Err(errors::A15.to_owned());
                }
                self.pos += 1;

                let start = self.parse_value()?;

                if self.current_token() != Some(&Token::To) {
                    return Err(errors::A15.to_owned());
                }
                self.pos += 1;

                let end = self.parse_value()?;

                let step = if self.current_token() == Some(&Token::Step) {
                    self.pos += 1;
                    Some(self.parse_value()?)
                } else {
                    None
                };

                let mut body = Vec::new();

                self.next_line();
                while self.current_token().cloned() != Some(Token::End) {
                    body.push(self.parse_main()?);
                    self.next_line();
                }

                Ok(node!(
                    self.line,
                    AstKind::For {
                        var_name: name,
                        start: Box::new(start),
                        end: Box::new(end),
                        step: Box::new(step),
                        body
                    }
                ))
            }
            Some(Token::If) => {
                self.pos += 1;
                let expr = self.parse_value()?;
                self.next_line();

                let mut body = Vec::new();

                while !matches!(
                    self.current_token().cloned(),
                    Some(Token::End) | Some(Token::Else)
                ) {
                    body.push(self.parse_main()?);
                    self.next_line();
                }

                match self.current_token().cloned() {
                    Some(Token::End) => Ok(node!(
                        self.line,
                        AstKind::Condition {
                            expr: Box::new(expr),
                            yes: body,
                            no: None,
                        }
                    )),
                    Some(Token::Else) => {
                        self.pos += 1;

                        match self.current_token().cloned() {
                            Some(Token::If) => Ok(node!(
                                self.line,
                                AstKind::Condition {
                                    expr: Box::new(expr),
                                    yes: body,
                                    no: Some(ElseBlock::ElseIf(Box::new(self.parse_main()?))),
                                }
                            )),
                            None => {
                                self.next_line();

                                let mut else_body = Vec::new();

                                while self.current_token().cloned() != Some(Token::End) {
                                    else_body.push(self.parse_main()?);
                                    self.next_line();
                                }

                                Ok(node!(
                                    self.line,
                                    AstKind::Condition {
                                        expr: Box::new(expr),
                                        yes: body,
                                        no: Some(ElseBlock::Else(else_body)),
                                    }
                                ))
                            }
                            _ => Err(errors::A15.to_owned()),
                        }
                    }
                    _ => Err(errors::A15.to_owned()),
                }
            }
            Some(Token::Ident(receiver)) => {
                let next = self.tokens.get(self.line).and_then(|l| l.get(self.pos + 1));
                match next {
                    Some(Token::Assign)
                    | Some(Token::PlusAssign)
                    | Some(Token::MinusAssign)
                    | Some(Token::MultiplyAssign)
                    | Some(Token::DivideAssign)
                    | Some(Token::ModAssign)
                    | Some(Token::BitAndAssign)
                    | Some(Token::BitOrAssign)
                    | Some(Token::BitXorAssign)
                    | Some(Token::BitRightAssign)
                    | Some(Token::BitLeftAssign) => {
                        self.pos += 1;
                        let op = self
                            .current_token()
                            .cloned()
                            .ok_or_else(|| errors::A15.to_owned())?;
                        self.pos += 1;
                        let expr = self.parse_value()?;
                        Ok(node!(
                            self.line,
                            AstKind::Assign {
                                name: receiver,
                                op,
                                expr: Box::new(expr),
                                var_type: TypesCheck::Number,
                            }
                        ))
                    }
                    Some(Token::OpenParen) => {
                        self.pos += 1;
                        let args = self.parse_call_args()?;
                        Ok(node!(
                            self.line,
                            AstKind::FuncCall {
                                name: receiver,
                                args
                            }
                        ))
                    }
                    _ => Err(errors::A15.to_owned()),
                }
            }
            _ => Err(errors::A20.to_owned()),
        }
    }
}
