#![feature(let_chains, if_let_guard)]

use constant::Constant;

use num::BigInt;
use simplify::SimpleExpr;

mod cmp;
pub mod constant;
mod helpers;
mod rational_expressions;
pub mod simplify;

#[derive(Debug)]
pub struct Undefined;

pub type ComputeResult<T = SimpleExpr> = Result<T, Undefined>;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BasicAlgebraicExpr {
    Const(Constant),
    Symbol(String),
    Product(Vec<BasicAlgebraicExpr>),
    Sum(Vec<BasicAlgebraicExpr>),
    Pow(Box<(BasicAlgebraicExpr, BasicAlgebraicExpr)>),
    Factorial(Box<BasicAlgebraicExpr>),
    Function(String, Vec<BasicAlgebraicExpr>),
}

impl From<i32> for SimpleExpr {
    fn from(x: i32) -> Self {
        SimpleExpr::Const(BigInt::from(x).into())
    }
}

pub mod ඞ {
    /// ```compile_fail
    /// use acas::create_expr;
    ///
    /// let x = create_expr!(bigint("hahaha"));
    /// ```
    pub const fn assert_digits(s: &str) {
        let mut i = 0;
        while i < s.len() {
            if !s.as_bytes()[i].is_ascii_digit() {
                panic!("please supply a number")
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests;
