use std::fmt::Debug;

use crate::constant::Constant;
use crate::rational_expressions::RationalExpr;
use crate::{BasicAlgebraicExpr, ComputeResult, SimpleExpr, Undefined};
use num::{BigInt, One, Signed, Zero};
use smallvec::{smallvec, SmallVec};
use tracing::{debug, info};

pub trait Operation: Copy + Debug {
    // https://en.wikipedia.org/wiki/Absorbing_element
    // we are talking about zero
    const HAS_ABSORBING_ELEMENT: bool;

    fn is_absorbing_element(self, x: &SimpleExpr) -> bool;

    // multiplicative identity / additive identity
    fn identity(self) -> SimpleExpr;

    fn is_identity(self, x: &Constant) -> bool;

    fn is_list(self, x: &SimpleExpr) -> bool;

    fn try_extract_list(self, x: SimpleExpr) -> Result<Vec<SimpleExpr>, SimpleExpr>;

    fn make_list(self, x: Vec<SimpleExpr>) -> SimpleExpr;

    fn extract_or_make_list(self, x: SimpleExpr) -> Vec<SimpleExpr> {
        self.try_extract_list(x).unwrap_or_else(|x| vec![x])
    }

    fn do_constant(self, x: Constant, y: Constant) -> Constant;

    /// The backbone of simplify_pair. If we can simplify by collecting like terms in addition or powers in multiplication, we do so.
    /// if not, we return `Ok(Err((a, b)))`.
    fn simplify_pair_collect(
        self,
        a: SimpleExpr,
        b: SimpleExpr,
    ) -> ComputeResult<Option<SmallVec<[SimpleExpr; 2]>>>;

    #[tracing::instrument]
    fn simplify_pair(
        self,
        a: SimpleExpr,
        b: SimpleExpr,
    ) -> ComputeResult<SmallVec<[SimpleExpr; 2]>> {
        if self.is_list(&a) || self.is_list(&b) {
            let a = self.extract_or_make_list(a);
            let b = self.extract_or_make_list(b);
            return self.merge(a, b).map(Into::into);
        }

        Ok(match (a, b) {
            (SimpleExpr::Const(a), SimpleExpr::Const(b)) => {
                let result = self.do_constant(a, b);
                if result.is_one() {
                    SmallVec::new()
                } else {
                    smallvec![result.into()]
                }
            }
            (SimpleExpr::Const(a), b) | (b, SimpleExpr::Const(a)) if self.is_identity(&a) => {
                smallvec![b]
            }
            (a, b) => {
                // NOTE: when in addition, we merge x + x = 2x, 3x + 4x = 7x, etc.
                // but when in multiplication, we merge x * x = x^2, x^3 * x^4 = x^7, etc.

                if let Some(res) = self.simplify_pair_collect(a.clone(), b.clone())? {
                    res
                } else if b < a {
                    smallvec![b, a]
                } else {
                    smallvec![a, b]
                }
            }
        })
    }

    // requirement: `exprs.len() >= 2`
    #[tracing::instrument(level = "debug", ret)]
    fn simplify_rec(self, list: Vec<SimpleExpr>) -> ComputeResult<Vec<SimpleExpr>> {
        let res: Result<[SimpleExpr; 2], _> = list.try_into();
        match res {
            Ok([a, b]) => self.simplify_pair(a, b).map(|x| x.into_vec()),
            Err(mut v) => {
                assert!(v.len() > 2);
                let first = v.remove(0);

                let first = self.extract_or_make_list(first);

                self.merge(first, v)
            }
        }
    }

    #[tracing::instrument(level = "debug")]
    fn simplify_entry(self, exprs: Vec<BasicAlgebraicExpr>) -> ComputeResult {
        let mut exprs: Vec<_> = exprs
            .into_iter()
            .map(BasicAlgebraicExpr::simplify)
            .collect::<Result<_, _>>()?;
        exprs.sort_unstable();
        self.simplify(exprs)
    }

