use assert_matches::assert_matches;
use unlox_ast::TokenKind;
use unlox_interpreter::{self as interpreter, Interpreter};
use unlox_lexer::Lexer;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
enum Error {
    Parse(unlox_parse::Error),
    Interpret(unlox_interpreter::Error),
}

fn interpret(code: &str) -> Result<String, Error> {
    let mut out = Vec::new();
    let lexer = Lexer::new(code);
    let ast = unlox_parse::parse(lexer).map_err(Error::Parse)?;
    let mut interpreter = Interpreter::new(&mut out);
    interpreter
        .interpret(code, &ast)
        .map_err(Error::Interpret)?;
    Ok(String::from_utf8(out).unwrap())
}

#[test]
fn empty() {
    assert_eq!(interpret("").unwrap(), "");
}

#[test]
fn math_expressions() {
    assert_eq!(interpret("print 2 + 2 * 2;").unwrap(), "6\n");
    assert_eq!(interpret("print (2 + 2) * 2;").unwrap(), "8\n");
}

#[test]
fn boolean_logic() {
    let code = r#"
        print "hi" or 2;
        print nil or "yes";
    "#;
    assert_eq!(interpret(code).unwrap(), "hi\nyes\n");
}

#[test]
fn if_statements() {
    let code = r#"
        if (true) {
            print true;
        } else {
            print false;
        }

        if (true) print true; else print false;
        
        if (false) {
            print true;
        } else {
            print false;
        }

        if (false) print true; else print false;
    "#;
    assert_eq!(interpret(code).unwrap(), "true\ntrue\nfalse\nfalse\n");
}

#[test]
fn while_statements() {
    let code = r#"
        var n = 3;
        while (n > 0) {
            print n;
            n = n - 1;
        }
    "#;
    assert_eq!(interpret(code).unwrap(), "3\n2\n1\n");
}

#[test]
fn for_statements() {
    let code = r#"
        var a = 0;
        var temp;

        for (var b = 1; a < 100; b = temp + b) {
            print a;
            temp = a;
            a = b;
        }
    "#;
    assert_eq!(
        interpret(code).unwrap(),
        "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n55\n89\n"
    );
}

#[test]
fn functions() {
    let code = r#"
        fun sayHi(first, last) {
            print "Hi, " + first + " " + last + "!";
        }

        sayHi("Dear", "Reader");
    "#;
    assert_eq!(interpret(code).unwrap(), "Hi, Dear Reader!\n");

    let code = r#"
        fun fibonacci(n) {
            var a = 0;
            var b = 1;

            for (var i = 0; i < n; i = i + 1) {
                var temp = a;
                a = b;
                b = temp + b;
            }
            return a;
        }

        print fibonacci(12);
    "#;
    assert_eq!(interpret(code).unwrap(), "144\n");

    let code = r#"
        fun fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 2) + fibonacci(n - 1);
        }

        print fibonacci(12);
    "#;
    assert_eq!(interpret(code).unwrap(), "144\n");

    let code = r#"
        var a = 1;

        fun main() {
            var b = 2;

            fun nested() {
                print a;
                print b;
            }

            nested();
        }
        main();
    "#;
    let err = interpret(code).unwrap_err();
    assert_matches!(err, Error::Interpret(interpreter::Error::UndefinedVariable { name, token }) => {
        assert_eq!(name, "b");
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.line, 9);
    });
}
