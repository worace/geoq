use nom::{ IResult,
           branch::alt,
           bytes::streaming::{take_while, tag},
           multi::separated_list,
           number::streaming::double,
           character::streaming::char,
           combinator::map,
           error::{context, ErrorKind, ParseError},
           sequence::{delimited, preceded, separated_pair}
};
#[derive(Debug, PartialEq, Clone)]
pub struct Coordinates(f64, f64);

#[derive(Debug, PartialEq)]
pub enum Geometry {
    Point(Coordinates),
    MultiPoint(Vec<Coordinates>),
    LineString(Vec<Coordinates>),
    MultiLineString(Vec<Vec<Coordinates>>),
    Polygon(Vec<Vec<Coordinates>>),
    MultiPolygon(Vec<Vec<Vec<Coordinates>>>),
    GeometryCollection(Vec<Geometry>)
}


fn json_arr_of<'a, T, Error: ParseError<&'a str>>(elem: impl Fn(&'a str) -> IResult<&'a str, T, Error>) -> impl Fn(&'a str) -> IResult<&'a str, Vec<T>, Error>
where T: std::marker::Sized
{
    delimited(
        spaced(char('[')),
        separated_list(spaced(char(',')), elem),
        spaced(char(']'))
    )
}

fn coord_pair<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Coordinates, E> {
    map(context(
        "coordinates",
        delimited(
            spaced(char('[')),
            separated_pair(spaced(double), spaced(char(',')), spaced(double)),
            spaced(char(']'))
        )),
        |(x,y)| Coordinates(x,y)
    )(i)
}

fn coord_seq<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Coordinates>, E> {
    context("coordinate ring", json_arr_of(coord_pair))(i)
}

fn coord_seq_seq<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Vec<Coordinates>>, E> {
    context("coordinate sequence sequence", json_arr_of(coord_seq))(i)
}

fn coord_seq_seq_seq<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Vec<Vec<Coordinates>>>, E> {
    context("coordinate sequence sequence sequence", json_arr_of(coord_seq_seq))(i)
}

fn spaced<'a, T, Error: ParseError<&'a str>>(inner: impl Fn(&'a str) -> IResult<&'a str, T, Error>) -> impl Fn(&'a str) -> IResult<&'a str, T, Error>
where T: std::marker::Sized
{
    let whitespace = " \t\r\n";
    // delimited(
    //     take_while(move |c| whitespace.contains(c)),
    //     inner,
    //     take_while(move |c| whitespace.contains(c))
    // )
    preceded(
        take_while(move |c| whitespace.contains(c)),
        inner
    )
}

fn json_string<'a, Error: ParseError<&'a str>>(string: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, Error> {
    delimited(char('"'), tag(string), char('"'))
}

fn json_key_with_value<'a, T, Error: ParseError<&'a str>>(key: &'a str, value: impl Fn(&'a str) -> IResult<&'a str, T, Error>) -> impl Fn(&'a str) -> IResult<&'a str, T, Error>
where T: std::marker::Sized
{
    preceded(
        preceded(spaced(json_string(key)),
                 spaced(char(':'))),
        spaced(value)
    )
}

fn type_parser<'a, Error: ParseError<&'a str>>(type_name: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, Error> {
    json_key_with_value("type", delimited(char('"'), tag(type_name), char('"')))
}

fn geometry<'a, CoordT, Error: ParseError<&'a str>>(type_name: &'static str, coord_parser: fn(&'a str) -> IResult<&'a str, CoordT, Error>) -> impl Fn(&'a str) -> IResult<&'a str, CoordT, Error>
where CoordT: std::marker::Sized
{
    let tp1 = type_parser(type_name);
    let tp2 = type_parser(type_name);
    context(type_name,
            delimited(
                spaced(char('{')),
                alt(
                    (map(separated_pair(
                        tp1,
                        spaced(char(',')),
                        json_key_with_value("coordinates", coord_parser)
                    ), |(_, coords)| coords),
                     map(separated_pair(
                         json_key_with_value("coordinates", coord_parser),
                         spaced(char(',')),
                         tp2,
                     ), |(coords, _)| coords))
                ),
                spaced(char('}'))
            )
    )
}

fn point<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(geometry("Point", coord_pair), Geometry::Point)(i)
}

fn linestring<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(geometry("LineString", coord_seq), Geometry::LineString)(i)
}

fn multipoint<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(geometry("MultiPoint", coord_seq), Geometry::MultiPoint)(i)
}

fn multilinestring<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(geometry("MultiLineString", coord_seq_seq), Geometry::MultiLineString)(i)
}

