use std::str;
use nom::{is_alphanumeric, recognize_float};
use std::collections::HashMap;

fn is_non_brace(a: char) -> bool {
    let res = a != '{' && a != '}';
    println!("is_non_brace: {} -- {}", a, res);
    res
}

macro_rules! debug_tag (
    ($i:expr, $tag: expr) => (
        {
            use std::result::Result::*;
            use nom::{Err,Needed,IResult,ErrorKind};
            use nom::{Compare,CompareResult,InputLength,need_more,InputTake};

            println!("checking input: {} for tag: {}", $i, $tag);
            let res: IResult<_,_> = match ($i).compare($tag) {
                CompareResult::Ok => {
                    println!("matched, will consume 1 char");
                    let blen = $tag.input_len();
                    Ok($i.take_split(blen))
                },
                CompareResult::Incomplete => {
                    need_more($i, Needed::Size($tag.input_len()))
                },
                CompareResult::Error => {
                    let e:ErrorKind<u32> = ErrorKind::Tag;
                    Err(Err::Error(nom::Context::Code($i, e)))
                }
            };
            res
        }
    );
);


// POINT(...)
// POLYGON(...)

// https://github.com/Geal/nom/blob/master/doc/choosing_a_combinator.md
// JSON example from docs:
// https://github.com/Geal/nom/blob/master/tests/json.rs


named!(json_content<&str, &str>, take_while!(is_non_brace));

// named!(
//     value<JsonValue>,
//     ws!(alt!(
//         hash   => { |h|   JsonValue::Object(h)            } |
//         array  => { |v|   JsonValue::Array(v)             } |
//         string => { |s|   JsonValue::Str(String::from(s)) } |
//         float  => { |num| JsonValue::Num(num)             }
//     ))
// );

// let nested = "{a{b}a}";
// "{inner}"
named!(json<&str, &str>,
       delimited!( debug_tag!("{"),
                   // alt!(
                   //     json_content | json
                   // ),
                   // take_while!(|_| false),
                   // debug_tag!("x"),
                   // fold_many0!(alt!(
                   //     json_content
                   // ), "", |acc, i| { println!("folding acc: {}, i: {}", acc, i); i }),
                   alt!(json|json_content),
                   // map_res!(take_while!(nom::is_alphanumeric),
                   //          str::from_utf8) ,
                   debug_tag!("}")
       )
);

#[cfg(test)]
mod tests {
    use geoq::parser::json;
    use geoq::parser::*;

    #[test]
    fn test_hello() {
        // println!("{:?}", json("a"));
        println!("{:?}", json("{x}"));
        println!("{:?}", json("{inner}"));
        println!("{:?}", json("{in{n}er}"));
        // let nested = "{a{b}a}";
        // assert_eq!(Ok(("", nested)), json(nested));
        // println!("** Start {{inner}} **");
        // println!("** Start {} **", nested);
        // println!("{:?}", json(nested));

        // println!("{:?}", json("dfasfas"));

        // println!("{:?}", json("{}"));
        // println!("{:?}", json("{}}"));
        // println!("{:?}", json("{{}"));
        // println!("{:?}", json("}"));

        // let input = "{inner}";
        // let res = json(input);
        // println!("{:?}",res);
        // assert_eq!(Ok(("", "{inner}")), res);

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
