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

    Int(i64),
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
    ArrayLiteral {
        values: Vec<AstKind>,
    },
    ArrayFill {
        size: Box<AstKind>,
        value: Box<AstKind>,
    },
    ArraySet {
        name: String,
        index: Box<AstKind>,
        op: Token,
        expr: Box<AstKind>,
        elem_type: TypesCheck,
    },
    ArrayGet {
        name: String,
        index: Box<AstKind>,
    },
    StructDecl {
        name: String,
        fields: Vec<(String, TypesCheck)>,
    },
    StructShortLiteral {
        fields: Vec<AstKind>,
    },
    StructLongLiteral {
        fields: Vec<(String, Box<AstKind>)>,
    },
    StructSet {
        name: String,
        member: String,
        op: Token,
        expr: Box<AstKind>,
        elem_type: TypesCheck,
    },
    StructGet {
        name: String,
        index: String,
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
    Int,
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

    fn peek_token(&self, offset: usize) -> Option<&Token> {
        self.tokens
            .get(self.line)
            .and_then(|line| line.get(self.pos + offset))
    }

    fn next_line(&mut self) {
        self.line += 1;
        self.pos = 0;
    }

    fn parse(&mut self) -> Result<AstNode, String> {
        match self.current_token() {
            Some(Token::Func) => self.parse_func(),
            Some(Token::Struct) => self.parse_struct_decl(),
            Some(Token::Const) => {
                let result = self.parse_var();
                self.next_line();
                result
            }
            _ => Err(errors::A20.to_owned()),
        }
    }

    fn parse_type(&mut self) -> Result<TypesCheck, String> {
        let mut type_ = match self.current_token().cloned() {
            Some(Token::Int) => TypesCheck::Int,
            Some(Token::Float) => TypesCheck::Float,
            Some(Token::String) => TypesCheck::String,
            Some(Token::Bool) => TypesCheck::Boolean,
            Some(Token::Ident(name)) => TypesCheck::Struct(name),
            _ => return Err(errors::A15.to_owned()),
        };

        self.pos += 1;
        if self.current_token() == Some(&Token::OpenBracket) {
            self.pos += 1;
            if self.current_token() != Some(&Token::CloseBracket) {
                return Err(errors::A15.to_owned());
            }
            self.pos += 1;
            type_ = TypesCheck::Array(Box::new(type_));
        }

        Ok(type_)
    }

    fn parse_return_type(&mut self) -> Result<ReturnType, String> {
        let mut type_ = match self.current_token().cloned() {
            Some(Token::Void) => ReturnType::Void,
            Some(Token::Int) => ReturnType::Int,
            Some(Token::Float) => ReturnType::Float,
            Some(Token::String) => ReturnType::String,
            Some(Token::Bool) => ReturnType::Boolean,
            Some(Token::Ident(name)) => ReturnType::Struct(name),
            _ => return Err(errors::A15.to_owned()),
        };

        self.pos += 1;
        if self.current_token() == Some(&Token::OpenBracket) {
            self.pos += 1;
            if self.current_token() != Some(&Token::CloseBracket) {
                return Err(errors::A15.to_owned());
            }
            self.pos += 1;
            type_ = ReturnType::Array(Box::new(type_));
        }

        Ok(type_)
    }

    fn is_struct_decl_lookahead(&self) -> bool {
        if !matches!(self.current_token(), Some(Token::Ident(_))) {
            return false;
        }

        matches!(self.peek_token(1), Some(Token::Ident(_)))
            || (matches!(self.peek_token(1), Some(Token::OpenBracket))
                && matches!(self.peek_token(2), Some(Token::CloseBracket))
                && matches!(self.peek_token(3), Some(Token::Ident(_))))
    }

    fn parse_struct_decl(&mut self) -> Result<AstNode, String> {
        let struct_line = self.line;
        self.pos += 1;

        let name = if let Some(Token::Ident(n)) = self.current_token().cloned() {
            self.pos += 1;
            n
        } else {
            return Err(errors::A15.to_owned());
        };

        let mut fields = Vec::new();

        self.next_line();
        while self.current_token().cloned() != Some(Token::End) {
            let type_ = self.parse_type()?;

            let field_name = if let Some(Token::Ident(n)) = self.current_token().cloned() {
                self.pos += 1;
                n
            } else {
                return Err(errors::A15.to_owned());
            };

            fields.push((field_name, type_));
            self.next_line();
        }
        self.next_line();

        Ok(node!(struct_line, AstKind::StructDecl { name, fields }))
    }

    fn parse_struct_literal(&mut self) -> Result<AstKind, String> {
        if self.current_token() != Some(&Token::OpenParen) {
            return Err(errors::A15.to_owned());
        }
        self.pos += 1;

        let is_multiline_long = self.current_token().is_none();
        if is_multiline_long {
            self.next_line();
        }

        let is_long = is_multiline_long
            || (matches!(self.current_token(), Some(Token::Ident(_)))
                && matches!(self.peek_token(1), Some(Token::Colon)));

        if is_long {
            let mut fields = Vec::new();

            loop {
                if self.current_token() == Some(&Token::CloseParen) {
                    break;
                }

                let field_name = if let Some(Token::Ident(n)) = self.current_token().cloned() {
                    self.pos += 1;
                    n
                } else {
                    return Err(errors::A15.to_owned());
                };

                if self.current_token() != Some(&Token::Colon) {
                    return Err(errors::A15.to_owned());
                }
                self.pos += 1;

                let value = self.parse_value()?;
                fields.push((field_name, Box::new(value)));

                if self.current_token() == Some(&Token::CloseParen) {
                    break;
                }

                self.next_line();
            }

            self.pos += 1;
            Ok(AstKind::StructLongLiteral { fields })
        } else {
            let mut fields = Vec::new();

            if self.current_token() != Some(&Token::CloseParen) {
                fields.push(self.parse_value()?);

                while self.current_token() == Some(&Token::Comma) {
                    self.pos += 1;
                    fields.push(self.parse_value()?);
                }
            }

            if self.current_token() != Some(&Token::CloseParen) {
                return Err(errors::A10.to_owned());
            }
            self.pos += 1;

            Ok(AstKind::StructShortLiteral { fields })
        }
    }

    fn parse_array(&mut self) -> Result<AstKind, String> {
        if self.current_token() != Some(&Token::OpenBracket) {
            return Err(errors::A15.to_owned());
        }

        let mut values = Vec::new();

        while !matches!(
            self.current_token(),
            Some(Token::CloseBracket) | Some(Token::Semicolon)
        ) {
            self.pos += 1;
            values.push(self.parse_value()?);
        }

        if self.current_token() == Some(&Token::CloseBracket) {
            self.pos += 1;

            Ok(AstKind::ArrayLiteral { values })
        } else if values.len() == 1 {
            self.pos += 1;

            Ok(AstKind::ArrayFill {
                size: Box::new(values[0].clone()),
                value: Box::new(self.parse_value()?),
            })
        } else {
            Err(errors::A15.to_owned())
        }
    }

    fn parse_var(&mut self) -> Result<AstNode, String> {
        let is_const = self.current_token() == Some(&Token::Const);

        if is_const {
            self.pos += 1;
        }

        let type_ = self.parse_type()?;

        let array = matches!(type_, TypesCheck::Array(_));
        let is_struct = matches!(type_, TypesCheck::Struct(_))
            || matches!(&type_, TypesCheck::Array(inner) if matches!(**inner, TypesCheck::Struct(_)));

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
                        value: Box::new(
                            if array && self.current_token() == Some(&Token::OpenBracket) {
                                self.parse_array()?
                            } else if is_struct && self.current_token() == Some(&Token::OpenParen) {
                                self.parse_struct_literal()?
                            } else {
                                self.parse_value()?
                            }
                        ),
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
        if self.current_token() == Some(&Token::OpenBracket) {
            self.parse_array()
        } else {
            self.parse_expr(0)
        }
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
                    expr_type: TypesCheck::Int,
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
                } else if self.current_token() == Some(&Token::OpenBracket) {
                    self.pos += 1;
                    let index = self.parse_value()?;

                    if self.current_token() != Some(&Token::CloseBracket) {
                        return Err(errors::A15.to_owned());
                    }

                    self.pos += 1;
                    AstKind::ArrayGet {
                        name,
                        index: Box::new(index),
                    }
                } else if self.current_token() == Some(&Token::Dot) {
                    self.pos += 1;
                    let member = if let Some(Token::Ident(m)) = self.current_token().cloned() {
                        self.pos += 1;
                        m
                    } else {
                        return Err(errors::A15.to_owned());
                    };

                    AstKind::StructGet {
                        name,
                        index: member,
                    }
                } else {
                    AstKind::Ident(name)
                }
            }
            Some(Token::Int) => {
                self.pos += 1;
                AstKind::Ident("int".to_owned())
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
            Some(Token::IntValue(v)) => {
                self.pos += 1;
                AstKind::Int(v)
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
                            "int" => Cast::Int,
                            "float" => Cast::Float,
                            "bool" => Cast::Boolean,
                            _ => return Err(errors::A15.to_owned()),
                        }
                    } else {
                        return Err(errors::A15.to_owned());
                    },
                    src_type: TypesCheck::Int,
                };
            } else {
                left = AstKind::BinaryOp {
                    left: Box::new(left),
                    op: op_tok,
                    right: Box::new(right),
                    left_type: TypesCheck::Int,
                    right_type: TypesCheck::Int,
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
            let type_ = self.parse_type()?;

            let arg = if let Some(Token::Ident(a)) = self.current_token().cloned() {
                self.pos += 1;
                a
            } else {
                return Err(errors::A15.to_owned());
            };

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

        let return_type = self.parse_return_type()?;

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
            args.push(if self.current_token() == Some(&Token::OpenBracket) {
                self.parse_array()?
            } else {
                self.parse_expr(0)?
            });

            while self.current_token() == Some(&Token::Comma) {
                self.pos += 1;
                args.push(if self.current_token() == Some(&Token::OpenBracket) {
                    self.parse_array()?
                } else {
                    self.parse_expr(0)?
                });
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
                        AstKind::Return(Some(Box::new(
                            if self.current_token() == Some(&Token::OpenBracket) {
                                self.parse_array()?
                            } else {
                                self.parse_expr(0)?
                            }
                        )))
                    ))
                }
            }
            Some(Token::Exit) => {
                self.pos += 1;
                if self.current_token() != Some(&Token::OpenParen) {
                    return Err(errors::A15.to_owned());
                }
                self.pos += 1;
                let code = if let Some(Token::IntValue(c)) = self.current_token().cloned() {
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
            Some(Token::Int) | Some(Token::Float) | Some(Token::String) | Some(Token::Bool)
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
            Some(Token::Ident(_)) if self.is_struct_decl_lookahead() => self.parse_var(),
            Some(Token::Ident(receiver)) => {
                let mut next = self
                    .tokens
                    .get(self.line)
                    .and_then(|l| l.get(self.pos + 1))
                    .cloned();

                let mut index = None;
                let mut member = None;
                if next == Some(Token::OpenBracket) {
                    self.pos += 2;

                    index = Some(self.parse_value()?);
                    if self.current_token() != Some(&Token::CloseBracket) {
                        return Err(errors::A15.to_owned());
                    }

                    self.pos += 1;
                    next = self.current_token().cloned();
                    self.pos -= 1;
                } else if next == Some(Token::Dot) {
                    self.pos += 2;

                    member = if let Some(Token::Ident(m)) = self.current_token().cloned() {
                        self.pos += 1;
                        Some(m)
                    } else {
                        return Err(errors::A15.to_owned());
                    };

                    next = self.current_token().cloned();
                    self.pos -= 1;
                }

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
                            if let Some(index) = index {
                                AstKind::ArraySet {
                                    name: receiver,
                                    index: Box::new(index),
                                    op,
                                    expr: Box::new(expr),
                                    elem_type: TypesCheck::Int,
                                }
                            } else if let Some(member) = member {
                                AstKind::StructSet {
                                    name: receiver,
                                    member,
                                    op,
                                    expr: Box::new(expr),
                                    elem_type: TypesCheck::Int,
                                }
                            } else {
                                AstKind::Assign {
                                    name: receiver,
                                    op,
                                    expr: Box::new(expr),
                                    var_type: TypesCheck::Int,
                                }
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
