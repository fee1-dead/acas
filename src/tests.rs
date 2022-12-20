use std::io::stderr;
use std::ops::{Add, Mul};

use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{BasicAlgebraicExpr, SimpleExpr};

mod parse;

#[derive(Debug, Clone)]
pub enum TestExpr {
    LazySym(&'static str, bool),
    Simplified(SimpleExpr),
    Basic(BasicAlgebraicExpr),
}

impl TestExpr {
    pub fn simplify(self) -> SimpleExpr {
        match self {
            TestExpr::LazySym(s, _) => SimpleExpr::Symbol(s.to_string()),
            TestExpr::Simplified(a) => a,
            TestExpr::Basic(b) => b.simplify().unwrap(),
        }
    }
}

impl PartialEq for TestExpr {
    fn eq(&self, other: &Self) -> bool {
        self.clone().simplify() == other.clone().simplify()
    }
}

macro_rules! impl_op {
    ($Tr:ident($fn_name:ident), $variant:ident) => {
        impl $Tr<i128> for TestExpr {
            type Output = TestExpr;
            fn $fn_name(self, rhs: i128) -> Self::Output {
                match self {
                    TestExpr::LazySym(st, is_simple) => {
                        if is_simple {
                            TestExpr::Simplified(SimpleExpr::Sum(vec![ss(st), sn(rhs)]))
                        } else {
                            TestExpr::Basic(BasicAlgebraicExpr::$variant(vec![s(st), n(rhs)]))
                        }
                    }
                    TestExpr::Simplified(s) => {
                        TestExpr::Simplified(SimpleExpr::$variant(vec![s, sn(rhs)]))
                    }
                    TestExpr::Basic(b) => {
                        TestExpr::Basic(BasicAlgebraicExpr::$variant(vec![b, n(rhs)]))
                    }
                }
            }
        }

        impl $Tr<TestExpr> for i128 {
            type Output = TestExpr;
            fn $fn_name(self, rhs: TestExpr) -> Self::Output {
                match rhs {
                    TestExpr::LazySym(st, is_simple) => {
                        if is_simple {
                            TestExpr::Simplified(SimpleExpr::$variant(vec![sn(self), ss(st)]))
                        } else {
                            TestExpr::Basic(BasicAlgebraicExpr::$variant(vec![n(self), s(st)]))
                        }
                    }
                    TestExpr::Simplified(s) => {
                        TestExpr::Simplified(SimpleExpr::$variant(vec![sn(self), s]))
                    }
                    TestExpr::Basic(b) => {
                        TestExpr::Basic(BasicAlgebraicExpr::$variant(vec![n(self), b]))
                    }
                }
            }
        }
    };
}

impl_op!(Add(add), Sum);
impl_op!(Mul(mul), Product);

impl Add for TestExpr {
    type Output = TestExpr;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (TestExpr::LazySym(st, is_simple), TestExpr::LazySym(st2, is_simple2)) => {
                if is_simple != is_simple2 {
                    panic!("Cannot add two lazy symbols with different simplification status");
                }
                if is_simple || is_simple2 {
                    TestExpr::Simplified(SimpleExpr::Sum(vec![ss(st), ss(st2)]))
                } else {
                    TestExpr::Basic(s(st) + s(st2))
                }
            }
            (TestExpr::LazySym(st, is_simple), TestExpr::Simplified(s))
            | (TestExpr::Simplified(s), TestExpr::LazySym(st, is_simple)) => {
                if is_simple {
                    TestExpr::Simplified(SimpleExpr::Sum(vec![ss(st), s]))
                } else {
                    panic!("Cannot add a lazy symbol with simplification status false to a simplified expression");
                }
            }
            (TestExpr::LazySym(st, is_simple), TestExpr::Basic(b))
            | (TestExpr::Basic(b), TestExpr::LazySym(st, is_simple)) => {
                if is_simple {
                    panic!("Cannot add a lazy symbol with simplification status true to a basic algebraic expression");
                } else {
                    TestExpr::Basic(s(st) + b)
                }
            }
            (TestExpr::Simplified(s), TestExpr::Simplified(s2)) => {
                TestExpr::Simplified(SimpleExpr::Sum(vec![s, s2]))
            }
            (TestExpr::Basic(b), TestExpr::Basic(b2)) => TestExpr::Basic(b + b2),
            _ => panic!("Cannot add different types of expressions"),
        }
    }
}

#[allow(non_upper_case_globals)]
const x: TestExpr = TestExpr::LazySym("x", false);

#[allow(non_upper_case_globals)]
const sx: TestExpr = TestExpr::LazySym("x", true);

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::EXIT)
        .map_writer(|_| || stderr())
        .with_max_level(Level::TRACE)
        .try_init();
}

fn n(a: i128) -> BasicAlgebraicExpr {
    a.into()
}

fn sn(a: i128) -> SimpleExpr {
    a.into()
}

fn s(a: &str) -> BasicAlgebraicExpr {
    BasicAlgebraicExpr::Symbol(a.into())
}

fn ss(a: &str) -> SimpleExpr {
    SimpleExpr::Symbol(a.into())
}

fn opaque() -> BasicAlgebraicExpr {
    s("opaque")
}

fn sopaque() -> SimpleExpr {
    ss("opaque")
}

fn simplify(a: BasicAlgebraicExpr) -> SimpleExpr {
    a.simplify().unwrap()
}

#[test]
pub fn simplify_power() {
    // n^0 = 1
    assert_eq!(1, simplify(n(1) ^ n(0)));
    assert_eq!(1, simplify(opaque() ^ n(0)));
    // 1^n = 1
    assert_eq!(1, simplify(n(1) ^ opaque()));
    // 0^n = 0 if n > 0
    assert_eq!(0, simplify(n(0) ^ n(1)));
    assert_eq!(0, simplify(n(0) ^ n(2)));
    // 0^n = 0^n
    assert_eq!(
        SimpleExpr::Pow(Box::new((sn(0), sopaque()))),
        simplify(n(0) ^ opaque())
    );
}

macro_rules! assert_simplified_eq {
    ($left:expr, $right:expr) => {
        assert_eq!(($left).simplify(), ($right).simplify())
    };
}

#[test]
pub fn simplify_addition() {
    init();
    assert_simplified_eq!(3 * sx, x + 2 * x);
    assert_simplified_eq!(6 * sx, x + 2 * x + 3 * x);
}
