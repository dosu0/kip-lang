use super::Parser;
use crate::ast::{Expr, Stmt, StmtKind};
use crate::lexer::Lexer;
use crate::name::Name;
use crate::source::Source;

pub fn parse(source_code: &'static str) -> Vec<Box<Stmt>> {
    let source = Source::new(source_code, "<string literal>");
    let mut lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer.lex(), &source);

    let stmts: Result<Vec<_>, _> = parser.parse().into_iter().collect();
    stmts.expect("failed to parse")
}

fn extract_stmt(module: &[Box<Stmt>]) -> &Stmt {
    module.first().expect("expected stmt")
}

fn extract_expr(module: &[Box<Stmt>]) -> &Expr {
    let stmt = extract_stmt(module);

    match &stmt.kind {
        StmtKind::Expr(expr) => expr,
        _ => panic!("expected expr"),
    }
}

mod stmt {
    use crate::ast::ty::IntSize;
    use crate::ast::Type;

    use super::*;

    #[test]
    fn var_decl() {
        let module = parse("var x = 32;");
        let stmt = extract_stmt(&module);

        match stmt.kind {
            StmtKind::Var(name, ref value) => {
                assert_eq!(name, "x");
                assert_eq!(value.kind, 32.into());
            }
            _ => panic!("expected a var decl"),
        }
    }

    #[test]
    fn basic_func_decl() {
        let module = parse("func main() { }");
        let stmt = extract_stmt(&module);

        let (proto, block) = match &stmt.kind {
            StmtKind::Func(proto, block) => (proto, block),
            _ => panic!("expected func"),
        };

        assert!(proto.params.is_empty());
        assert_eq!(proto.name, "main");
        assert_eq!(proto.ret, Type::Void);
        assert!(block.is_empty());
    }

    #[test]
    fn complex_func_decl() {
        let module = parse(
            r#"func foo(a: int32, b: uint8): String {
                   var sum = a + b;
                   ret "foo";
            }"#,
        );
        let stmt = extract_stmt(&module);

        let (proto, block) = match &stmt.kind {
            StmtKind::Func(proto, block) => (proto, block),
            _ => panic!("expected func"),
        };

        assert_eq!(proto.params.len(), 2);
        let mut params = proto.params.iter();
        let param_1 = params.next().unwrap();
        assert_eq!(param_1.name, "a");
        assert_eq!(
            param_1.ty,
            Type::Int {
                signed: true,
                size: IntSize::new(32)
            }
        );

        let param_2 = params.next().unwrap();
        assert_eq!(param_2.name, "b");
        assert_eq!(
            param_2.ty,
            Type::Int {
                signed: false,
                size: IntSize::new(8)
            }
        );

        assert_eq!(proto.name, "foo");
        assert_eq!(proto.ret, Type::Name(Name::from("String")));
        assert_eq!(block.len(), 2);
    }
}

mod expr {
    use super::*;

    use crate::ast::{BinOp, ExprKind, StmtKind};

    #[test]
    fn assign_expression() {
        use ExprKind::{Assign, Variable};

        let module = parse("a = b = c;");
        let expr = extract_expr(&module);

        if let Assign(var_name, ref expr) = expr.kind {
            assert_eq!(var_name, "a");

            if let Assign(var_name, ref expr) = expr.kind {
                assert_eq!(var_name, "b");
                assert_eq!(expr.kind, Variable(Name::from("c")));
            } else {
                panic!("expected assignment expression");
            }
        } else {
            panic!("expected assignment expression");
        }
    }

    #[test]
    fn call() {
        use ExprKind::{Call, Variable};

        let module = parse("foo(bar, baz);");
        let expr = extract_expr(&module);

        // TODO: this should be possible :)
        // assert_eq!(expr.to_string(), "Call([Variable(\"bar\"), Variable(\"baz\")])");

        if let Call(fn_name, ref args) = expr.kind {
            assert_eq!(fn_name, "foo");

            let mut args = args.iter();

            match args.next().expect("expected arg 1").kind {
                Variable(name) => assert_eq!(name, "bar"),
                _ => panic!("expected var"),
            }

            match args.next().expect("expected arg 2").kind {
                Variable(name) => assert_eq!(name, "baz"),
                _ => panic!("expected var"),
            }
        }
    }

    #[test]
    fn paren_expressions() {
        use ExprKind::{Binary, Variable};

        let module = parse("(a + b);");
        let expr = extract_expr(&module);

        if let Binary(op, lhs, rhs) = &expr.kind {
            assert_eq!(op, &BinOp::Add);

            match lhs.kind {
                Variable(name) => assert_eq!(name, "a"),
                _ => panic!("expected variable"),
            }
            match rhs.kind {
                Variable(name) => assert_eq!(name, "b"),
                _ => panic!("expected variable"),
            }
        } else {
            panic!("expected binary expression");
        }
    }

    #[test]
    fn conditionals() {
        use crate::ast::Lit::*;
        use ExprKind::*;

        let input = "\
        if x >= 0 {\n\
            positive();\n\
        } else {\n\
            negative();\n\
        }";
        let module = parse(input);
        let expr = extract_expr(&module);

        let (condition, then_branch, else_branch) = match &expr.kind {
            Cond(condition, then_branch, else_branch) => (condition, then_branch, else_branch),
            _ => panic!("expected a conditional expression"),
        };

        if let Binary(op, ref lhs, ref rhs) = condition.kind {
            assert_eq!(op, BinOp::Ge);

            match lhs.kind {
                Variable(name) => assert_eq!(name, "x"),
                _ => panic!("expected variable reference"),
            }

            match rhs.kind {
                Lit(Int(num)) => assert_eq!(num, 0),
                _ => panic!("expected an integer literal"),
            }
        } else {
            panic!("expected binary expression");
        }

        if let StmtKind::Expr(expr) = &then_branch[0].kind {
            match expr.kind {
                Call(name, ref args) => {
                    assert_eq!(name, "positive");
                    assert!(args.is_empty());
                }
                _ => panic!("expected call expr"),
            }
        } else {
            panic!("expected expr in then branch");
        }

        let else_branch = else_branch.as_ref().unwrap();
        if let StmtKind::Expr(expr) = &else_branch[0].kind {
            if let Call(name, ref args) = expr.kind {
                assert_eq!(name, "negative");
                assert!(args.is_empty());
            } else {
                panic!("expected call expr");
            }
        } else {
            panic!("expected expr in else_block");
        }
    }
}
