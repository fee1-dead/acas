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

#[cfg(test)]
mod tests;
