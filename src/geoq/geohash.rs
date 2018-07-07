extern crate geohash;

pub fn neighbors(gh: &String, include_self: bool) -> Vec<String> {
    let mut output: Vec<String> = if include_self {
        Vec::with_capacity(9)
    } else {
        Vec::with_capacity(8)
    };

    if include_self {
        output.push(gh.clone());
    }

    let neighbs = geohash::neighbors(gh);
    output.push(neighbs.n);
    output.push(neighbs.ne);
    output.push(neighbs.e);
    output.push(neighbs.se);
    output.push(neighbs.s);
    output.push(neighbs.sw);
    output.push(neighbs.w);
    output.push(neighbs.nw);
    output
}
