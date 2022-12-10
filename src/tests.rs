use std::io::stderr;

use tracing::{info, Level};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{BasicAlgebraicExpr, SimpleExpr};

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::EXIT)
        .map_writer(|_| || stderr())
        .with_max_level(Level::TRACE)
        .try_init();
}

fn n(x: i128) -> BasicAlgebraicExpr {
    x.into()
}

fn sn(x: i128) -> SimpleExpr {
    x.into()
}

fn s(x: &str) -> BasicAlgebraicExpr {
    BasicAlgebraicExpr::Symbol(x.into())
}

fn ss(x: &str) -> SimpleExpr {
    SimpleExpr::Symbol(x.into())
}

fn opaque() -> BasicAlgebraicExpr {
    s("opaque")
}

fn sopaque() -> SimpleExpr {
    ss("opaque")
}

fn simplify(x: BasicAlgebraicExpr) -> SimpleExpr {
    x.simplify().unwrap()
}

#[test]
pub fn simplify_power() {
    // n^0 = 1
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

#[test]
pub fn simplify_addition() {
    init();
    assert_eq!(sn(3) * sopaque(), simplify(opaque() + n(2) * opaque()));
    
}
