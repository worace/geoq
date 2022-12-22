use crate::geoq::{self, bbox::BBoxToPoly, entity::Entity, error::Error, par};
use clap::ArgMatches;
use geo::{
    prelude::{Centroid, Contains, Intersects},
    Geometry, MultiPolygon, Point, Polygon,
};
use geo_types::Coord;
use h3ron::{collections::indexvec::IndexVec, FromH3Index, H3Cell, Index, ToCoordinate, ToPolygon};
use std::{
    collections::{HashSet, VecDeque},
    io::{self, prelude::*},
    str::FromStr,
};

fn parse_resolution(resolution_str: &str) -> Result<u8, Error> {
    let resolution_parsed = resolution_str.parse::<u8>();
    if resolution_parsed.is_err() {
        return Err(Error::InvalidNumberFormat(format!(
            "Expected valid H3 cell resolution: {}",
            resolution_str
        )));
    }
    let resolution = resolution_parsed.unwrap();
    if resolution > 15 {
        return Err(Error::InvalidInput(format!(
            "Invalid H3 cell resolution: {}. Expected number from 0 to 15.",
            resolution_str
        )));
    }
    Ok(resolution)
}

fn read_resolution(matches: &ArgMatches) -> Result<u8, Error> {
    let resolution_arg = matches.value_of("resolution");
    if resolution_arg.is_none() {
        return Err(Error::MissingArgument);
    }

    let resolution_str = resolution_arg.unwrap();
    parse_resolution(resolution_str)
}

fn point(matches: &ArgMatches) -> Result<(), Error> {
    let resolution = read_resolution(matches)?;

    par::for_stdin_entity(move |e| match e.geom() {
        geo_types::Geometry::Point(p) => cell_at_res(p, resolution).map(|c| vec![c.to_string()]),
        _ => Err(Error::InvalidInput(
            "Input for 'geoq h3 point' should be a Point geometry".to_string(),
        )),
    })
}

fn cell_children(cell: H3Cell, _res: Option<u8>) -> Result<Vec<String>, Error> {
    let cell_res = cell.resolution();
    if _res.filter(|r| r <= &cell_res).is_some() {
        return Err(Error::InvalidInput(format!(
            "Resolution must be greater than provided cell resolution."
        )));
    }
    let res: u8 = _res.unwrap_or_else(|| cell.resolution() + 1);
    if cell_res == 15 {
        return Err(Error::InvalidInput(format!(
            "Can't get children for resolution 15 cell. Cell: {}",
            cell.to_string()
        )));
    }
    cell.get_children(res)
        .map_err(|e| {
            Error::InvalidInput(format!(
                "Unable to compute children for H3 Cell: {} -- {}",
                cell.to_string(),
                e
            ))
        })
        .map(|cells| cells.iter().map(|c| c.to_string()).collect())
}

fn parent(matches: &ArgMatches) -> Result<(), Error> {
    let resolution = match read_resolution(matches) {
        Ok(res) => Some(res),
        Err(Error::MissingArgument) => None,
        err => return err.map(|_| ()),
    };

    par::for_stdin_entity(move |e| match e {
        Entity::H3(cell) => {
            let cell_res = cell.resolution();
            if resolution.is_none() && cell_res == 0 {
                Err(Error::InvalidInput(format!(
                    "Can't get parent or ancestor for cell {} at res {} -- Can't go below res 0.",
                    cell.to_string(),
                    cell_res
                )))
            } else {
                let parent_res = resolution.unwrap_or(cell_res - 1);
                if parent_res >= cell_res {
                    Err(Error::InvalidInput(format!(
                        "Parent resolution must be less than or equal to cell resolution. Can't get parent at res {} for cell {} at res {}.",
                        parent_res, cell.to_string(), cell_res
                    )))
                } else {
                    cell.get_parent(parent_res)
                        .map_err(|e| Error::ProgramError(format!("H3 error: {}", e)))
                        .map(|p| vec![p.to_string()])
                }
            }
        }
        _ => Err(Error::InvalidInput(format!(
            "Input for 'geoq h3 children' should be a hexadecimal h3 cell. Got: {}",
            e
        ))),
    })
}

