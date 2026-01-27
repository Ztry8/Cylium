// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{errors, lexer::Token, show_error, types::Types};

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
    Proc {
        name: String,
        body: Vec<AstNode>,
        args: Vec<String>,
    },
    VarDecl {
        name: String,
        value: Box<AstKind>,
        is_const: bool,
    },
    Echo(Box<AstKind>),
    Exit(i32),
    ProcCall {
        name: String,
        args: Vec<Token>,
    },
    MethodCall {
        receiver: String,
        name: String,
        args: Vec<Types>,
    },
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
    Value(Types),
    Ident(String),
    Condition {
        expr: Box<AstKind>,
        yes: Vec<AstNode>,
        no: Option<ElseBlock>,
    },
    While {
        expr: Box<AstKind>,
        body: Vec<AstNode>,
    },
}

#[derive(Debug, Clone)]
pub enum ElseBlock {
    ElseIf(Box<AstNode>),
    Else(Vec<AstNode>),
}

pub struct Parser {
    tokens: Vec<Vec<Token>>,
    file: Vec<String>,
    line: usize,
    pos: usize,
}

impl Parser {
    pub fn new(file: Vec<String>, tokens: Vec<Vec<Token>>) -> Self {
        Self {
            line: 0,
            pos: 0,
            tokens,
            file,
        }
    }

    pub fn start(&mut self) -> Vec<AstNode> {
        let mut tree = Vec::new();

        while self.line < self.tokens.len() {
            tree.push(
                self.parse()
                    .unwrap_or_else(|e| show_error(self.line, &self.file[self.line], &e)),
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
        self.pos += 1;

        if let Some(Token::Ident(name)) = self.current_token().cloned() {
            self.pos += 1;

            if name.is_empty()
                || !name.is_ascii()
                || name.len() > 256
            {
                return Err(errors::A05.to_owned());
            }

            if self.current_token() == Some(&Token::Assign) {
                self.pos += 1;

                Ok(node!(
                    self.line,
                    AstKind::VarDecl {
                        name,
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
            if self.current_token().is_some() {
                return Err(errors::A15.to_owned());
            }

            return Ok(first);
        }

        let mut values = Vec::new();
        match first {
            AstKind::Value(v) => values.push(v),
            _ => return Err(errors::A19.to_owned()),
        };

        while self.current_token() == Some(&Token::Comma) {
            self.pos += 1;

            match self.parse_expr(0)? {
                AstKind::Value(v) => values.push(v),
                _ => return Err(errors::A19.to_owned()),
            };
        }

        if self.current_token().is_some() {
            return Err(errors::A15.to_owned());
        }

        Ok(AstKind::Value(Types::Vector(values)))
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
            Some(Token::Value(v)) => {
                self.pos += 1;
                AstKind::Value(v)
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

            left = AstKind::BinaryOp {
                left: Box::new(left),
                op: op_tok,
                right: Box::new(right),
            };
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
                    Some(Token::Ident(arg)) => args.push(arg),
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
            Some(Token::Exit) => {
                self.pos += 1;

                if let Some(Token::Value(Types::Number(code))) = self.current_token().cloned() {
                    self.pos += 1;
                    Ok(node!(self.line, AstKind::Exit(code)))
                } else {
                    Err(errors::A26.to_owned())
                }
            }
            Some(Token::Var) | Some(Token::Const) => self.parse_var(),
            Some(Token::Call) => {
                self.pos += 1;
                if let Some(Token::Ident(name)) = self.current_token().cloned() {
                    self.pos += 1;

                    let mut args = Vec::new();

                    while self.pos < self.tokens[self.line].len() {
                        let current = self.current_token().cloned();

                        if matches!(current, Some(Token::Ident(_)) | Some(Token::Value(_))) {
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
                Token::Ident(name) => {
                    self.pos += 2;

                    let mut args = Vec::new();

                    while self.pos < self.tokens[self.line].len() {
                        match self.current_token().cloned() {
                            Some(Token::Value(arg)) => args.push(arg),
                            _ => return Err(errors::A15.to_owned()),
                        }

                        self.pos += 1;
                    }

                    Ok(node!(
                        self.line,
                        AstKind::MethodCall {
                            receiver: receiver.clone(),
                            name: name.clone(),
                            args,
                        }
                    ))
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
