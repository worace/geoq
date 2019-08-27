use nom::{ IResult,
           branch::alt,
          bytes::streaming::{take_while, tag},
          number::streaming::double,
          character::streaming::char,
          combinator::map,
          error::{context, ErrorKind, ParseError},
          sequence::{delimited, preceded, separated_pair, terminated}
};

// rustup doc
// rustup doc --book
// cargo doc --open
// cargo test --package geoq geoq::parser
// file:///Users/worace/code/geoq/target/doc/nom/index.html
// ~/code/nom/doc
// http://localhost:4000/Users/worace/code/nom/doc/upgrading_to_nom_5.md
// http://localhost:4000/Users/worace/code/nom/doc/choosing_a_combinator.md

fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  let chars = " \t\r\n";
  take_while(move |c| chars.contains(c))(i)
}

// {"type": "Point", "coordinates": [0,0]}

#[derive(Debug, PartialEq)]
pub struct Coordinates(f64, f64);

#[derive(Debug, PartialEq)]
pub enum Geometry {
    Point(Coordinates)
}

fn coordinates<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Coordinates, E> {
    map(context(
        "coordinates",
        preceded(
            char('['),
            terminated(
                separated_pair(double, char(','), double),
                char(']')
            )
        )),
        |(x,y)| Coordinates(x,y)
    )(i)
}

fn point<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(context("point",
                delimited(
                    char('{'),
                    alt(
                        (map(separated_pair(
                            tag("\"type\":\"Point\""),
                            char(','),
                            preceded(
                                tag("\"coordinates\":"),
                                coordinates
                            )
                        ), |(_, coords)| coords),
                         map(separated_pair(
                             preceded(
                                 tag("\"coordinates\":"),
                                 coordinates
                             ),
                             char(','),
                             tag("\"type\":\"Point\"")
                         ), |(coords, _)| coords))
                    ),
                    char('}')
                )
    ), Geometry::Point)(i)

}

fn root<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    preceded(
        sp,
        point
    )(i)
}



#[cfg(test)]
mod tests {
    use geoq::parser::*;
    #[test]
    fn test_parser_hello() {
        assert_eq!(root::<(&str, ErrorKind)>("{\"type\":\"Point\",\"coordinates\":[0.0,0.0]}\n"), Ok(("\n", Geometry::Point(Coordinates(0.0, 0.0)))));
        assert_eq!(root::<(&str, ErrorKind)>("{\"coordinates\":[0.0,0.0],\"type\":\"Point\"}\n"), Ok(("\n", Geometry::Point(Coordinates(0.0, 0.0)))));
        assert_eq!(1, 1);
    }
}