fn resolution() -> Result<(), Error> {
    par::for_stdin_entity(move |e| match e {
        Entity::H3(cell) => Ok(vec![cell.resolution().to_string()]),
        _ => Err(Error::InvalidInput(format!(
            "Input for 'geoq h3 resolution' should be a hexadecimal h3 cell. Got: {}",
            e
        ))),
    })
}

fn children(matches: &ArgMatches) -> Result<(), Error> {
    let resolution = match read_resolution(matches) {
        Ok(res) => Some(res),
        Err(Error::MissingArgument) => None,
        err => return err.map(|_| ()),
    };

    par::for_stdin_entity(move |e| match e {
        Entity::H3(cell) => cell_children(cell, resolution),
        _ => Err(Error::InvalidInput(format!(
            "Input for 'geoq h3 children' should be a hexadecimal h3 cell. Got: {}",
            e
        ))),
    })
}

fn cell_at_res(p: Point<f64>, res: u8) -> Result<H3Cell, Error> {
    H3Cell::from_point(p, res).map_err(|e| {
        Error::InvalidInput(format!(
            "Unable to calculate h3 cell for point {},{} -- {}",
            p.x(),
            p.y(),
            e
        ))
    })
}

fn coord_cell(c: Coord<f64>, res: u8) -> Result<H3Cell, Error> {
    H3Cell::from_coordinate(c, res).map_err(|e| {
        Error::InvalidInput(format!(
            "Unable to calculate h3 cell for point {},{} -- {}",
            c.x, c.y, e
        ))
    })
}

fn hierarchy() -> Result<(), Error> {
    par::for_stdin_entity(move |e| match e.geom() {
        geo_types::Geometry::Point(p) => {
            let res: Result<Vec<String>, Error> = (0..=15)
                .map(|res| cell_at_res(p, res).map(|c| c.to_string()))
                .collect();
            res
        }
        _ => Err(Error::InvalidInput(
            "Input for 'geoq h3 hierarchy' should be a Point geometry".to_string(),
        )),
    })
}

fn from_str() -> Result<(), Error> {
    par::for_stdin_entity(move |e| match e {
        Entity::H3(cell) => Ok(vec![cell.h3index().to_string()]),
        other => Err(Error::InvalidInput(format!(
            "geoq h3 from-str requires H3 cell strings as inputs -- got {}",
            other
        ))),
    })
}

fn h3_from_int(i: u64) -> Result<H3Cell, Error> {
    let cell = H3Cell::from_h3index(i);
    if cell.is_valid() {
        Ok(cell)
    } else {
        Err(Error::InvalidInput(format!(
            "Invalid H3 Index: {} -- Is not a valid 64-bit integer H3 cell representation.",
            i
        )))
    }
}

// match l.parse::<u64>() {
//                Ok(gh_num) => println!("{}", geoq::geohash::encode_long(gh_num)),
//                _ => {
//                    return Err(Error::InvalidNumberFormat(format!(
//                        "Expected u64 geohash number: {}",
//                        l
//                    )))
//                }
//            }
fn to_str() -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let res = l
                    .parse::<u64>()
                    .map_err(|_| {
                        Error::InvalidInput(format!(
                            "Expected valid H3 integer cell index. Got: {}",
                            l
                        ))
                    })
                    .and_then(h3_from_int);

                match res {
                    Ok(cell) => println!("{}", cell.to_string()),
                    e => return e.map(|_| ()),
                }
            }
            _ => return Err(Error::IOError),
        }
    }
    Ok(())
}

fn read_radius(matches: &ArgMatches) -> Result<Option<u32>, Error> {
    let arg = matches.value_of("radius");
    if arg.is_none() {
        return Ok(None);
    }

    let radius_str = arg.unwrap();

    radius_str
        .parse::<u32>()
        .map_err(|_| {
            Error::InvalidNumberFormat(format!("Expected integer radius, got: {}", radius_str))
        })
        .map(|radius| Some(radius))
}