fn polygon<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(geometry("Polygon", coord_seq_seq), Geometry::Polygon)(i)
}

fn multipolygon<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    map(geometry("MultiPolygon", coord_seq_seq_seq), Geometry::MultiPolygon)(i)
}

fn geometry_collection<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    let tp1 = type_parser("GeometryCollection");
    let tp2 = type_parser("GeometryCollection");
    let geoms_1 = json_arr_of(primitive_geometry);
    let geoms_2 = json_arr_of(primitive_geometry);
    context("GeometryCollection",
            map(delimited(
                spaced(char('{')),
                alt(
                    (map(separated_pair(
                        tp1,
                        spaced(char(',')),
                        json_key_with_value("geometries", geoms_1)
                    ), |(_, coords)| coords),
                     map(separated_pair(
                         json_key_with_value("geometries", geoms_2),
                         spaced(char(',')),
                         tp2,
                     ), |(coords, _)| coords))
                ),
                spaced(char('}'))
            ), Geometry::GeometryCollection)
    )(i)
}

pub fn primitive_geometry<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    alt((point, multipoint, linestring, multilinestring, polygon, multipolygon))(i)
}

pub fn root<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Geometry, E> {
    alt(
        (geometry_collection, primitive_geometry)
    )(i)
}

#[cfg(test)]
mod tests {
    use geoq::parser::*;
    use nom::Err::Incomplete;
    use nom::Needed::Size;
    #[test]
    fn test_coord_pair() {
        assert_eq!(coord_pair::<(&str, ErrorKind)>("[0.0,0.0]"), Ok(("", Coordinates(0.0, 0.0))));
        assert_eq!(coord_pair::<(&str, ErrorKind)>(" [ 0.0 , 0.0 ]"), Ok(("", Coordinates(0.0, 0.0))));
        assert_eq!(coord_pair::<(&str, ErrorKind)>(" [ 0.0 , 0.0 ] "), Ok((" ", Coordinates(0.0, 0.0))));
    }

    #[test]
    fn test_coord_seq() {
        assert_eq!(coord_seq::<(&str, ErrorKind)>("[[0.0,0.0],[1.0,2.0]]"),
                   Ok(("", vec![Coordinates(0.0, 0.0), Coordinates(1.0, 2.0)]))
        );
    }

    #[test]
    fn test_coord_seq_seq() {
        assert_eq!(coord_seq_seq::<(&str, ErrorKind)>("[[[0.0,0],[1.0,-2.0]],[[3.567,4.0],[5.0,6.0]]]"),
                   Ok(("", vec![vec![Coordinates(0.0, 0.0), Coordinates(1.0, -2.0)],
                                vec![Coordinates(3.567, 4.0), Coordinates(5.0, 6.0)]]))
        );
    }

    #[test]
    fn test_point() {
        assert_eq!(root::<(&str, ErrorKind)>("{\"type\":\"Point\",\"coordinates\":[0.0,0.0]}"), Ok(("", Geometry::Point(Coordinates(0.0, 0.0)))));
        assert_eq!(root::<(&str, ErrorKind)>("{\"coordinates\":[0.0,0.0],\"type\":\"Point\"}"), Ok(("", Geometry::Point(Coordinates(0.0, 0.0)))));
        assert_eq!(root::<(&str, ErrorKind)>("{\"coordinates\":[0.0,0.0], \"type\" : \"Point\" }"), Ok(("", Geometry::Point(Coordinates(0.0, 0.0)))));
    }

