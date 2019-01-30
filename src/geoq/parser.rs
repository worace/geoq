use std::str;
use nom::{is_alphanumeric, recognize_float};
use std::collections::HashMap;

fn is_non_brace(a: char) -> bool {
    let res = a != '{' && a != '}';
    println!("is_non_brace: {} -- {}", a, res);
    res
}

// POINT(...)
// POLYGON(...)

// https://github.com/Geal/nom/blob/master/doc/choosing_a_combinator.md
// JSON example from docs:
// https://github.com/Geal/nom/blob/master/tests/json.rs


// let nested = "{a{b}a}";
named!(json<&str, &str>,
       dbg_dmp!(
           delimited!( tag_s!("{"),
                       alt!(
                           take_while!(is_non_brace) | json
                       ),
                       // take_until!("}"),
                       // map_res!(take_while!(nom::is_alphanumeric),
                       //          str::from_utf8) ,
                       tag_s!("}")
           )
       )
);

#[cfg(test)]
mod tests {
    use geoq::parser::json;
    use geoq::parser::*;

    #[test]
    fn test_hello() {
        let nested = "{a{b}a}";
        // assert_eq!(Ok(("", nested)), json(nested));
        println!("{:?}", json(nested));

        // println!("{:?}", json("dfasfas"));

        // println!("{:?}", json("{}"));
        // println!("{:?}", json("{}}"));
        // println!("{:?}", json("{{}"));
        // println!("{:?}", json("}"));

        // let input = "{inner}";
        // let res = json(input);
        // println!("{:?}",res);
        // assert_eq!(Ok(("", "{inner}")), res);
        assert_eq!(1,0)

    }

    #[test]
    fn test_json_object() {
        let input =
            r#"{
              "a": 42,
              "b": "x"
            }\0"#;

        let mut expected_map = HashMap::new();
        expected_map.insert(String::from("a"), JsonValue::Num(42f32));
        expected_map.insert(String::from("b"), JsonValue::Str(String::from("x")));
        let expected = JsonValue::Object(expected_map);

        assert_eq!(expected, value(input.as_bytes()).unwrap().1);


        println!("{:?}", json_str(input.as_bytes()).unwrap().0);
        println!("{:?}", str::from_utf8(json_str(input.as_bytes()).unwrap().1));

        assert_eq!(1,0);
    }
}



#[derive(Debug, PartialEq)]
pub enum JsonValue {
  Str(String),
  Num(f32),
  Array(Vec<JsonValue>),
  Object(HashMap<String, JsonValue>),
}

named!(float<f32>, flat_map!(recognize_float, parse_to!(f32)));

//FIXME: verify how json strings are formatted
named!(
  string<&str>,
  delimited!(
    char!('"'),
    //map_res!(escaped!(call!(alphanumeric), '\\', is_a!("\"n\\")), str::from_utf8),
    map_res!(
      escaped!(take_while1!(is_alphanumeric), '\\', one_of!("\"n\\")),
      str::from_utf8
    ),
    char!('"')
  )
);

named!(
  array<Vec<JsonValue>>,
  ws!(delimited!(
    char!('['),
    separated_list!(char!(','), value),
    char!(']')
  ))
);

named!(
  key_value<(&str, JsonValue)>,
  ws!(separated_pair!(string, char!(':'), value))
);

named!(
  hash<HashMap<String, JsonValue>>,
  ws!(map!(
    delimited!(char!('{'), separated_list!(char!(','), key_value), char!('}')),
    |tuple_vec| {
      let mut h: HashMap<String, JsonValue> = HashMap::new();
      for (k, v) in tuple_vec {
        h.insert(String::from(k), v);
      }
      h
    }
  ))
);

named!(
    value<JsonValue>,
    ws!(alt!(
        hash   => { |h|   JsonValue::Object(h)            } |
        array  => { |v|   JsonValue::Array(v)             } |
        string => { |s|   JsonValue::Str(String::from(s)) } |
        float  => { |num| JsonValue::Num(num)             }
    ))
);

named!(
    json_str<&[u8]>,
    recognize!(value)
);
