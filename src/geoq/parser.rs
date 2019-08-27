use nom::{Err, IResult,
          bytes::streaming::take_while,
          number::streaming::double,
          character::streaming::char,
          separated_pair,
          combinator::map,
          error::{context, convert_error, ErrorKind, ParseError,VerboseError},
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
pub enum Geometry {
    Coordinates((f64, f64))
}

fn coordinates<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(context(
        "coordinates",
        preceded(
            char('['),
            terminated(
                separated_pair(double, char(','), double),
                char(']')
            )
        )),
        Geometry::Coordinates
    )(i)
}

fn root<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    preceded(
        sp,
        coordinates
    )(i)
}



#[cfg(test)]
mod tests {
    use geoq::parser::*;
    #[test]
    fn test_parser_hello() {
        assert_eq!(root::<(&str, ErrorKind)>("[0.0,0.0]\n"), Ok(("\n", Geometry::Coordinates((0.0, 0.0)))));
        assert_eq!(1, 1);
    }
}