fn cell_disk(cell: H3Cell, radius: u32) -> Result<Vec<String>, Error> {
    cell.grid_disk(radius)
        .map(|cells| cells.iter().map(|c| c.to_string()).collect())
        .map_err(|_| {
            Error::ProgramError(format!(
                "Unable to compute H3 grid disk for cell: {}, radius: {}.",
                cell.to_string(),
                radius
            ))
        })
}

fn grid_disk(matches: &ArgMatches) -> Result<(), Error> {
    let radius_opt = read_radius(matches)?;
    let radius = radius_opt.unwrap_or(1);

    par::for_stdin_entity(move |e| match e {
        Entity::H3(cell) => cell_disk(cell, radius),
        other => Err(Error::InvalidInput(format!(
            "geoq h3 grid-disk requires H3 cell strings as inputs -- got {}",
            other
        ))),
    })
}

fn polyfill_res(matches: &ArgMatches) -> Result<(u8, u8), Error> {
    let min: &str = matches.value_of("min-res").unwrap_or("0");
    let max = matches.value_of("max-res").unwrap_or("15");

    let min_parsed = parse_resolution(min)?;
    let max_parsed = parse_resolution(max)?;

    if min_parsed <= max_parsed {
        Ok((min_parsed, max_parsed))
    } else {
        Err(Error::InvalidInput(format!(
            "Min resolution must be less than or equal to max resolution. Got min: {}, max: {}",
            min_parsed, max_parsed
        )))
    }
}

fn start_cells(geom: &Geometry<f64>, max_res: u8) -> Result<(Vec<H3Cell>, u8), Error> {
    // Attempt to find a starting h3 cell which fully covers the provided geometry
    // This should only fail if the geometry is so large that there is no single cell at
    // any resolution which fully covers it. If this is the case, our starting set
    // will simply be the full set of res 0 cells (assuming a ~continent-sized geometry)
    let starting_res: Option<H3Cell> = match geom.centroid() {
        Some(cen) => {
            let cells: Result<Vec<H3Cell>, Error> = (max_res..=0)
                .map(|res| {
                    H3Cell::from_point(cen, res)
                        .map_err(|e| Error::ProgramError(format!("H3 Error: {}", e)))
                })
                .collect();

            cells.map(|cells| {
                cells.into_iter().find(|cell| match cell.to_polygon() {
                    Ok(poly) => geoq::contains::contains(&poly, geom),
                    _ => false,
                })
            })
        }
        None => Err(Error::ProgramError(
            "Unable to take centroid of geometry".to_string(),
        )),
    }?;

    Ok(starting_res
        .map(|c| (vec![c], c.resolution()))
        .unwrap_or_else(|| (h3ron::res0_cells().iter().collect::<Vec<H3Cell>>(), 0)))
}

struct CellGroup {
    cells: Vec<H3Cell>,
    res: u8,
    parent: Option<H3Cell>,
}

// How does this cell relate to the query geometry to be covered
// cached to avoid recomputing these relationships throughout the function
struct CellRelation {
    cell: H3Cell,
    is_contained: bool,
    intersects: bool,
    centroid_contained: bool,
}

fn group_relations(group: &CellGroup, geometry: &Geometry<f64>) -> Vec<CellRelation> {
    group
        .cells
        .iter()
        .map(|cell| match (cell.to_polygon(), cell.to_coordinate()) {
            (Ok(poly), Ok(center)) => {
                let geom = Geometry::Polygon(poly);
                CellRelation {
                    cell: cell.clone(),
                    is_contained: geoq::contains::contains_any(geometry, &geom),
                    intersects: geom.intersects(geometry),
                    centroid_contained: geoq::contains::contains_any(
                        geometry,
                        &Geometry::Point(Point::new(center.x, center.y)),
                    ),
                }
            }
            _ => CellRelation {
                cell: cell.clone(),
                is_contained: false,
                intersects: false,
                centroid_contained: false,
            },
        })
        .collect()
}

