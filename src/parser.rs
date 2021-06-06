/// A top-down parser implementation
use self::ParseError::*;
use crate::lexer::{Token, TokenStream};
use crate::util::{get_tok_prec, BINOP_PRECEDENCE};
use std::{env, fmt};

#[derive(Debug, PartialEq)]
/// expr -> number | var | binary | call | var_def
enum Expr {
    /// A numeric literal expression, e.g. (`3.14`)
    Number(f64),
    /// A variable reference expression, e.g. (`foo`)
    Var(String),
    /// A binary operator expression, e.g. (`3.14 + foo`)
    Binary {
        op: char,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// A function call, e.g. (`some_func(3.14, foo)`)
    /// call_expr -> ident `(` expr* `)`
    Call {
        callee: String,
        #[allow(clippy::vec_box)]
        args: Vec<Box<Expr>>,
    },
    /// var_def_expr -> `var` ident `=` expr
    VarDef { name: String, init: Box<Expr> },
}

/// A function prototype.
/// Captures the function's names its parameters
#[derive(Debug, PartialEq)]
struct FuncProto {
    name: String,
    params: Vec<String>,
}

/// A function definition
/// Captures the function's prototype and its body
/// func_def -> func_proto expr
#[derive(Debug, PartialEq)]
struct FuncDef {
    proto: Box<FuncProto>,
    body: Vec<Box<Expr>>,
}

type ParseResult<T> = Result<Box<T>, ParseError>;

#[derive(Debug)]
enum ParseError {
    /// message, line, col
    /// TODO: make a SyntaxErrorKind enum
    SyntaxError(String, usize, usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseError::*;
        match self {
            SyntaxError(msg, line, col) => {
                write!(f, "({},{}) {}", line, col, msg)
            }
        }
    }
}

fn parse_paren_expr(tokens: &mut TokenStream) -> ParseResult<Expr> {
    let expr = parse_expr(tokens)?;

    if tokens.eat() != Token::CloseParen {
        return Err(SyntaxError(
            String::from("expected a `)`"),
            tokens.line(),
            tokens.col(),
        ));
    }

    Ok(expr)
}

/// ident_expr -> ident | call_expr
fn parse_ident_expr(
    tokens: &mut TokenStream,
    ident_name: &str,
) -> ParseResult<Expr> {
    if tokens.peek() == Token::OpenParen {
        tokens.eat();
        // function call
        let mut args = Vec::new();

        loop {
            args.push(parse_expr(tokens)?);

            match tokens.eat() {
                // End of argument list
                Token::CloseParen => break,
                // More arguments ...
                Token::Comma => continue,
                _ => {
                    return Err(SyntaxError(
                        String::from(
                            "expected a `,` or `)` in an argument list",
                        ),
                        tokens.line(),
                        tokens.col(),
                    ))
                }
            }
        }

        Ok(Box::new(Expr::Call {
            callee: String::from(ident_name),
            args,
        }))
    } else {
        Ok(Box::new(Expr::Var(String::from(ident_name))))
    }
}

fn parse_var_def(tokens: &mut TokenStream) -> ParseResult<Expr> {
    let name = if let Token::Ident(name) = tokens.eat() {
        name
    } else {
        return Err(SyntaxError(
            String::from("expected a variable name in variable declaration"),
            tokens.line(),
            tokens.col(),
        ));
    };

    if tokens.eat() != Token::Op('=') {
        return Err(SyntaxError(
            String::from("expected `=` in variable declaration"),
            tokens.line(),
            tokens.col(),
        ));
    }

    let init = parse_expr(tokens)?;

    Ok(Box::new(Expr::VarDef { name, init }))
}

fn parse_primary(tokens: &mut TokenStream) -> ParseResult<Expr> {
    match tokens.eat() {
        Token::Ident(name) => parse_ident_expr(tokens, &name),
        Token::Number(val) => Ok(Box::new(Expr::Number(val))),
        Token::OpenParen => parse_paren_expr(tokens),
        Token::Var => parse_var_def(tokens),
        tok => Err(SyntaxError(
            format!("expected an expression, instead found `{}`", tok),
            tokens.line(),
            tokens.col(),
        )),
    }
}

fn parse_expr(tokens: &mut TokenStream) -> ParseResult<Expr> {
    let lhs = parse_primary(tokens)?;
    parse_bin_op_rhs(tokens, 0, lhs)
}

fn parse_bin_op_rhs(
    tokens: &mut TokenStream,
    expr_prec: i32,
    mut lhs: Box<Expr>,
) -> ParseResult<Expr> {
    loop {
        let bin_op = match tokens.peek() {
            Token::Op(o) => {
                tokens.eat();
                o
            }
            _ => return Ok(lhs),
        };

        let tok_prec = BINOP_PRECEDENCE.get(&bin_op).unwrap_or(&-1);

        if tok_prec < &expr_prec {
            return Ok(lhs);
        };

        let mut rhs = parse_primary(tokens)?;

        let next_prec = get_tok_prec(tokens.peek());

        if tok_prec < &next_prec {
            rhs = parse_bin_op_rhs(tokens, tok_prec + 1, rhs)?;
        }

        lhs = Box::new(Expr::Binary {
            op: bin_op,
            lhs,
            rhs,
        });
    }
}

fn parse_proto(tokens: &mut TokenStream) -> ParseResult<FuncProto> {
    let name = match tokens.eat() {
        Token::Ident(name) => name,
        _ => {
            return Err(SyntaxError(
                String::from("expected function name in function prototype"),
                tokens.line(),
                tokens.col(),
            ))
        }
    };

    if tokens.eat() == Token::OpenParen {
        let mut params = Vec::new();

        while let Token::Ident(name) = tokens.eat() {
            params.push(name);

            match tokens.eat() {
                // End of argument list
                Token::CloseParen => break,
                // More arguments ...
                Token::Comma => continue,
                _ => {
                    return Err(SyntaxError(
                        String::from("expected `,` or `)` in a parameter list"),
                        tokens.line(),
                        tokens.col(),
                    ))
                }
            }
        }

        Ok(Box::new(FuncProto { name, params }))
    } else {
        Err(SyntaxError(
            String::from("expected `(` in function prototype"),
            tokens.line(),
            tokens.col(),
        ))
    }
}

fn parse_func(tokens: &mut TokenStream) -> ParseResult<FuncDef> {
    let proto = parse_proto(tokens)?;

    if tokens.eat() != Token::OpenBrace {
        return Err(SyntaxError(
            String::from("expected `{` in function definition"),
            tokens.line(),
            tokens.col(),
        ));
    }

    let mut body = Vec::new();

    while tokens.peek() != Token::CloseBrace {
        // TODO: maybe add parse_stmt?
        let expr = parse_expr(tokens)?;

        if tokens.eat() != Token::Semicolon {
            return Err(SyntaxError(
                String::from("expected `;` to terminate statement"),
                tokens.line(),
                tokens.col(),
            ));
        }

        body.push(expr);
    }

    tokens.eat();

    Ok(Box::new(FuncDef { proto, body }))
}

fn parse_top_lvl_expr(tokens: &mut TokenStream) -> ParseResult<FuncDef> {
    let body = vec![parse_expr(tokens)?];
    let proto = Box::new(FuncProto {
        name: String::from("__anon_expr"),
        params: vec![],
    });
    Ok(Box::new(FuncDef { proto, body }))
}

fn parse_extern(tokens: &mut TokenStream) -> ParseResult<FuncProto> {
    if tokens.eat() != Token::Func {
        return Err(SyntaxError(
            String::from(
                "expected a function declaration in an extern statement",
            ),
            tokens.line(),
            tokens.col(),
        ));
    }

    parse_proto(tokens)
}

pub fn handle_func(tokens: &mut TokenStream) {
    match parse_func(tokens) {
        Ok(func) => eprintln!("parsed a function definition: {:#?}", func),
        Err(e) => {
            eprintln!("[kip::parser] error: {}", e);
            tokens.eat();
        }
    }
}

pub fn handle_extern(tokens: &mut TokenStream) {
    match parse_extern(tokens) {
        Ok(proto) => eprintln!("parsed an extern statement: {:#?}", proto),
        Err(e) => {
            eprintln!("[kip::parser] error: {}", e);
            tokens.eat();
        }
    }
}

pub fn handle_top_lvl_expr(tokens: &mut TokenStream) {
    match parse_top_lvl_expr(tokens).as_deref() {
        Ok(FuncDef { body: expr, .. }) => {
            eprintln!("parsed a top level expression: {:#?}", expr)
        }
        Err(e) => {
            eprintln!("[kip::parser] error: {}", e);
            tokens.eat();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_paren_expr() {
        use Expr::*;

        let input = "(a + b)";
        let mut tokens = TokenStream::new(input);
        let expr = parse_expr(&mut tokens).unwrap();

        assert_eq!(
            *expr,
            Binary {
                op: '+',
                lhs: Box::new(Var(String::from("a"))),
                rhs: Box::new(Var(String::from("b"))),
            }
        );
    }

    #[test]
    fn parse_func_def() {
        use Expr::*;

        let input = "\
func add(x, y) {\n
    var sum = x + y;\n
}";

        let mut tokens = TokenStream::new(input);

        assert_eq!(tokens.eat(), Token::Func);

        let func = parse_func(&mut tokens).unwrap();

        let proto = Box::new(FuncProto {
            name: String::from("add"),
            params: vec![String::from("x"), String::from("y")],
        });

        let binary_expr = Box::new(Binary {
            op: '+',
            lhs: Box::new(Var(String::from("x"))),
            rhs: Box::new(Var(String::from("y"))),
        });

        let body = vec![Box::new(VarDef {
            name: String::from("sum"),
            init: binary_expr,
        })];

        assert_eq!(*func, FuncDef { body, proto });
    }

    #[test]
    fn var_def() {
        use Expr::*;

        let input = "var my_var = 42 * 100";
        let mut tokens = TokenStream::new(input);

        assert_eq!(tokens.eat(), Token::Var);

        let var_def = parse_var_def(&mut tokens).unwrap();

        let name = String::from("my_var");

        let init = Box::new(Binary {
            op: '*',
            lhs: Box::new(Number(42.0)),
            rhs: Box::new(Number(100.0)),
        });

        assert_eq!(*var_def, VarDef { name, init });
    }
}
