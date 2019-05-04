extern crate pest;

use pest::Parser;
use std::io::BufRead;
use geoq::entity::{self, Entity};
use geoq::input::Input;

#[derive(Parser)]
#[grammar = "geoq.pest"]
pub struct GeoqParser;

pub fn read_inputs(text: &str) -> Result<Vec<Input>, pest::error::Error<Rule>> {
    let mut inputs = vec![];
    match GeoqParser::parse(Rule::geoq, text) {
        Ok(pairs) => {
            for p in pairs {
                let raw = p.as_str().to_owned();
                match p.as_rule() {
                    Rule::geohash => inputs.push(Input::Geohash(raw)),
                    Rule::latlon => inputs.push(Input::LatLon(raw)),
                    Rule::wkt => inputs.push(Input::WKT(raw)),
                    Rule::json_object => inputs.push(Input::GeoJSON(raw)),
                    _ => ()
                }
            }
            Ok(inputs)
        }
        Err(e) => Err(e)
    }
}

        // let mut pointer = "9q5\n9q4".as_bytes();
        // let mut reader = Reader::new(&mut pointer);
// fn to_input(Rule::)


// pub fn read<'a>(instream: &'a BufRead) -> Iterator<Item = Entity> {
//     // let pairs = GeoqParser::parse(Rule::geoq, "9q5 {} POINT(0 0)");
//     vec![].into_iter();
// }

// pub struct Reader<'a> {
//     reader: &'a mut BufRead,
//     entities: VecDeque<Entity>
// }

// impl<'a> Reader<'a> {
//     pub fn new(reader: &'a mut BufRead) -> Reader<'a> {
//         Reader{reader, entities: VecDeque::new()}
//     }
// }


#[cfg(test)]
mod tests {
    use geoq::pest_parser::*;
    use pest::Parser;

    #[test]
    fn test_hello() {
        let pairs = GeoqParser::parse(Rule::geoq, "9q5 {} POINT(0 0) {\"type\":\"Point\", \"coordinates\": [0, 0]}")
            .unwrap_or_else(|e| panic!("{}", e));

        let inputs = pairs.flat_map(|p| {
            let raw = p.as_str().to_owned();
            match p.as_rule() {
                Rule::geohash => vec![Input::Geohash(raw)],
                Rule::latlon => vec![Input::LatLon(raw)],
                Rule::wkt => vec![Input::WKT(raw)],
                Rule::json_object => vec![Input::GeoJSON(raw)],
                _ => vec![]
            }.into_iter()
        });

        let entities = inputs.map(|i| {
            entity::from_input(i)
        });

        for e in entities {
            match e {
                Ok(e) => {
                    for e in e {
                        println!("*** GOT ENTITY ***");
                        println!("{}", e);
                    }
                },
                Err(e) => {
                    println!("** FAILED **");
                    println!("{:?}", e);
                }
            }
        }

        // for pair in pairs {
        //     println!("Rule:  {:?}", pair.as_rule());
        //     println!("Span:    {:?}", pair.as_span());
        //     println!("Text:    {}", pair.as_str());

        // }
        assert_eq!(2,3);
    }
}
