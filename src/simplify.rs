use crate::rational_expressions::RationalExpr;
use crate::Constant;

mod ops;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum SimpleExpr {
    Const(Constant),
    Symbol(String),
    Product(Vec<SimpleExpr>),
    Sum(Vec<SimpleExpr>),
    Pow(Box<(SimpleExpr, SimpleExpr)>),
    Factorial(Box<SimpleExpr>),
    Function(String, Vec<SimpleExpr>),
}

impl SimpleExpr {
    pub fn is_constant(&self) -> bool {
        matches!(self, SimpleExpr::Const(_))
    }

    // If this is a product, split this into (constant, symbolic) parts
    // otherwise, retutn (1, x)
    // if this is a constant, return none
    pub fn split_product(self) -> Result<(RationalExpr, SimpleExpr), SimpleExpr> {
        match self {
            SimpleExpr::Product(mut x) => {
                if let Some(sym_index) = x.iter().position(|x| !x.is_constant()) {
                    let mut symbols = x.split_off(sym_index);

                    Ok((
                        RationalExpr::Mul(
                            x.into_iter()
                                .map(|x| match x {
                                    SimpleExpr::Const(c) => c.into(),
                                    _ => unreachable!(),
                                })
                                .collect(),
                        ),
                        match symbols.len() {
                            1 => symbols.pop().unwrap(),
                            _ => SimpleExpr::Product(symbols),
                        },
                    ))
                } else {
                    unreachable!("product with only constant parts should be simplified already");
                }
            }
            SimpleExpr::Const(_) => Err(self),
            _ => Ok((RationalExpr::Const(1.into()), self.clone())),
        }
    }
    pub fn base(&self) -> Option<&SimpleExpr> {
        Some(match self {
            SimpleExpr::Pow(x) => &x.0,
            SimpleExpr::Const(_) => return None,
            _ => self,
        })
    }

    pub fn exponent(&self) -> Option<SimpleExpr> {
        Some(match self {
            SimpleExpr::Pow(x) => x.1.clone(),
            SimpleExpr::Const(_) => return None,
            _ => SimpleExpr::Const(1.into()),
        })
    }
}

impl From<Constant> for SimpleExpr {
    fn from(c: Constant) -> Self {
        Self::Const(c)
    }
}