    #[test]
    fn test_linestring() {
        let coords = vec![Coordinates(0.0, 0.0), Coordinates(1.0, 2.0)];
        assert_eq!(root::<(&str, ErrorKind)>("{\"type\":\"LineString\",\"coordinates\":[[0.0,0.0],[1.0,2.0]]}"),
                   Ok(("", Geometry::LineString(coords.clone()))));
        assert_eq!(root::<(&str, ErrorKind)>("{\"coordinates\":[[0.0,0.0],[1.0,2.0]],\"type\":\"LineString\"}"),
                   Ok(("", Geometry::LineString(coords.clone()))));
    }

    #[test]
    fn test_multipoint() {
        let coords = vec![Coordinates(0.0, 0.0), Coordinates(1.0, 2.0)];
        assert_eq!(root::<(&str, ErrorKind)>("{\"type\":\"MultiPoint\",\"coordinates\":[[0.0,0.0],[1.0,2.0]]}"),
                   Ok(("", Geometry::MultiPoint(coords.clone()))));
        assert_eq!(root::<(&str, ErrorKind)>("{\"coordinates\":[[0.0,0.0],[1.0,2.0]],\"type\":\"MultiPoint\"}"),
                   Ok(("", Geometry::MultiPoint(coords.clone()))));
    }

    #[test]
    fn test_multilinestring() {
        let coords = vec![vec![Coordinates(0.0, 0.0), Coordinates(1.0, 2.0)],
                          vec![Coordinates(3.0, 4.0), Coordinates(5.0, 6.0)]];
        assert_eq!(root::<(&str, ErrorKind)>("{\"type\":\"MultiLineString\",\"coordinates\":[[[0.0,0.0],[1.0,2.0]],[[3.0,4.0],[5.0,6.0]]]}"),
                   Ok(("", Geometry::MultiLineString(coords.clone()))));
        assert_eq!(root::<(&str, ErrorKind)>("{\"coordinates\":[[[0.0,0.0],[1.0,2.0]],[[3.0,4.0],[5.0,6.0]]],\"type\":\"MultiLineString\"}"),
                   Ok(("", Geometry::MultiLineString(coords.clone()))));
    }

    #[test]
    fn test_polygon() {
        let poly_str = r#"
          {
            "type": "Polygon",
            "coordinates": [
                [ [100.0, 0.0], [101.0, 0.0], [101.0, 1.0], [100.0, 1.0], [100.0, 0.0] ],
                [ [100.2, 0.2], [100.8, 0.2], [100.8, 0.8], [100.2, 0.8], [100.2, 0.2] ]
            ]
          }"#;

        let coords = vec![vec![Coordinates(100.0, 0.0), Coordinates(101.0, 0.0), Coordinates(101.0, 1.0), Coordinates(100.0, 1.0), Coordinates(100.0, 0.0)],
                          vec![Coordinates(100.2, 0.2), Coordinates(100.8, 0.2), Coordinates(100.8, 0.8), Coordinates(100.2, 0.8), Coordinates(100.2, 0.2)]];
        assert_eq!(root::<(&str, ErrorKind)>(poly_str), Ok(("", Geometry::Polygon(coords.clone()))));
    }

    #[test]
    fn test_multipolygon() {
        let mp_str = r#"
        {
          "type": "MultiPolygon",
          "coordinates": [
              [
                  [ [102.0, 2.0], [103.0, 2.0], [103.0, 3.0], [102.0, 3.0], [102.0, 2.0] ]
              ],
              [
                  [ [100.0, 0.0], [101.0, 0.0], [101.0, 1.0], [100.0, 1.0], [100.0, 0.0] ],
                  [ [100.2, 0.2], [100.8, 0.2], [100.8, 0.8], [100.2, 0.8], [100.2, 0.2] ]
              ]
          ]
        }"#;
        let coords = vec![
            vec![vec![Coordinates(102.0, 2.0), Coordinates(103.0, 2.0), Coordinates(103.0, 3.0), Coordinates(102.0, 3.0), Coordinates(102.0, 2.0)]],
            vec![vec![Coordinates(100.0, 0.0), Coordinates(101.0, 0.0), Coordinates(101.0, 1.0), Coordinates(100.0, 1.0), Coordinates(100.0, 0.0)],
                 vec![Coordinates(100.2, 0.2), Coordinates(100.8, 0.2), Coordinates(100.8, 0.8), Coordinates(100.2, 0.8), Coordinates(100.2, 0.2)]]
        ];
        assert_eq!(root::<(&str, ErrorKind)>(mp_str), Ok(("", Geometry::MultiPolygon(coords.clone()))));
    }

