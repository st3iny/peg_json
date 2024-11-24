use std::{
    collections::HashMap,
    io::{stdin, Read},
    vec,
};

#[derive(Debug, Clone)]
enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

peg::parser! {
    // TODO: handle exponential number format, e.g. 1e+10
    grammar json_parser() for str {
        rule __() // whitespace
            = $(['\n' | '\r' | '\t' | ' ']*) {}

        rule int() -> Value
            = d:$("-"? ['0'..='9']+) { Value::Integer(d.parse().unwrap()) }

        rule float() -> Value
            = d:$("-"? ['0'..='9']+ "." ['0'..='9']+) { Value::Float(d.parse().unwrap()) }

        rule null() -> Value
            = "null" { Value::Null }

        rule true() -> Value
            = "true" { Value::Bool(true) }

        rule false() -> Value
            = "false" { Value::Bool(false) }

        rule bool() -> Value
            = true() / false()

        rule char() -> char
            = c:$([^ '"' | '\\']) { c.chars().next().unwrap() }

        rule escape_hex() -> char
            = "\\u" h:$(['a'..='z' | 'A'..='Z' | '0'..='9']) {
                let char_value = u32::from_str_radix(h, 16).unwrap();
                char::from_u32(char_value).unwrap()
            }

        rule escape() -> char
            = "\\" e:$(['"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't']) {
                match e {
                    "b" => unsafe { char::from_u32_unchecked(0x08) }, // backspace
                    "f" => unsafe { char::from_u32_unchecked(0x0c) }, // form feed
                    "n" => '\n',
                    "r" => '\r',
                    "t" => '\t',
                    _ => e.chars().next().unwrap(),
                }
            }

        rule string() -> String
            = "\"" s:(escape_hex() / escape() / char())* "\"" {
                let mut string = String::with_capacity(s.len());
                string.extend(s.into_iter());
                string
            }

        rule comma_val() -> Value
            = "," __ v:value() { v }

        rule array() -> Value
            = "[" __ e:value()? __ es:comma_val()* __ "]" {
                let mut vec = Vec::with_capacity(1 + es.len());
                if let Some(e) = e {
                    vec.push(e);
                }
                vec.extend(es.into_iter());
                Value::Array(vec)
            }

        rule object_entry() -> (String, Value)
            = k:string() __ ":" __ v:value() { (k, v) }

        rule object_comma_entry() -> (String, Value)
            = "," __ entry:object_entry() { entry }

        rule object() -> Value
            = "{" __ e:object_entry()? __ es:object_comma_entry()* __ "}" {
                let mut es: std::collections::HashMap<String, Value> = es.into_iter().collect();
                if let Some((k, v)) = e {
                    es.insert(k, v);
                }
                Value::Object(es)
            }

        rule string_value() -> Value
            = s:string() { Value::String(s) }

        rule value() -> Value
            = string_value() / float() / int() / object() / array() / bool() / null()

        pub rule document() -> Value
            = __ v:value() __ { v }
    }
}

fn main() {
    let mut json = String::new();
    stdin()
        .lock()
        .read_to_string(&mut json)
        .expect("To read a utf-8 string from stdin until EOF");
    let parsed = json_parser::document(&json).expect("The parser to parse json");
    println!("{parsed:#?}");
}
