use crate::simplify::SimpleExpr;

pub fn to_latex(x: &SimpleExpr) -> String {
    let mut f = String::new();
    latex_print(x, &mut f);
    f
}

pub fn latex_print(x: &SimpleExpr, f: &mut String) {
    match x {
        SimpleExpr::Const(x) if let Some(i) = x.as_integer() =>  {
            f.push_str(&i.to_string());
        }
        SimpleExpr::Const(x) => {
            let rational = &**x;
            let num = rational.numer();
            let denom = rational.denom();
            f.push_str(&format!("\\frac {{ {num} }} {{ {denom} }}"));
        }
        SimpleExpr::Symbol(x) => {
            f.push_str(&x);
        }
        SimpleExpr::Product(x) => {
            if x.len() == 0 {
                unreachable!()
            } else {
                for (i, x) in x.iter().enumerate() {
                    if i != 0 {
                        f.push_str(" \\cdot ");
                    }
                    latex_print(x, f);
                }
            }
        }
        SimpleExpr::Sum(x) => {
            for (i, x) in x.iter().enumerate() {
                if i != 0 {
                    f.push_str(" + ");
                }
                latex_print(x, f);
            }
        }
        SimpleExpr::Pow(x) => {
            f.push_str("(");
            latex_print(&x.0, f);
            f.push_str(")^{");
            latex_print(&x.1, f);
            f.push_str("}");
        }
        SimpleExpr::Factorial(x) => {
            latex_print(x, f);
            f.push_str("!");
        }
        SimpleExpr::Function(x, y) => {
            f.push_str(&x);
            f.push_str("(");
            for (i, x) in y.iter().enumerate() {
                if i != 0 {
                    f.push_str(", ");
                }
                latex_print(x, f);
            }
            f.push_str(")");
        }
    }
}