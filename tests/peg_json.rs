#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate nom;

use nom::peg::grammar;
use nom::{alphanumeric, recognize_float};

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Str(String),
    Boolean(bool),
    Num(f32),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}


#[test]
fn peg_json_test() {

    let json = grammar! {
        parse: JsonValue = <value> "="

        num: f32 = ::recognize_float => { result.parse::<f32>().unwrap() }
        // num: i64 = ("0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9")+ => { result.join("").parse::<i64>().unwrap() }

        string: &'input str = "\"" <::alphanumeric> "\""
        // string: &'input str = "\"" <s: ("a"|"b"|"c"|"d")*> "\"" => { s.join("") }

        boolean: bool = "false" => { false }
                      | "true" => { true }

        array: Vec<JsonValue> = "[" <list: (<value> ",")*> <last: value> "]" => { list.push(last); list }

        key_value: (&'input str, JsonValue) = <string> ":" <value>

        hash: HashMap<String, JsonValue> = "{" <list: (<key_value> ",")*> <last: key_value> "}"
            => {
                list.push(last);
                list.into_iter()
                    .map(|(k, v)| (String::from(k), v))
                    .collect()
                }

        value: JsonValue = hash => { JsonValue::Object(result) }
                         | array => { JsonValue::Array(result) }
                         | string => { JsonValue::Str(String::from(result)) }
                         | num => { JsonValue::Num(result) }
                         | boolean => { JsonValue::Boolean(result) }
    };

    let data = "{\"a\":42.3,\"b\":[\"x\",\"y\",12],\"c\":{\"hello\":\"world\"}}=";
    println!("{:?}", json.parse(data));
    assert!(json.parse(data).is_ok())

    // assert_eq!(json.parse(data), Ok(("", JsonValue::Num(0.0))));
}
