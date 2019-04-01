#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate nom;

use nom::peg::grammar;

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
// };
// // and using the (sub) parsers
// result: u64 = parser.Expr("2+2*(3-5)") // should return -2


#[test]
fn peg_test() {

    // fn execute_op(left: i64, op: &str, right: i64) -> i64 {
    //     match op {
    //         "+" => left + right,
    //         "-" => left - right,
    //         "*" => left * right,
    //         "/" => left / right,
    //         _ => unreachable!()
    //     }
    // }
    //
    // let arithmetic = grammar! {
    //     parse: i64 = expr "=" => { result.0 }
    //
    //     expr: i64 = product (("+" | "-") product)* => {
    //         result.1.iter().fold(result.0, |a, i| execute_op(a, i.0, i.1))
    //     }
    //
    //     product: i64 = value (("*" | "/") value)* => {
    //         result.1.iter().fold(result.0, |a, i| execute_op(a, i.0, i.1))
    //     }
    //
    //     value: i64 = ("0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9")+ => { result.join("").parse::<i64>().unwrap() }
    //                | "(" expr ")" => { result.1 }
    // };

    // this grammar is right-associative, which might give different results for integer division
    // eg. 3*7/2 should equal 10, but here it's executed as 3*(7/2), which equals 9 instead.
    let arithmetic = grammar! {
        // parse: i64 = <expr> "="
        parse: i64 = <expr> "="

        // expr: i64 = <l: product> "+" <r: expr> => { l + r }
        expr: i64 = <l: product> "+" <r: expr> => { l + r }
                  | <l: product> "+" <r: expr> => { l - r }
                  | product

        product: i64 = <l: value> "*" <r: product> => { l * r }
                     | <l: value> "/" <r: product> => { l / r }
                     | value

        value: i64 = ("0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9")+ => { result.join("").parse::<i64>().unwrap() }
                   // | "(" <expr> ")"
                   | "(" <expr> ")"
    };

    // I realised this grammar actually isn't correct,
    // e.g. it doesn't parse: "3*7/2"
    // let arithmetic = grammar! {
    //     parse: i64 = expr "=" => { result.0 }
    //
    //     expr: i64 = product ("+" product => { result.1 })* => { result.1.iter().fold(result.0, |a, i| a + i) }
    //               | product ("-" product => { result.1 })* => { result.1.iter().fold(result.0, |a, i| a + i) }
    //
    //     product: i64 = value ("*" value => { result.1 })* => { result.1.iter().fold(result.0, |a, i| a * i) }
    //                  | value ("/" value => { result.1 })* => { result.1.iter().fold(result.0, |a, i| a / i) }
    //
    //     value: i64 = ("0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9")+ => { result.join("").parse::<i64>().unwrap() }
    //                | "(" expr ")" => { result.1 }
    // };

    assert_eq!(arithmetic.parse("123="), Ok(("", 123 as i64)));
    assert_eq!(arithmetic.parse("1+1="), Ok(("", 2 as i64)));
    assert_eq!(arithmetic.parse("12+(3*7)="), Ok(("", 33 as i64)));
    // assert_eq!(arithmetic.parse("3*7/2="), Ok(("", 10 as i64))); // left-associative
    assert_eq!(arithmetic.parse("3*7/2="), Ok(("", 9 as i64))); // right-associative


    let parser = grammar! {
        p = &"a"* "a"*  => { "yay" }
        q: String = a ("b" | "c") => { result.0 }
          | a "d" => { result.0 }
        // a = "a"* => { result[0] }
        a: String = "a"* => { result.join("") }
        // a: Vec<&'input str> = "a"* => { result }
    };

    assert_eq!(parser.p("abc"), Ok(("bc", "yay")));
    assert_eq!(parser.p("aaaaaaab"), Ok(("b", "yay")));

    assert_eq!(parser.q("aaabcc"), Ok(("cc", String::from("aaa"))));
    assert_eq!(parser.q("aac"), Ok(("", String::from("aa"))));
}
