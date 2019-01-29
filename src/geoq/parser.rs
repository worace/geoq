// fn is_any(a: u8) -> bool { (a as char).is_uppercase() }
use std::str;

named!(parse_features<&str,&str>,
       recognize!(
           delimited!( tag_s!("{"),
                       take_until!("}"),
                       // map_res!(take_while!(nom::is_alphanumeric),
                       //          str::from_utf8) ,
                       tag_s!("}")
           )
       )
);


#[cfg(test)]
mod tests {
    use geoq::parser::parse_features;

    #[test]
    fn test_hello() {
        let input = "{inner}";
        let res = parse_features(input);
        println!("***********");
        println!("***********");
        println!("{:?}",res);
        assert_eq!(Ok(("", "{inner}")), res);

        let nested = "{a{b}a}";
        assert_eq!(Ok(("", nested)), parse_features(nested));
    }
}
