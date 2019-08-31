use nom::{ IResult,
           branch::alt,
           bytes::streaming::{take_while, tag},
           multi::separated_list,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Coordinates(f64, f64);

#[derive(Debug, PartialEq)]
pub enum Geometry {
    Point(Coordinates),
    LineString(Vec<Coordinates>)
}

fn coord_pair<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Coordinates, E> {
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

fn coord_ring<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Coordinates>, E> {
    context(
        "coordinate ring",
        delimited(
            char('['),
            separated_list(char(','), coord_pair),
            char(']')
        )
    )(i)
}

// type Parser<'a, T, E: ParseError<&'a str>> = dyn Fn(&'a str) -> IResult<&'a str, T, E>;

// pub fn pair<I, O1, O2, E: ParseError<I>, F, G>(first: F, second: G) -> impl Fn(I) -> IResult<I, (O1, O2), E>
// where
//   F: Fn(I) -> IResult<I, O1, E>,
//   G: Fn(I) -> IResult<I, O2, E>,
// {
//   move |input: I| {
//     let (input, o1) = first(input)?;
//     second(input).map(|(i, o2)| (i, (o1, o2)))
//   }
// }

fn type_parser<'a, Error: ParseError<&'a str>>(type_name: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, Error> {
    preceded(tag("\"type\":"), delimited(char('"'), tag(type_name), char('"')))
}

fn geometry<'a, CoordT, Error: ParseError<&'a str>>(type_name: &'static str, coord_parser: fn(&'a str) -> IResult<&'a str, CoordT, Error>) -> impl Fn(&'a str) -> IResult<&'a str, CoordT, Error>
where CoordT: std::marker::Sized
{
    let tp1 = type_parser(type_name);
    let tp2 = type_parser(type_name);
    context(type_name,
            delimited(
                char('{'),
                alt(
                    (map(separated_pair(
                        tp1,
                        char(','),
                        preceded(
                            tag("\"coordinates\":"),
                            coord_parser
                        )
                    ), |(_, coords)| coords),
                     map(separated_pair(
                         preceded(
                             tag("\"coordinates\":"),
                             coord_parser
                         ),
                         char(','),
                         tp2,
                     ), |(coords, _)| coords))
                ),
                char('}')
            )
    )
}

fn point<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(geometry("Point", coord_pair), Geometry::Point)(i)
}

fn linestring<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(geometry("LineString", coord_ring), Geometry::LineString)(i)
}

fn root<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    alt(
        (point,
         linestring)
    )(i)
}

#[cfg(test)]
mod tests {
    use geoq::parser::*;
    #[test]
    fn test_coord_pair() {
        assert_eq!(coord_pair::<(&str, ErrorKind)>("[0.0,0.0]"), Ok(("", Coordinates(0.0, 0.0))));
    }

    #[test]
    fn test_coord_ring() {
        assert_eq!(coord_ring::<(&str, ErrorKind)>("[[0.0,0.0],[1.0,2.0]]"),
                   Ok(("", vec![Coordinates(0.0, 0.0), Coordinates(1.0, 2.0)]))
        );
    }

    #[test]
    fn test_point() {
        assert_eq!(root::<(&str, ErrorKind)>("{\"type\":\"Point\",\"coordinates\":[0.0,0.0]}\n"), Ok(("\n", Geometry::Point(Coordinates(0.0, 0.0)))));
        assert_eq!(root::<(&str, ErrorKind)>("{\"coordinates\":[0.0,0.0],\"type\":\"Point\"}\n"), Ok(("\n", Geometry::Point(Coordinates(0.0, 0.0)))));
    }

    #[test]
    fn test_linestring() {
        let coords = vec![Coordinates(0.0, 0.0), Coordinates(1.0, 2.0)];
        assert_eq!(root::<(&str, ErrorKind)>("{\"type\":\"LineString\",\"coordinates\":[[0.0,0.0],[1.0,2.0]]}\n"),
                   Ok(("\n", Geometry::LineString(coords.clone()))));
        assert_eq!(root::<(&str, ErrorKind)>("{\"coordinates\":[[0.0,0.0],[1.0,2.0]],\"type\":\"LineString\"}\n"),
                   Ok(("\n", Geometry::LineString(coords.clone()))));
    }
}