fn top_down_covering_cells(
    geom: &Geometry<f64>,
    min_res: u8,
    max_res: u8,
) -> Result<Vec<H3Cell>, Error> {
    eprintln!("top_down_covering");
    let (start, start_res) = start_cells(geom, max_res)?;
    eprintln!("start_res: {}", start_res);
    let mut queue = VecDeque::<CellGroup>::new();
    let start_group = CellGroup {
        cells: start,
        res: start_res,
        parent: None,
    };
    queue.push_back(start_group);
    let mut cells = Vec::<H3Cell>::new();
    while let Some(candidate) = queue.pop_front() {
        eprintln!(
            "check candidate group at res {}, under parent {}",
            candidate.res,
            candidate
                .parent
                .map(|p| p.to_string())
                .unwrap_or("none".to_string())
        );
        let rels = group_relations(&candidate, geom);
        if candidate.res > min_res
            && rels.iter().all(|rel| rel.centroid_contained)
            && candidate.parent.is_some()
        {
            cells.push(candidate.parent.unwrap())
        } else {
            rels.iter().filter(|r| r.intersects).for_each(|r| {
                if candidate.res == max_res {
                    if r.centroid_contained {
                        cells.push(r.cell)
                    }
                } else if candidate.res < 15 {
                    match r.cell.get_children(candidate.res + 1) {
                        Ok(cells) => {
                            let next_group = CellGroup {
                                cells: cells.into(),
                                res: candidate.res + 1,
                                parent: Some(r.cell),
                            };
                            queue.push_back(next_group);
                        }
                        _ => (),
                    }
                }
            })
        }
    }
    Ok(cells)
}

// This function uses the built-in H3 impl (homogeneous polyfill at specific res)
fn polygon_cells(poly: &Polygon<f64>, res: u8) -> Result<Vec<H3Cell>, Error> {
    h3ron::to_h3::polygon_to_cells(poly, res)
        .map_err(|e| {
            Error::ProgramError(format!(
                "Unable to calculate H3 polygon cells for polygon: {}",
                e
            ))
        })
        .map(|iv| iv.into())
}

fn multi_polygon_cells(mp: &MultiPolygon<f64>, res: u8) -> Result<Vec<H3Cell>, Error> {
    let mut cells = HashSet::<H3Cell>::new();
    for poly in mp.0.iter() {
        let poly_cells = polygon_cells(&poly, res)?;
        for cell in poly_cells.into_iter() {
            cells.insert(cell);
        }
    }
    Ok(cells.into_iter().collect())
}

fn linestring_cells(ls: &geo_types::LineString, res: u8) -> Result<Vec<H3Cell>, Error> {
    if ls.0.is_empty() {
        Ok(vec![])
    } else {
        let start = ls.0.first().unwrap();
        let mut queue = VecDeque::<H3Cell>::new();
        let mut matches = HashSet::<H3Cell>::new();
        let mut seen = HashSet::<H3Cell>::new();
        queue.push_back(coord_cell(*start, res)?);
        while let Some(cell) = queue.pop_front() {
            let poly = cell.to_polygon()?;
            if poly.intersects(ls) {
                matches.insert(cell);
                let neighbors = cell.grid_disk(1)?;
                for c in neighbors.iter() {
                    if !seen.contains(&c) {
                        queue.push_back(c);
                        seen.insert(c);
                    }
                }
            }
        }
        Ok(matches.into_iter().collect())
    }
}

fn flatten_cell_results(r: Result<Vec<Vec<H3Cell>>, Error>) -> Result<Vec<H3Cell>, Error> {
    let mut flattened = Vec::<H3Cell>::new();
    for v in r? {
        for cell in v {
            flattened.push(cell);
        }
    }

    Ok(flattened)
}

