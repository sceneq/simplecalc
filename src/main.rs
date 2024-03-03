#[rust_sitter::grammar("arithmetic")]
mod tuma1 {
    #[rust_sitter::language]
    #[derive(Debug)]
    pub enum Expr {
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] u32),
        #[rust_sitter::prec_left(1)]
        Add(Box<Expr>, #[rust_sitter::leaf(text = "+")] (), Box<Expr>),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[test]
fn example1() {
    use tuma1::Expr;

    fn eval(tree: &Expr) -> u32 {
        match tree {
            Expr::Number(v) => *v,
            Expr::Add(l, _, r) => eval(l) + eval(r),
        }
    }

    let code = "1+2+   3";
    let want = 6;
    let got = eval(&tuma1::parse(code).unwrap());
    assert_eq!(got, want);
}

#[rust_sitter::grammar("arithmetic")]
pub mod tuma2 {
    #[rust_sitter::language]
    #[derive(PartialEq, Eq, Debug)]
    pub enum Expr {
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
        #[rust_sitter::prec_left(1)]
        Sub(Box<Expr>, #[rust_sitter::leaf(text = "-")] (), Box<Expr>),
        #[rust_sitter::prec_left(2)]
        Mul(Box<Expr>, #[rust_sitter::leaf(text = "*")] (), Box<Expr>),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[test]
fn example2() {
    use tuma2::Expr;

    fn eval(tree: &Expr) -> i32 {
        match tree {
            Expr::Number(v) => *v,
            Expr::Sub(l, _, r) => eval(l) - eval(r),
            Expr::Mul(l, _, r) => eval(l) * eval(r),
        }
    }

    let code = "9*3-  8 - 5";
    let want = 6;
    let got = eval(&tuma2::parse(code).unwrap());
    assert_eq!(got, want);
}

#[rust_sitter::grammar("arithmetic")]
pub mod tuma3 {
    #[rust_sitter::language]
    #[derive(PartialEq, Eq, Debug)]
    pub enum Expr {
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
        #[rust_sitter::prec_left(1)]
        Add(Box<Expr>, #[rust_sitter::leaf(text = "+")] (), Box<Expr>),
        #[rust_sitter::prec_left(2)]
        Mul(Box<Expr>, #[rust_sitter::leaf(text = "*")] (), Box<Expr>),
        #[rust_sitter::prec_left(3)]
        Parenthesized(
            #[rust_sitter::leaf(text = "(")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = ")")] (),
        ),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[test]
fn example3() {
    use tuma3::Expr;

    fn eval(tree: &Expr) -> i32 {
        match tree {
            Expr::Parenthesized(_, expr, _) => eval(expr),
            Expr::Number(v) => *v,
            Expr::Add(l, _, r) => eval(l) + eval(r),
            Expr::Mul(l, _, r) => eval(l) * eval(r),
        }
    }

    let code = "9* (3 +4) +8";
    let want = 71;
    let got = eval(&tuma3::parse(code).unwrap());
    assert_eq!(got, want);
}

#[rust_sitter::grammar("arithmetic")]
pub mod tuma4 {

    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct Program {
        pub statements: Vec<Statement>,
    }

    #[derive(Debug)]
    pub enum Statement {
        Expr(Expr),
        Assign(Assign),
    }

    #[derive(Debug)]
    pub enum Expr {
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
        #[rust_sitter::prec_left(1)]
        Add(Box<Expr>, #[rust_sitter::leaf(text = "+")] (), Box<Expr>),
        #[rust_sitter::prec_left(2)]
        Mul(Box<Expr>, #[rust_sitter::leaf(text = "*")] (), Box<Expr>),
        #[rust_sitter::prec_left(3)]
        Parenthesized(
            #[rust_sitter::leaf(text = "(")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = ")")] (),
        ),
        Var(Ident),
    }

    #[derive(Debug)]
    pub struct Assign {
        #[rust_sitter::leaf(text = "let")]
        _let: (),
        pub lhs: Ident,
        #[rust_sitter::leaf(text = "=")]
        _equal: (),
        pub rhs: Box<Expr>,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[derive(Debug)]
    pub struct Ident {
        #[rust_sitter::leaf(pattern = r"[_\p{XID_Start}][_\p{XID_Continue}]*", transform = |s| s.to_string())]
        pub name: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s|\r|\n")]
        _whitespace: (),
    }
}

fn main() {
    use std::collections::HashMap;
    use tuma4::{Expr, Program, Statement};

    fn eval(tree: &Expr, vars: &mut HashMap<String, i32>) -> i32 {
        match tree {
            Expr::Parenthesized(_, expr, _) => eval(expr, vars),
            Expr::Number(v) => *v,
            Expr::Add(l, _, r) => eval(l, vars) + eval(r, vars),
            Expr::Mul(l, _, r) => eval(l, vars) * eval(r, vars),
            Expr::Var(var) => *vars.get(&var.name).unwrap(),
        }
    }

    fn eval_program(tree: &Program) -> Option<i32> {
        let mut vars = HashMap::new();
        let mut last_value = None;
        for s in &tree.statements {
            let val = match s {
                Statement::Expr(expr) => eval(expr, &mut vars),
                Statement::Assign(a) => {
                    let v = eval(&a.rhs, &mut vars);
                    vars.insert(a.lhs.name.to_owned(), v);
                    v
                }
            };
            last_value = Some(val)
        }
        last_value
    }

    let code = "
        let a = 2*(8+2);
        let b = 3 + a + 5;
        b
    ";
    let want = 28;
    let tree = tuma4::parse(code).unwrap();
    let got = eval_program(&tree).unwrap();
    assert_eq!(got, want);
}