    #[test]
    fn test_geometry_collection() {
        let gc_str = r#"
        {
          "type": "GeometryCollection",
          "geometries": [
              {"type": "Point", "coordinates": [100.0, 0.0]},
              {"type": "LineString", "coordinates": [[101.0, 0.0], [102.0, 1.0]]}
          ]
        }"#;
        let geoms = vec![
            Geometry::Point(Coordinates(100.0, 0.0)),
            Geometry::LineString(vec![Coordinates(101.0, 0.0), Coordinates(102.0, 1.0)]),
        ];
        assert_eq!(root::<(&str, ErrorKind)>(gc_str), Ok(("", Geometry::GeometryCollection(geoms))));
    }

    #[test]
    fn test_partial_input() {
        let p1 = "{\"type\":\"Point\",\"coordi";

        assert_eq!(root::<(&str, ErrorKind)>(p1), Err(Incomplete(Size(11))));
    }

    #[test]
    fn test_multiple_partial_input() {
        // https://github.com/rust-bakery/generator_nom/blob/master/src/main.rs
        // https://docs.rs/nom/5.0.0-beta2/nom/combinator/fn.iterator.html
        // https://docs.rs/circular/0.3.0/circular/
        let p1 = "{\"type\":\"Point\",\"coordinates\":[0.0,0.0]} {\"type\":\"Point\",\"coord";
        let p2 = "inates\":[0.0,0.0]}";
        let mut it = nom::combinator::iterator(p1, root::<(&str, ErrorKind)>);
        let results = it.collect::<Vec<_>>();
        assert_eq!(results, vec![Geometry::Point(Coordinates(0.0, 0.0))]);
        let parser_result: IResult<_, _> = it.finish();
        assert_eq!(Err(Incomplete(Size(11))), parser_result);
        // match parser_result {
        //     Err(Incomplete(Size(_))) => assert_eq!(true, true),
        //     Ok((remaining_input, ())) => assert_eq!(true, true),
        //         _ => assert_eq!(false, true)
        // };
        // let (remaining_input, ()) = parser_result.unwrap();
        // assert_eq!(root::<(&str, ErrorKind)>(p1), Ok((" {\"type\":\"Point\",\"coord", Geometry::Point(Coordinates(0.0, 0.0)))));
        // assert_eq!(root::<(&str, ErrorKind)>("{\"type\":\"Point\",\"coordinates\":[0.0,0.0]}"), Ok(("", Geometry::Point(Coordinates(0.0, 0.0)))));
    }
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


// rustup doc
// rustup doc --book
// cargo doc --open
// cargo test --package geoq geoq::parser
// file:///Users/worace/code/geoq/target/doc/nom/index.html
// ~/code/nom/doc
// http://localhost:4000/Users/worace/code/nom/doc/upgrading_to_nom_5.md
// http://localhost:4000/Users/worace/code/nom/doc/choosing_a_combinator.md


// JSON Spacing
// https://tools.ietf.org/html/rfc4627
// These are the six structural characters:

//    begin-array     = ws %x5B ws  ; [ left square bracket
//    begin-object    = ws %x7B ws  ; { left curly bracket
//    end-array       = ws %x5D ws  ; ] right square bracket
//    end-object      = ws %x7D ws  ; } right curly bracket
//    name-separator  = ws %x3A ws  ; : colon
//    value-separator = ws %x2C ws  ; , comma

// Insignificant whitespace is allowed before or after any of the six
// structural characters.
//    ws = *(
//              %x20 /              ; Space
//              %x09 /              ; Horizontal tab
//              %x0A /              ; Line feed or New line
//              %x0D                ; Carriage return
//          )
