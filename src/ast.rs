//! AST node types for the Lingo programming language (Lisp dialect).
//!
//! The entire AST is a single `Expr` enum -- homoiconic S-expressions.

use std::fmt;

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Symbol(String),
    List(Vec<Expr>),
    Nil,
}

impl Expr {
    /// Extract the symbol name, or return an error.
    pub fn as_symbol(&self) -> Result<&str, String> {
        match self {
            Expr::Symbol(s) => Ok(s),
            other => Err(format!("expected symbol, got {}", other)),
        }
    }

    /// Extract the list elements, or return an error.
    pub fn as_list(&self) -> Result<&[Expr], String> {
        match self {
            Expr::List(elems) => Ok(elems),
            other => Err(format!("expected list, got {}", other)),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Int(n) => write!(f, "{}", n),
            Expr::Float(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Expr::Str(s) => write!(f, "\"{}\"", s),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Symbol(s) => write!(f, "{}", s),
            Expr::List(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Expr::Nil => write!(f, "nil"),
        }
    }
}
