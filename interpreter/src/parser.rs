// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{errors, file_handler::FileHandler, lexer::Token, types::TypesCheck};

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

    Proc {
        name: String,
        body: Vec<AstNode>,
        args: Vec<(String, TypesCheck)>,
    },
    VarDecl {
        name: String,
        type_: TypesCheck,
        value: Box<AstKind>,
        is_const: bool,
    },
    ProcCall {
        name: String,
        args: Vec<Token>,
    },
    // MethodCall {
    //     receiver: String,
    //     name: String,
    //     args: Vec<AstKind>,
    // },
    Assign {
        name: String,
        op: Token,
        expr: Box<AstKind>,
    },
    BinaryOp {
        left: Box<AstKind>,
        op: Token,
        right: Box<AstKind>,
    },
    UnaryOp {
        op: Token,
        expr: Box<AstKind>,
    },
    AsOp {
        expr: Box<AstKind>,
        op: Cast,
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
    Sin,
    Cos,
    Sqrt,
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
            Some(Token::Proc) => self.parse_proc(),
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
                None | Some(Token::To) | Some(Token::Step)
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
            Some(Token::Minus) | Some(Token::Not) => {
                let op = self.current_token().unwrap().clone();

                self.pos += 1;
                let expr = self.parse_expr(6)?;

                AstKind::UnaryOp {
                    op,
                    expr: Box::new(expr),
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
                AstKind::Ident(name)
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
                            "sin" => Cast::Sin,
                            "cos" => Cast::Cos,
                            "sqrt" => Cast::Sqrt,
                            _ => return Err(errors::A15.to_owned()),
                        }
                    } else {
                        return Err(errors::A15.to_owned());
                    },
                };
            } else {
                left = AstKind::BinaryOp {
                    left: Box::new(left),
                    op: op_tok,
                    right: Box::new(right),
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
            _ => None,
        }
    }

    fn parse_proc(&mut self) -> Result<AstNode, String> {
        self.pos += 1;

        if let Some(Token::Ident(name)) = self.current_token().cloned() {
            self.pos += 1;

            let mut args = Vec::new();

            while self.pos < self.tokens[self.line].len() {
                match self.current_token().cloned() {
                    Some(Token::Ident(arg)) => {
                        self.pos += 1;
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

                        args.push((arg, type_));
                    }
                    _ => return Err(errors::A15.to_owned()),
                }

                self.pos += 1;
            }

            let mut body = Vec::new();

            self.next_line();
            while Some(Token::End) != self.current_token().cloned() {
                body.push(self.parse_main()?);
                self.next_line();
            }

            self.next_line();
            Ok(node!(self.line, AstKind::Proc { name, body, args }))
        } else {
            Err(errors::A15.to_owned())
        }
    }

    fn parse_main(&mut self) -> Result<AstNode, String> {
        match self.current_token().cloned() {
            Some(Token::Echo) => {
                self.pos += 1;
                Ok(node!(
                    self.line,
                    AstKind::Echo(Box::new(self.parse_value()?))
                ))
            }
            Some(Token::Delete) => {
                self.pos += 1;

                if let Some(Token::Ident(name)) = self.current_token().cloned() {
                    self.pos += 1;
                    Ok(node!(self.line, AstKind::Delete(name)))
                } else {
                    Err(errors::A29.to_owned())
                }
            }
            Some(Token::Exit) => {
                self.pos += 1;

                if let Some(Token::NumberValue(code)) = self.current_token().cloned() {
                    self.pos += 1;
                    Ok(node!(self.line, AstKind::Exit(code as i32)))
                } else {
                    Err(errors::A26.to_owned())
                }
            }
            Some(Token::Number) | Some(Token::Float) | Some(Token::String) | Some(Token::Bool)
            | Some(Token::Const) => self.parse_var(),
            Some(Token::Call) => {
                self.pos += 1;
                if let Some(Token::Ident(name)) = self.current_token().cloned() {
                    self.pos += 1;

                    let mut args = Vec::new();

                    while self.pos < self.tokens[self.line].len() {
                        let current = self.current_token().cloned();

                        if matches!(
                            current,
                            Some(Token::Ident(_))
                                | Some(Token::NumberValue(_))
                                | Some(Token::FloatValue(_))
                                | Some(Token::StringValue(_))
                                | Some(Token::BooleanValue(_))
                        ) {
                            args.push(current.unwrap());
                        } else {
                            return Err(errors::A15.to_owned());
                        }

                        self.pos += 1;
                    }

                    Ok(node!(self.line, AstKind::ProcCall { name, args }))
                } else {
                    Err(errors::A15.to_owned())
                }
            }
            Some(Token::While) => {
                self.pos += 1;
                let expr = self.parse_value()?;
                self.next_line();

                let mut body = Vec::new();

                while self.current_token().cloned() != Some(Token::EndWhile) {
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
                while self.current_token().cloned() != Some(Token::EndFor) {
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
                    Some(Token::EndIf) | Some(Token::Else)
                ) {
                    body.push(self.parse_main()?);
                    self.next_line();
                }

                match self.current_token().cloned() {
                    Some(Token::EndIf) => Ok(node!(
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

                                while self.current_token().cloned() != Some(Token::EndIf) {
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
            Some(Token::Ident(receiver)) => match self
                .tokens
                .get(self.line)
                .and_then(|line| line.get(self.pos + 1))
                .ok_or(errors::A15.to_owned())?
                .clone()
            {
                Token::Ident(_) => {
                    self.pos += 2;

                    let mut args = Vec::new();

                    while self.pos < self.tokens[self.line].len() {
                        match self.current_token().cloned() {
                            Some(Token::NumberValue(arg)) => args.push(AstKind::Number(arg)),
                            Some(Token::FloatValue(arg)) => args.push(AstKind::Float(arg)),
                            Some(Token::BooleanValue(arg)) => args.push(AstKind::Boolean(arg)),
                            Some(Token::StringValue(arg)) => args.push(AstKind::String(arg)),
                            _ => return Err(errors::A15.to_owned()),
                        }

                        self.pos += 1;
                    }

                    // Ok(node!(
                    //     self.line,
                    //     AstKind::MethodCall {
                    //         receiver: receiver.clone(),
                    //         name: name.clone(),
                    //         args,
                    //     }
                    // ))
                    todo!()
                }
                Token::Assign
                | Token::PlusAssign
                | Token::MinusAssign
                | Token::MultiplyAssign
                | Token::DivideAssign
                | Token::ModAssign => {
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
                        }
                    ))
                }
                _ => Err(errors::A15.to_owned()),
            },
            _ => Err(errors::A20.to_owned()),
        }
    }
}
