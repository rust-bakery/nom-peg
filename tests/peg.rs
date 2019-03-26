#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate nom;

use nom::peg::peg_grammar;

// Goal syntax
// parser = peg!{
//     Expr = <l: Product> ("+" <r: Product>)* => { r.fold(l, |a, i| a + i) }
//          | <l: Product> ("-" <r: Product>)* => { r.fold(l, |a, i| a - i) }
//
//     Product = <l: Value> ("*" <r: Value>)* => { r.fold(l, |a, i| a * i) }
//             | <l: Value> ("/" <r: Value>)* => { r.fold(l, |a, i| a / i) }
//
//     Value = <s: [0-9]+> => { s.parse::<u64>() }
//           | "(" <Expr> ")"
// }
// // and using the (sub) parsers
// result: u64 = parser.Expr("2+2*(3-5)") // should return -2


#[test]
fn peg_test() {
    let parser = peg_grammar! {
        p = &"a"* "a"* => { "yay" }
        a = "a"* => { "" }
        q = a ("b" | "c") => { "boo" }
    };

    assert_eq!(parser.p("abc"), Ok(("bc", "yay")));
    assert_eq!(parser.p("aaaaaaab"), Ok(("b", "yay")));

    assert_eq!(parser.q("abcc"), Ok(("cc", "boo")));
    assert_eq!(parser.q("aaaaaaab"), Ok(("", "boo")));
}
