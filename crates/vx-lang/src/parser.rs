use crate::ast::*;
use crate::error::LangError;
use crate::lexer::{LexError, Token};
use logos::{Logos, Span};

pub struct Parser<'s> {
    tokens: Vec<(Token<'s>, Span)>,
    cursor: usize,
}

impl<'s> Parser<'s> {
    pub fn new(src: &'s str) -> Result<Self, LangError> {
        let tokens = Token::lexer(src)
            .spanned()
            .map(|(res, span)| match res {
                Ok(tok) => Ok((tok, span)),
                Err(LexError) => Err(LangError::UnexpectedChar { offset: span.start }),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { tokens, cursor: 0 })
    }

    fn peek(&self) -> Option<&Token<'s>> {
        self.tokens.get(self.cursor).map(|(t, _)| t)
    }

    /// Peek at the token one position ahead without consuming.
    fn peek_next(&self) -> Option<&Token<'s>> {
        self.tokens.get(self.cursor + 1).map(|(t, _)| t)
    }

    fn span(&self) -> Span {
        self.tokens
            .get(self.cursor)
            .map(|(_, s)| s.clone())
            .unwrap_or(0..0)
    }

    fn advance(&mut self) -> Option<(Token<'s>, Span)> {
        if self.cursor < self.tokens.len() {
            let item = self.tokens[self.cursor].clone();
            self.cursor += 1;
            Some(item)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &Token<'s>) -> Result<Span, LangError> {
        match self.peek() {
            Some(t) if t == expected => {
                let (_, span) = self.advance().unwrap();
                Ok(span)
            }
            Some(_) => Err(LangError::ParseError {
                msg: format!("expected {:?}", expected),
                offset: self.span().start,
            }),
            None => Err(LangError::UnexpectedEof),
        }
    }

    pub fn parse_module(&mut self) -> Result<Module, LangError> {
        let mut decls = Vec::new();
        while self.peek().is_some() {
            decls.push(self.parse_decl()?);
        }
        Ok(Module { decls })
    }

    fn parse_decl(&mut self) -> Result<Spanned<Decl>, LangError> {
        let start = self.span().start;
        match self.peek() {
            Some(Token::Fn)  => self.parse_fn_decl(start),
            Some(Token::Let) => self.parse_top_let(start),
            Some(_) => {
                // Any other token: parse as a bare statement at module scope
                let stmt = self.parse_stmt()?;
                let span = stmt.span.clone();
                Ok(Spanned::new(Decl::Stmt(stmt), span))
            }
            None => Err(LangError::UnexpectedEof),
        }
    }

    fn parse_fn_decl(&mut self, start: usize) -> Result<Spanned<Decl>, LangError> {
        self.advance(); // consume `fn`
        let name = self.parse_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        let ret_ty = if matches!(self.peek(), Some(Token::Arrow)) {
            self.advance();
            Some(self.parse_ty()?)
        } else {
            None
        };
        let body = self.parse_block_expr()?;
        let end = body.span.end;
        Ok(Spanned::new(Decl::Fn { name, params, ret_ty, body }, start..end))
    }

    fn parse_top_let(&mut self, start: usize) -> Result<Spanned<Decl>, LangError> {
        self.advance(); // consume `let`
        let name = self.parse_ident()?;
        let ty = if matches!(self.peek(), Some(Token::Colon)) {
            self.advance();
            Some(self.parse_ty()?)
        } else {
            None
        };
        self.expect(&Token::Eq)?;
        let init = self.parse_expr()?;
        let end = self.expect(&Token::Semi)?.end;
        Ok(Spanned::new(Decl::Let { name, ty, init }, start..end))
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, LangError> {
        let mut params = Vec::new();
        while !matches!(self.peek(), Some(Token::RParen) | None) {
            if !params.is_empty() {
                self.expect(&Token::Comma)?;
            }
            let ps = self.span().start;
            let name = self.parse_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_ty()?;
            let pe = self.span().start;
            params.push(Param { name, ty, span: ps..pe });
        }
        Ok(params)
    }

    fn parse_ty(&mut self) -> Result<Ty, LangError> {
        match self.advance() {
            Some((Token::TyInt, _)) => Ok(Ty::Int),
            Some((Token::TyFloat, _)) => Ok(Ty::Float),
            Some((Token::TyBool, _)) => Ok(Ty::Bool),
            Some((Token::TyStr, _)) => Ok(Ty::Str),
            Some((Token::TyVec2, _)) => Ok(Ty::Vec2),
            Some((Token::TyVec3, _)) => Ok(Ty::Vec3),
            Some((Token::TyVec4, _)) => Ok(Ty::Vec4),
            Some((Token::TyMat4, _)) => Ok(Ty::Mat4),
            Some((Token::TyColor, _)) => Ok(Ty::Color),
            Some((Token::TyGeo, _)) => Ok(Ty::Geo),
            Some((Token::Ident(n), _)) => Ok(Ty::Named(n.to_string())),
            Some((_, span)) => Err(LangError::ParseError {
                msg: "expected type".into(),
                offset: span.start,
            }),
            None => Err(LangError::UnexpectedEof),
        }
    }

    fn parse_ident(&mut self) -> Result<String, LangError> {
        match self.advance() {
            Some((Token::Ident(n), _)) => Ok(n.to_string()),
            Some((_, span)) => Err(LangError::ParseError {
                msg: "expected identifier".into(),
                offset: span.start,
            }),
            None => Err(LangError::UnexpectedEof),
        }
    }

    fn parse_block_expr(&mut self) -> Result<Spanned<Expr>, LangError> {
        let start = self.expect(&Token::LBrace)?.start;
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Some(Token::RBrace) | None) {
            stmts.push(self.parse_stmt()?);
        }
        let end = self.expect(&Token::RBrace)?.end;
        Ok(Spanned::new(Expr::Block(stmts), start..end))
    }

    fn parse_stmt(&mut self) -> Result<Spanned<Stmt>, LangError> {
        let start = self.span().start;
        match self.peek() {
            Some(Token::Let) => {
                self.advance();
                let name = self.parse_ident()?;
                let ty = if matches!(self.peek(), Some(Token::Colon)) {
                    self.advance();
                    Some(self.parse_ty()?)
                } else {
                    None
                };
                let init = if matches!(self.peek(), Some(Token::Eq)) {
                    self.advance();
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                let end = self.expect(&Token::Semi)?.end;
                Ok(Spanned::new(Stmt::Let { name, ty, init }, start..end))
            }
            Some(Token::Return) => {
                self.advance();
                let val = if !matches!(self.peek(), Some(Token::Semi)) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                let end = self.expect(&Token::Semi)?.end;
                Ok(Spanned::new(Stmt::Return(val), start..end))
            }
            Some(Token::For) => {
                self.advance();
                let var = self.parse_ident()?;
                self.expect(&Token::In)?;
                let iter = self.parse_expr()?;
                let body = Box::new(self.parse_block_expr()?);
                let end = body.span.end;
                Ok(Spanned::new(Stmt::For { var, iter, body }, start..end))
            }
            _ if matches!(self.peek(), Some(Token::Ident(_)))
                && matches!(
                    self.peek_next(),
                    Some(Token::Eq | Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq)
                ) =>
            {
                let target_span = self.span();
                let name_str = self.parse_ident()?;
                let target = Spanned::new(Expr::Ident(name_str), target_span.clone());
                let op = match self.advance() {
                    Some((Token::Eq, _))      => AssignOp::Assign,
                    Some((Token::PlusEq, _))  => AssignOp::AddAssign,
                    Some((Token::MinusEq, _)) => AssignOp::SubAssign,
                    Some((Token::StarEq, _))  => AssignOp::MulAssign,
                    Some((Token::SlashEq, _)) => AssignOp::DivAssign,
                    _ => unreachable!(),
                };
                let value = self.parse_expr()?;
                let end = self.expect(&Token::Semi)?.end;
                Ok(Spanned::new(Stmt::Assign { target, op, value }, start..end))
            }
            _ => {
                let expr = self.parse_expr()?;
                let end = self.expect(&Token::Semi)?.end;
                Ok(Spanned::new(Stmt::Expr(expr), start..end))
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Spanned<Expr>, LangError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Spanned<Expr>, LangError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Some(Token::PipePipe)) {
            self.advance();
            let rhs = self.parse_and()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Spanned::new(
                Expr::Binary { op: BinOp::Or, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                span,
            );
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Spanned<Expr>, LangError> {
        let mut lhs = self.parse_cmp()?;
        while matches!(self.peek(), Some(Token::AmpAmp)) {
            self.advance();
            let rhs = self.parse_cmp()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Spanned::new(
                Expr::Binary { op: BinOp::And, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                span,
            );
        }
        Ok(lhs)
    }

    fn parse_cmp(&mut self) -> Result<Spanned<Expr>, LangError> {
        let mut lhs = self.parse_add()?;
        loop {
            let op = match self.peek() {
                Some(Token::EqEq) => BinOp::Eq,
                Some(Token::BangEq) => BinOp::NotEq,
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::LtEq) => BinOp::LtEq,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::GtEq) => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_add()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Spanned::new(
                Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                span,
            );
        }
        Ok(lhs)
    }

    fn parse_add(&mut self) -> Result<Spanned<Expr>, LangError> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_mul()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Spanned::new(
                Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                span,
            );
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Spanned<Expr>, LangError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Rem,
                Some(Token::StarStar) => BinOp::Pow,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_unary()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Spanned::new(
                Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                span,
            );
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Spanned<Expr>, LangError> {
        let start = self.span().start;
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_postfix()?;
                let end = expr.span.end;
                Ok(Spanned::new(
                    Expr::Unary { op: UnOp::Neg, expr: Box::new(expr) },
                    start..end,
                ))
            }
            Some(Token::Bang) => {
                self.advance();
                let expr = self.parse_postfix()?;
                let end = expr.span.end;
                Ok(Spanned::new(
                    Expr::Unary { op: UnOp::Not, expr: Box::new(expr) },
                    start..end,
                ))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Spanned<Expr>, LangError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(Token::LParen) => {
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.peek(), Some(Token::RParen) | None) {
                        if !args.is_empty() {
                            self.expect(&Token::Comma)?;
                        }
                        args.push(self.parse_expr()?);
                    }
                    let end = self.expect(&Token::RParen)?.end;
                    let span = expr.span.start..end;
                    expr = Spanned::new(
                        Expr::Call { callee: Box::new(expr), args },
                        span,
                    );
                }
                Some(Token::LBracket) => {
                    self.advance();
                    let idx = self.parse_expr()?;
                    let end = self.expect(&Token::RBracket)?.end;
                    let span = expr.span.start..end;
                    expr = Spanned::new(
                        Expr::Index { obj: Box::new(expr), idx: Box::new(idx) },
                        span,
                    );
                }
                Some(Token::Dot) => {
                    self.advance();
                    let name = self.parse_ident()?;
                    let end = self.span().start;
                    let span = expr.span.start..end;
                    expr = Spanned::new(
                        Expr::Field { obj: Box::new(expr), name },
                        span,
                    );
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Spanned<Expr>, LangError> {
        let start = self.span().start;
        match self.advance() {
            Some((Token::Int(s), span)) => {
                let v = s.parse::<i64>().map_err(|e| LangError::BadInt {
                    text: s.to_string(),
                    span: span.clone(),
                    source: e,
                })?;
                Ok(Spanned::new(Expr::Lit(Lit::Int(v)), span))
            }
            Some((Token::Float(s), span)) => {
                let v = s.parse::<f64>().map_err(|e| LangError::BadFloat {
                    text: s.to_string(),
                    span: span.clone(),
                    source: e,
                })?;
                Ok(Spanned::new(Expr::Lit(Lit::Float(v)), span))
            }
            Some((Token::Str(s), span)) => {
                let inner = &s[1..s.len() - 1];
                Ok(Spanned::new(Expr::Lit(Lit::Str(inner.to_string())), span))
            }
            Some((Token::True, span)) => Ok(Spanned::new(Expr::Lit(Lit::Bool(true)), span)),
            Some((Token::False, span)) => Ok(Spanned::new(Expr::Lit(Lit::Bool(false)), span)),
            Some((Token::Null, span)) => Ok(Spanned::new(Expr::Lit(Lit::Null), span)),
            Some((Token::Ident(n), span)) => {
                Ok(Spanned::new(Expr::Ident(n.to_string()), span))
            }
            Some((Token::LParen, _)) => {
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Some((Token::LBrace, _)) => {
                self.cursor -= 1;
                self.parse_block_expr()
            }
            Some((Token::If, _)) => self.parse_if_expr(start),
            Some((Token::Text2d, _)) => self.parse_text_props(start, false),
            Some((Token::Text3d, _)) => self.parse_text_props(start, true),
            Some((_, span)) => Err(LangError::ParseError {
                msg: "unexpected token in expression".into(),
                offset: span.start,
            }),
            None => Err(LangError::UnexpectedEof),
        }
    }

    fn parse_if_expr(&mut self, start: usize) -> Result<Spanned<Expr>, LangError> {
        let cond = self.parse_expr()?;
        let then_block = self.parse_block_expr()?;
        let else_block = if matches!(self.peek(), Some(Token::Else)) {
            self.advance();
            Some(Box::new(if matches!(self.peek(), Some(Token::If)) {
                self.advance();
                let s = self.span().start;
                self.parse_if_expr(s)?
            } else {
                self.parse_block_expr()?
            }))
        } else {
            None
        };
        let end = else_block
            .as_ref()
            .map(|e| e.span.end)
            .unwrap_or(then_block.span.end);
        Ok(Spanned::new(
            Expr::If {
                cond: Box::new(cond),
                then_block: Box::new(then_block),
                else_block,
            },
            start..end,
        ))
    }

    fn parse_text_props(&mut self, start: usize, is_3d: bool) -> Result<Spanned<Expr>, LangError> {
        self.expect(&Token::LBrace)?;
        let mut fields = std::collections::HashMap::new();
        while !matches!(self.peek(), Some(Token::RBrace) | None) {
            if !fields.is_empty() {
                if matches!(self.peek(), Some(Token::Comma)) {
                    self.advance();
                }
                if matches!(self.peek(), Some(Token::RBrace)) {
                    break;
                }
            }
            // Keys are identifiers; `color` is a type keyword so accept it too
            let key = match self.advance() {
                Some((Token::Ident(n), _)) => n.to_string(),
                Some((Token::TyColor, _)) => "color".to_string(),
                Some((_, span)) => return Err(LangError::ParseError {
                    msg: "expected property name in text block".into(),
                    offset: span.start,
                }),
                None => return Err(LangError::UnexpectedEof),
            };
            self.expect(&Token::Colon)?;
            let val = self.parse_expr()?;
            fields.insert(key, val);
        }
        let end = self.expect(&Token::RBrace)?.end;
        let props = Box::new(TextProps { fields });
        let node = if is_3d { Expr::Text3d(props) } else { Expr::Text2d(props) };
        Ok(Spanned::new(node, start..end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Module {
        Parser::new(src).unwrap().parse_module().expect("parse should succeed")
    }

    #[test]
    fn top_level_let() {
        let m = parse("let x: int = 42;");
        assert_eq!(m.decls.len(), 1);
        match &m.decls[0].node {
            Decl::Let { name, ty, init } => {
                assert_eq!(name, "x");
                assert_eq!(ty.as_ref().unwrap(), &Ty::Int);
                assert_eq!(init.node, Expr::Lit(Lit::Int(42)));
            }
            _ => panic!("expected Decl::Let"),
        }
    }

    #[test]
    fn fn_no_args() {
        let m = parse("fn pi() -> float { 3.14; }");
        match &m.decls[0].node {
            Decl::Fn { name, params, ret_ty, .. } => {
                assert_eq!(name, "pi");
                assert!(params.is_empty());
                assert_eq!(ret_ty.as_ref().unwrap(), &Ty::Float);
            }
            _ => panic!("expected Decl::Fn"),
        }
    }

    #[test]
    fn fn_with_params() {
        let m = parse("fn add(a: int, b: int) -> int { a; }");
        match &m.decls[0].node {
            Decl::Fn { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[0].ty, Ty::Int);
            }
            _ => panic!("expected Decl::Fn"),
        }
    }

    #[test]
    fn binary_add() {
        let m = parse("let r: int = 1 + 2;");
        match &m.decls[0].node {
            Decl::Let { init, .. } => match &init.node {
                Expr::Binary { op, .. } => assert_eq!(op, &BinOp::Add),
                _ => panic!("expected binary"),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn text2d_parses() {
        let m = parse(r#"let t: geo = text2d { content: "Hello", size: 24.0 };"#);
        match &m.decls[0].node {
            Decl::Let { init, .. } => assert!(matches!(init.node, Expr::Text2d(_))),
            _ => panic!("expected text2d"),
        }
    }

    #[test]
    fn text3d_parses() {
        let m = parse(r#"let t: geo = text3d { content: "VX", size: 1.0, depth: 0.2 };"#);
        match &m.decls[0].node {
            Decl::Let { init, .. } => assert!(matches!(init.node, Expr::Text3d(_))),
            _ => panic!("expected text3d"),
        }
    }
}