fn multi_linestring_cells(mls: &geo_types::MultiLineString, res: u8) -> Result<Vec<H3Cell>, Error> {
    flatten_cell_results(mls.0.iter().map(|ls| linestring_cells(ls, res)).collect())
}

fn gc_cells(g: &geo_types::GeometryCollection<f64>, res: u8) -> Result<Vec<H3Cell>, Error> {
    flatten_cell_results(g.0.iter().map(|g| geom_cells(g, res)).collect())
}

fn geom_cells(g: &geo_types::Geometry<f64>, res: u8) -> Result<Vec<H3Cell>, Error> {
    match g {
        geo_types::Geometry::Point(g) => cell_at_res(*g, res).map(|c| vec![c]),
        geo_types::Geometry::MultiPoint(g) => g.0.iter().map(|p| cell_at_res(*p, res)).collect(),
        geo_types::Geometry::Line(g) => {
            linestring_cells(&geo_types::LineString::new(vec![g.start, g.end]), res)
        }
        geo_types::Geometry::LineString(g) => linestring_cells(&g, res),
        geo_types::Geometry::MultiLineString(g) => multi_linestring_cells(&g, res),
        geo_types::Geometry::Triangle(g) => polygon_cells(&g.to_polygon(), res),
        geo_types::Geometry::Rect(g) => polygon_cells(&g.to_polygon(), res),
        geo_types::Geometry::Polygon(poly) => polygon_cells(&poly, res),
        geo_types::Geometry::MultiPolygon(mp) => multi_polygon_cells(&mp, res),
        geo_types::Geometry::GeometryCollection(g) => gc_cells(&g, res),
    }
}

fn covering(matches: &ArgMatches) -> Result<(), Error> {
    let res = read_resolution(matches)?;
    let include_original = matches.is_present("original");
    let compact = matches.is_present("compact");

    par::for_stdin_entity(move |e| {
        let mut results = if include_original {
            vec![e.raw()]
        } else {
            vec![]
        };
        let cells = match e.geom() {
            geo_types::Geometry::Point(g) => cell_at_res(g, res).map(|c| vec![c]),
            geo_types::Geometry::MultiPoint(g) => {
                g.0.iter().map(|p| cell_at_res(*p, res)).collect()
            }
            geo_types::Geometry::LineString(g) => linestring_cells(&g, res),
            geo_types::Geometry::MultiLineString(g) => multi_linestring_cells(&g, res),
            geo_types::Geometry::Triangle(g) => polygon_cells(&g.to_polygon(), res),
            geo_types::Geometry::Rect(g) => polygon_cells(&g.to_polygon(), res),
            geo_types::Geometry::Polygon(poly) => polygon_cells(&poly, res),
            geo_types::Geometry::MultiPolygon(mp) => multi_polygon_cells(&mp, res),
            _ => Err(Error::InvalidInput(format!(
                "geoq h3 polyfill requires Polygon or MultiPolygon geometries -- got {}",
                e.raw()
            ))),
        };
        match cells {
            Ok(cells) => {
                if compact {
                    let cells_vec: Vec<H3Cell> = cells.into();
                    match h3ron::compact_cells(&cells_vec[0..cells_vec.len()]) {
                        Ok(compacted) => compacted.iter().for_each(|c| results.push(c.to_string())),
                        _ => (),
                    }
                } else {
                    cells.iter().for_each(|c| results.push(c.to_string()));
                }
            }
            _ => (),
        }
        Ok(results)
    })
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("point", Some(m)) => point(m),
        ("children", Some(m)) => children(m),
        ("parent", Some(m)) => parent(m),
        ("hierarchy", _) => hierarchy(),
        ("from-str", _) => from_str(),
        ("to-str", _) => to_str(),
        ("grid-disk", Some(m)) => grid_disk(m),
        ("resolution", _) => resolution(),
        ("covering", Some(m)) => covering(m),
        _ => Err(Error::UnknownCommand),
    }
}
