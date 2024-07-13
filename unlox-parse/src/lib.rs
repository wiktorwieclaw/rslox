//! # Expression grammar:
//! program        → declaration* EOF ;
//!
//! declaration    → var_decl | statement ;
//!
//! statement      → expr_stmt | print_stmt ;
//!
//! expr_stmt      → expression ";" ;
//! print_stmt     → "print" expression ";" ;
//!
//! var_decl       → "var" IDENTIFIER ( "=" expression )? ";" ;
//! expression     → equality ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary | primary ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER;

use unlox_ast::{
    tokens::{TokenStream, TokenStreamExt},
    Expr, Lit, Stmt, Token, TokenKind,
};

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Error {
    pub token: Token,
    pub message: String,
}

impl Error {
    fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub fn parse(mut stream: impl TokenStream) -> Result<Vec<Stmt>> {
    let mut stmts = vec![];
    while !stream.eof() {
        if let Some(stmt) = declaration(&mut stream) {
            stmts.push(stmt);
        }
    }
    Ok(stmts)
}

fn declaration(stream: &mut impl TokenStream) -> Option<Stmt> {
    let token = stream.peek();
    let result = match &token.kind {
        TokenKind::Var => {
            stream.next();
            var_declaration(stream)
        }
        _ => statement(stream),
    };
    result.inspect_err(|e| eprintln!("{e}")).ok().or_else(|| {
        synchronize(stream);
        None
    })
}

fn statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    let token = stream.peek();
    match &token.kind {
        TokenKind::Print => {
            stream.next();
            print_statement(stream)
        }
        _ => expression_statement(stream),
    }
}

fn print_statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    let expr = expression(stream)?;
    consume(stream, TokenKind::Semicolon, "Expected ';' after value.")?;
    Ok(Stmt::Print(expr))
}

fn expression_statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    let expr = expression(stream)?;
    consume(
        stream,
        TokenKind::Semicolon,
        "Expected ';' after expression.",
    )?;
    Ok(Stmt::Expression(expr))
}

fn var_declaration(stream: &mut impl TokenStream) -> Result<Stmt> {
    let name = consume(stream, TokenKind::Identifier, "Expected variable name.")?;
    let token = stream.peek();
    let init = if token.kind == TokenKind::Equal {
        stream.next();
        Some(expression(stream)?)
    } else {
        None
    };
    consume(
        stream,
        TokenKind::Semicolon,
        "Expected ';' after variable declaration.",
    )?;
    Ok(Stmt::VarDecl { name, init })
}

fn expression(stream: &mut impl TokenStream) -> Result<Expr> {
    equality(stream)
}

fn equality(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = comparison(stream)?;
    while let TokenKind::BangEqual | TokenKind::EqualEqual = stream.peek().kind {
        let token = stream.next();
        expr = Expr::Binary(token, Box::new(expr), Box::new(comparison(stream)?));
    }
    Ok(expr)
}

fn comparison(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = term(stream)?;
    while let TokenKind::Less
    | TokenKind::LessEqual
    | TokenKind::Greater
    | TokenKind::GreaterEqual = stream.peek().kind
    {
        let token = stream.next();
        expr = Expr::Binary(token, Box::new(expr), Box::new(term(stream)?));
    }
    Ok(expr)
}

fn term(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = factor(stream)?;
    while let TokenKind::Minus | TokenKind::Plus = stream.peek().kind {
        let token = stream.next();
        expr = Expr::Binary(token, Box::new(expr), Box::new(factor(stream)?));
    }
    Ok(expr)
}

fn factor(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = unary(stream)?;
    while let TokenKind::Slash | TokenKind::Star = stream.peek().kind {
        let token = stream.next();
        expr = Expr::Binary(token, Box::new(expr), Box::new(unary(stream)?));
    }
    Ok(expr)
}

fn unary(stream: &mut impl TokenStream) -> Result<Expr> {
    match stream.peek().kind {
        TokenKind::Bang | TokenKind::Minus => {
            let token = stream.next();
            let expr = Expr::Unary(token, Box::new(unary(stream)?));
            Ok(expr)
        }
        _ => primary(stream),
    }
}

fn primary(stream: &mut impl TokenStream) -> Result<Expr> {
    let token = stream.peek();
    let expr = match &token.kind {
        TokenKind::False => Expr::Literal(Lit::Bool(false)),
        TokenKind::True => Expr::Literal(Lit::Bool(true)),
        TokenKind::Nil => Expr::Literal(Lit::Nil),
        TokenKind::Number(n) => Expr::Literal(Lit::Number(*n)),
        TokenKind::String {
            value,
            is_terminated: true,
        } => Expr::Literal(Lit::String(value.clone())),
        TokenKind::String {
            is_terminated: false,
            ..
        } => {
            return Err(Error::new(token.clone(), "Unterminated string.".into()));
        }
        TokenKind::LeftParen => {
            stream.next();
            let expr = expression(stream)?;
            let token = stream.peek();
            if token.kind != TokenKind::RightParen {
                return Err(Error::new(
                    token.clone(),
                    r#"Expected ")" after expression."#.into(),
                ));
            }
            Expr::Grouping(Box::new(expr))
        }
        TokenKind::Identifier => Expr::Variable(token.clone()),
        TokenKind::Eof => {
            return Err(Error::new(
                token.clone(),
                "Unexpected end of file.".to_owned(),
            ));
        }
        _ => {
            return Err(Error::new(token.clone(), "Expected expression.".to_owned()));
        }
    };
    stream.next();
    Ok(expr)
}

fn consume(
    stream: &mut impl TokenStream,
    kind: TokenKind,
    message: impl ToString,
) -> Result<Token> {
    let token = stream.peek();
    if token.kind != kind {
        return Err(Error::new(token.clone(), message.to_string()));
    }
    Ok(stream.next())
}

fn synchronize(stream: &mut impl TokenStream) {
    let mut current = stream.next();
    loop {
        if current.kind == TokenKind::Semicolon {
            break;
        }

        let next = stream.peek();

        if matches!(
            next.kind,
            TokenKind::Eof
                | TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return
        ) {
            break;
        }

        current = stream.next();
    }
}