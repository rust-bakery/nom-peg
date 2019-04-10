# PEG Grammars for Nom

nom-peg is a PEG (Parsing Expression Grammar) parser generator built on top of [nom](https://github.com/Geal/nom), using a syntax that is heavily inspired by [LALRPOP](https://github.com/lalrpop/lalrpop).

Grammars defined with nom-peg can freely be mixed with other nom parsers.

## Example
```rust
let arithmetic = grammar! {
    // a grammar can have as many non-terminals as you want, and can return any type
    parse: i64 = <expr> "="

    // alternatives are separated by `|`,
    // and the `=> { ... }` syntax is used to manipulate the output of the parser before returning it
    expr: i64 = <l: product> "+" <r: expr> => { l + r }
              | <l: product> "+" <r: expr> => { l - r }
              | product

    // the `<...>` syntax is used to capture the output of a sub-parser,
    // and optionally assign it to a local variable with `<name: ...>`
    product: i64 = <l: value> "*" <r: product> => { l * r }
                 | <l: value> "/" <r: product> => { l / r }
                 | value

    value: i64 = ("0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9")+ => { result.join("").parse::<i64>().unwrap() }
               | "(" <expr> ")"
};

// when the grammar is defined you can use any of the non-terminals as parser functions
assert_eq!(arithmetic.parse("123="), Ok(("", 123 as i64)));
assert_eq!(arithmetic.parse("1+1="), Ok(("", 2 as i64)));
assert_eq!(arithmetic.parse("12+(3*7)="), Ok(("", 33 as i64)));
```