    #[tracing::instrument(level = "debug", ret)]
    fn simplify(self, mut exprs: Vec<SimpleExpr>) -> ComputeResult {
        if Self::HAS_ABSORBING_ELEMENT {
            for exp in &exprs {
                if self.is_absorbing_element(exp) {
                    return Ok(0.into());
                }
            }
        }

        if exprs.len() == 1 {
            return Ok(exprs.pop().expect("len >= 1"));
        }

        let mut list = self.simplify_rec(exprs)?;
        // TODO replace with deref patterns
        Ok(match list.len() {
            0 => self.identity(),
            1 => list.pop().expect("len == 1"),
            _ => self.make_list(list),
        })
    }

    // entry point. Do not call in recursion. Call `merge_into` instead.
    fn merge(self, a: Vec<SimpleExpr>, b: Vec<SimpleExpr>) -> ComputeResult<Vec<SimpleExpr>> {
        let mut out = Vec::with_capacity(a.len() + b.len());
        self.merge_into(a, b, &mut out)?;
        Ok(out)
    }

    #[tracing::instrument(level = "debug", ret)]
    fn merge_into(
        self,
        mut a: Vec<SimpleExpr>,
        mut b: Vec<SimpleExpr>,
        out: &mut Vec<SimpleExpr>,
    ) -> ComputeResult<()> {
        if b.is_empty() {
            out.extend(a);
            return Ok(());
        }

        if a.is_empty() {
            out.extend(b);
            return Ok(());
        }

        let mut a_rest = a.split_off(1);
        let mut b_rest = b.split_off(1);
        let a = a.pop().unwrap();
        let b = b.pop().unwrap();

        let would_swap = a > b;

        let simplified = self.simplify_pair(a, b)?;

        match simplified.len() {
            0 => self.merge_into(a_rest, b_rest, out)?,
            1 => {
                out.extend(simplified);
                self.merge_into(a_rest, b_rest, out)?;
            }
            2 => {
                let [first, second]: [_; 2] = simplified.into_inner().unwrap();

                if would_swap {
                    a_rest.insert(0, second);
                } else {
                    b_rest.insert(0, second);
                };

                out.push(first);
                self.merge_into(a_rest, b_rest, out)?;
            }
            _ => unreachable!("nested operations should have been flattened already"),
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Product;

impl Operation for Product {
    const HAS_ABSORBING_ELEMENT: bool = true;

    fn is_absorbing_element(self, expr: &SimpleExpr) -> bool {
        matches!(expr, SimpleExpr::Const(n) if n.is_zero())
    }

    fn is_identity(self, expr: &Constant) -> bool {
        expr.is_one()
    }

    fn identity(self) -> SimpleExpr {
        1.into()
    }

    fn is_list(self, x: &SimpleExpr) -> bool {
        matches!(x, SimpleExpr::Product(_))
    }

    fn try_extract_list(self, x: SimpleExpr) -> Result<Vec<SimpleExpr>, SimpleExpr> {
        match x {
            SimpleExpr::Product(x) => Ok(x),
            x => Err(x),
        }
    }

    fn make_list(self, x: Vec<SimpleExpr>) -> SimpleExpr {
        SimpleExpr::Product(x)
    }

    fn do_constant(self, x: Constant, y: Constant) -> Constant {
        x * y
    }

    fn simplify_pair_collect(
        self,
        a: SimpleExpr,
        b: SimpleExpr,
    ) -> ComputeResult<Option<SmallVec<[SimpleExpr; 2]>>> {
        Ok(
            if let Some(base) = a.base().filter(|x| Some(*x) == b.base()) {
                let exponent = Sum.simplify(vec![
                    a.exponent().expect("base() is not None"),
                    b.exponent().expect("base() is not None"),
                ])?;
                let result = BasicAlgebraicExpr::simplify_power(base.clone(), exponent)?;
                Some(if let SimpleExpr::Const(c) = &result && c.is_one() {
                smallvec![]
            } else {
                smallvec![result]
            })
            } else {
                None
            },
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sum;

impl Operation for Sum {
    const HAS_ABSORBING_ELEMENT: bool = false;
    fn is_absorbing_element(self, _: &SimpleExpr) -> bool {
        false
    }
    fn identity(self) -> SimpleExpr {
        0.into()
    }
    fn is_identity(self, expr: &Constant) -> bool {
        expr.is_zero()
    }
    fn is_list(self, x: &SimpleExpr) -> bool {
        matches!(x, SimpleExpr::Sum(_))
    }
    fn try_extract_list(self, x: SimpleExpr) -> Result<Vec<SimpleExpr>, SimpleExpr> {
        match x {
            SimpleExpr::Sum(x) => Ok(x),
            x => Err(x),
        }
    }
    fn make_list(self, x: Vec<SimpleExpr>) -> SimpleExpr {
        SimpleExpr::Sum(x)
    }

    #[tracing::instrument(level = "debug")]
    fn do_constant(self, x: Constant, y: Constant) -> Constant {
        x + y
    }

    #[tracing::instrument(level = "debug")]
    // TODO should return smallvec?
    fn simplify_pair_collect(
        self,
        a: SimpleExpr,
        b: SimpleExpr,
    ) -> ComputeResult<Option<SmallVec<[SimpleExpr; 2]>>> {
        let (rationala, a_sym) = a.split_product().expect("must not be constant");
        let (rationalb, b_sym) = b.split_product().expect("must not be constant");

        debug!(?rationala, ?rationalb, ?a_sym, ?b_sym);

        Ok(if a_sym == b_sym {
            let sum = (rationala + rationalb).simplify().into_algebraic_expr()?;
            debug!(?sum, ?a_sym);
            Some(smallvec![SimpleExpr::Product(Product.simplify_pair(sum, a_sym)?.into_vec())])
        } else {
            None
        })
    }
}

impl BasicAlgebraicExpr {
    #[inline]
    pub const fn is_constant(&self) -> bool {
        matches!(self, BasicAlgebraicExpr::Const(_))
    }

    fn simplify_integer_power(base: SimpleExpr, exp: &BigInt) -> ComputeResult {
        match base {
            _ if exp.is_zero() => Ok(1.into()),
            _ if exp.is_one() => Ok(base),
            SimpleExpr::Const(base) => RationalExpr::Pow(Box::new(base.into()), exp.clone())
                .simplify()
                .into(),
            SimpleExpr::Pow(x) => {
                let (base, exp2) = *x;
                let exp = Product.simplify(vec![SimpleExpr::Const(exp.clone().into()), exp2])?;
                if let SimpleExpr::Const(n) = &exp && let Some(n) = n.as_integer() {
                    Self::simplify_integer_power(base, n)
                } else {
                    Ok(SimpleExpr::Pow(Box::new((base, exp))))
                }
            }
            _ => todo!(),
        }
    }
    fn simplify_power(base: SimpleExpr, exponent: SimpleExpr) -> ComputeResult {
        if base == 0 {
            match exponent {
                SimpleExpr::Const(i) if i.is_positive() => Ok(0.into()),
                // 0^0 or 0^(-n) is undefined
                SimpleExpr::Const(_) => Err(Undefined),
                _ => Ok(SimpleExpr::Pow(Box::new((base, exponent)))),
            }
        } else if base == 1 {
            // 1^x = 1
            Ok(SimpleExpr::Const(One::one()))
        } else if let SimpleExpr::Const(exp) = &exponent && let Some(exp) = exp.as_integer() {
            Self::simplify_integer_power(base, exp)
        } else {
            Ok(SimpleExpr::Pow(Box::new((base, exponent))))
        }
    }
    pub fn simplify(self) -> ComputeResult {
        use BasicAlgebraicExpr::*;
        use SimpleExpr as E;
        Ok(match self {
            Const(c) if c.denom().is_zero() => return Err(Undefined),
            Const(c) => E::Const(c),
            Symbol(s) => E::Symbol(s),
            Pow(x) => Self::simplify_power((*x).0.simplify()?, (*x).1.simplify()?)?,
            Sum(x) => self::Sum.simplify_entry(x)?,
            Product(x) => self::Product.simplify_entry(x)?,
            _ => todo!(),
        })
    }
}
