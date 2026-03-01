use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    CartesianPoint([f64; 3]),
    Direction([f64; 3]),
    Vector { dir: u32, magnitude: f64 },
    Axis2Placement3D { location: u32, axis: u32, ref_dir: u32 },
    VertexPoint(u32),
    Line { point: u32, dir: u32 },
    Circle { placement: u32, radius: f64 },
    EdgeCurve { start: u32, end: u32, geom: u32 },
    OrientedEdge { edge: u32, sense: bool },
    EdgeLoop(Vec<u32>),
    FaceBound(u32),
    AdvancedFace { bounds: Vec<u32>, surface: u32 },
    ClosedShell(Vec<u32>),
    ManifoldSolidBrep(u32),
    ItemDefinedTransformation { from: u32, to: u32 },
}

// ── Tokeniser ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ref(u32),
    Float(f64),
    Bool(bool),
    Str,
    List(Vec<Token>),
    Wildcard,
    Null,
    Enum(String),
}

/// Tokenise a comma-separated STEP argument string (without the outer parens).
fn tokenize(s: &str) -> Vec<Token> {
    let b = s.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();

    while i < b.len() {
        match b[i] {
            b' ' | b'\t' | b',' => i += 1,

            b'#' => {
                i += 1;
                let start = i;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                let n: u32 = s[start..i].parse().unwrap();
                out.push(Token::Ref(n));
            }

            b'\'' => {
                i += 1;
                while i < b.len() && b[i] != b'\'' {
                    i += 1;
                }
                i += 1; // skip closing '
                out.push(Token::Str);
            }

            b'.' => {
                i += 1;
                let start = i;
                while i < b.len() && b[i] != b'.' {
                    i += 1;
                }
                let name = &s[start..i];
                i += 1; // skip closing .
                out.push(match name {
                    "T" => Token::Bool(true),
                    "F" => Token::Bool(false),
                    _ => Token::Enum(name.to_string()),
                });
            }

            b'*' => {
                out.push(Token::Wildcard);
                i += 1;
            }

            b'$' => {
                out.push(Token::Null);
                i += 1;
            }

            b'(' => {
                i += 1;
                let start = i;
                let mut depth = 1usize;
                while i < b.len() && depth > 0 {
                    if b[i] == b'(' {
                        depth += 1;
                    } else if b[i] == b')' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        i += 1;
                    }
                }
                let inner = &s[start..i];
                i += 1; // skip closing )
                out.push(Token::List(tokenize(inner)));
            }

            // Number: leading digit, +, or -
            c if c == b'-' || c == b'+' || c.is_ascii_digit() => {
                let start = i;
                i += 1;
                while i < b.len() {
                    match b[i] {
                        x if x.is_ascii_digit() || x == b'.' => i += 1,
                        b'E' | b'e' => {
                            i += 1;
                            if i < b.len() && (b[i] == b'-' || b[i] == b'+') {
                                i += 1;
                            }
                        }
                        _ => break,
                    }
                }
                let f: f64 = s[start..i].parse().unwrap_or(0.0);
                out.push(Token::Float(f));
            }

            _ => i += 1,
        }
    }

    out
}

// ── Helper accessors ─────────────────────────────────────────────────────────

fn ref_at(toks: &[Token], i: usize) -> Option<u32> {
    match toks.get(i)? {
        Token::Ref(n) => Some(*n),
        _ => None,
    }
}

fn float_at(toks: &[Token], i: usize) -> Option<f64> {
    match toks.get(i)? {
        Token::Float(f) => Some(*f),
        _ => None,
    }
}

fn refs_from_list(toks: &[Token]) -> Vec<u32> {
    toks.iter()
        .filter_map(|t| if let Token::Ref(n) = t { Some(*n) } else { None })
        .collect()
}

// ── Entity dispatch ───────────────────────────────────────────────────────────

fn parse_entity(type_name: &str, toks: &[Token]) -> Option<Entity> {
    match type_name {
        "CARTESIAN_POINT" | "DIRECTION" => {
            // ('name', (x, y, z))
            let list = match toks.get(1)? {
                Token::List(v) => v,
                _ => return None,
            };
            let xyz = [float_at(list, 0)?, float_at(list, 1)?, float_at(list, 2)?];
            Some(if type_name == "CARTESIAN_POINT" {
                Entity::CartesianPoint(xyz)
            } else {
                Entity::Direction(xyz)
            })
        }

        "VERTEX_POINT" => Some(Entity::VertexPoint(ref_at(toks, 1)?)),

        "VECTOR" => Some(Entity::Vector {
            dir: ref_at(toks, 1)?,
            magnitude: float_at(toks, 2)?,
        }),

        "AXIS2_PLACEMENT_3D" => Some(Entity::Axis2Placement3D {
            location: ref_at(toks, 1)?,
            axis: ref_at(toks, 2)?,
            ref_dir: ref_at(toks, 3)?,
        }),

        "LINE" => Some(Entity::Line {
            point: ref_at(toks, 1)?,
            dir: ref_at(toks, 2)?,
        }),

        "CIRCLE" => Some(Entity::Circle {
            placement: ref_at(toks, 1)?,
            radius: float_at(toks, 2)?,
        }),

        "EDGE_CURVE" => Some(Entity::EdgeCurve {
            start: ref_at(toks, 1)?,
            end: ref_at(toks, 2)?,
            geom: ref_at(toks, 3)?,
        }),

        "ORIENTED_EDGE" => {
            // ('name', *, *, #edge, .T./.F.)
            let sense = match toks.get(4)? {
                Token::Bool(b) => *b,
                _ => return None,
            };
            Some(Entity::OrientedEdge { edge: ref_at(toks, 3)?, sense })
        }

        "EDGE_LOOP" => {
            let list = match toks.get(1)? {
                Token::List(v) => v,
                _ => return None,
            };
            Some(Entity::EdgeLoop(refs_from_list(list)))
        }

        "FACE_OUTER_BOUND" | "FACE_BOUND" => {
            // ('name', #loop, .T.)
            Some(Entity::FaceBound(ref_at(toks, 1)?))
        }

        "ADVANCED_FACE" => {
            // ('name', (#bounds...), #surface, .T./.F.)
            let list = match toks.get(1)? {
                Token::List(v) => v,
                _ => return None,
            };
            Some(Entity::AdvancedFace {
                bounds: refs_from_list(list),
                surface: ref_at(toks, 2)?,
            })
        }

        "CLOSED_SHELL" => {
            let list = match toks.get(1)? {
                Token::List(v) => v,
                _ => return None,
            };
            Some(Entity::ClosedShell(refs_from_list(list)))
        }

        "MANIFOLD_SOLID_BREP" => Some(Entity::ManifoldSolidBrep(ref_at(toks, 1)?)),

        "ITEM_DEFINED_TRANSFORMATION" => {
            // ('name', 'desc', #from, #to)
            Some(Entity::ItemDefinedTransformation {
                from: ref_at(toks, 2)?,
                to: ref_at(toks, 3)?,
            })
        }

        _ => None,
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn parse(path: &str) -> HashMap<u32, Entity> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Cannot read STEP file {path}: {e}"));
    parse_str(&text)
}

pub fn parse_str(text: &str) -> HashMap<u32, Entity> {
    let mut map = HashMap::new();
    let mut in_data = false;
    let mut buf = String::new();

    for line in text.lines() {
        let line = line.trim();

        if line == "DATA;" {
            in_data = true;
            continue;
        }
        if line == "ENDSEC;" {
            in_data = false;
            buf.clear();
            continue;
        }
        if !in_data {
            continue;
        }

        buf.push_str(line);

        // An entity record is complete when the buffer ends with ';'
        if !buf.ends_with(';') {
            continue;
        }

        let record = buf.trim_end_matches(';');
        process_record(record, &mut map);
        buf.clear();
    }

    map
}

/// Parse one complete entity record (the text between `#N=` … `;`) into the map.
fn process_record(record: &str, map: &mut HashMap<u32, Entity>) {
    if !record.starts_with('#') {
        return;
    }

    let (id_str, rest) = match record.split_once('=') {
        Some(p) => p,
        None => return,
    };
    let id: u32 = match id_str.trim_start_matches('#').parse() {
        Ok(n) => n,
        Err(_) => return,
    };

    // Compound entity: #N=(...) — skip
    if rest.starts_with('(') {
        return;
    }

    // Split TYPE_NAME from the outer ( ... )
    let open = match rest.find('(') {
        Some(p) => p,
        None => return,
    };
    let type_name = &rest[..open];

    // Find the matching closing paren for the outer (
    let after_open = &rest[open + 1..];
    let ab = after_open.as_bytes();
    let mut depth = 1usize;
    let mut close = None;
    for (j, &c) in ab.iter().enumerate() {
        if c == b'(' {
            depth += 1;
        } else if c == b')' {
            depth -= 1;
            if depth == 0 {
                close = Some(j);
                break;
            }
        }
    }
    let args_str = match close {
        Some(j) => &after_open[..j],
        None => return,
    };

    let toks = tokenize(args_str);
    if let Some(entity) = parse_entity(type_name, &toks) {
        map.insert(id, entity);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn step(entities: &str) -> String {
        format!("ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\n{entities}\nENDSEC;\nEND-ISO-10303-21;\n")
    }

    // ── Tokeniser ────────────────────────────────────────────────────────────

    #[test]
    fn tokenize_refs_and_floats() {
        let toks = tokenize("#85,#87,20.0");
        assert_eq!(toks, vec![Token::Ref(85), Token::Ref(87), Token::Float(20.0)]);
    }

    #[test]
    fn tokenize_string_and_list() {
        let toks = tokenize("'',(1.0,2.0,3.0)");
        assert_eq!(
            toks,
            vec![
                Token::Str,
                Token::List(vec![Token::Float(1.0), Token::Float(2.0), Token::Float(3.0)])
            ]
        );
    }

    #[test]
    fn tokenize_bool_and_wildcard() {
        let toks = tokenize("*,*,#89,.F.");
        assert_eq!(
            toks,
            vec![Token::Wildcard, Token::Wildcard, Token::Ref(89), Token::Bool(false)]
        );
    }

    #[test]
    fn tokenize_scientific_notation() {
        let toks = tokenize("70.0,-3.491483E-014,-10.0");
        assert!(matches!(toks[0], Token::Float(x) if (x - 70.0).abs() < 1e-9));
        assert!(matches!(toks[1], Token::Float(x) if x.abs() < 1e-10));
        assert!(matches!(toks[2], Token::Float(x) if (x - (-10.0)).abs() < 1e-9));
    }

    #[test]
    fn tokenize_nested_list() {
        // EDGE_LOOP style: list of refs inside outer list
        let toks = tokenize("'',(#90,#99,#107)");
        assert_eq!(
            toks,
            vec![
                Token::Str,
                Token::List(vec![Token::Ref(90), Token::Ref(99), Token::Ref(107)])
            ]
        );
    }

    // ── Entity parsing ───────────────────────────────────────────────────────

    #[test]
    fn parse_cartesian_point() {
        let map = parse_str(&step("#1=CARTESIAN_POINT('',(1.0,2.0,3.0));"));
        assert_eq!(map[&1], Entity::CartesianPoint([1.0, 2.0, 3.0]));
    }

    #[test]
    fn parse_cartesian_point_named() {
        // String arg can be 'NONE' or any label, not just ''
        let map = parse_str(&step("#1=CARTESIAN_POINT('NONE',(0.0,0.0,0.0));"));
        assert_eq!(map[&1], Entity::CartesianPoint([0.0, 0.0, 0.0]));
    }

    #[test]
    fn parse_cartesian_point_scientific() {
        let map = parse_str(&step("#1=CARTESIAN_POINT('',(70.0,-3.491483E-014,-10.0));"));
        if let Entity::CartesianPoint([x, y, z]) = map[&1] {
            assert!((x - 70.0).abs() < 1e-9);
            assert!(y.abs() < 1e-10);
            assert!((z - (-10.0)).abs() < 1e-9);
        } else {
            panic!("wrong entity");
        }
    }

    #[test]
    fn parse_direction() {
        let map = parse_str(&step("#2=DIRECTION('',(0.0,0.0,1.0));"));
        assert_eq!(map[&2], Entity::Direction([0.0, 0.0, 1.0]));
    }

    #[test]
    fn parse_vertex_point() {
        let map = parse_str(&step("#3=VERTEX_POINT('',#42);"));
        assert_eq!(map[&3], Entity::VertexPoint(42));
    }

    #[test]
    fn parse_vector() {
        let map = parse_str(&step("#4=VECTOR('',#86,20.0);"));
        assert_eq!(map[&4], Entity::Vector { dir: 86, magnitude: 20.0 });
    }

    #[test]
    fn parse_line() {
        let map = parse_str(&step("#5=LINE('',#85,#87);"));
        assert_eq!(map[&5], Entity::Line { point: 85, dir: 87 });
    }

    #[test]
    fn parse_circle() {
        let map = parse_str(&step("#6=CIRCLE('',#96,5.0);"));
        assert_eq!(map[&6], Entity::Circle { placement: 96, radius: 5.0 });
    }

    #[test]
    fn parse_axis2_placement_3d() {
        let map = parse_str(&step("#7=AXIS2_PLACEMENT_3D('',#58,#59,#60);"));
        assert_eq!(
            map[&7],
            Entity::Axis2Placement3D { location: 58, axis: 59, ref_dir: 60 }
        );
    }

    #[test]
    fn parse_edge_curve() {
        let map = parse_str(&step("#8=EDGE_CURVE('',#82,#84,#88,.T.);"));
        assert_eq!(map[&8], Entity::EdgeCurve { start: 82, end: 84, geom: 88 });
    }

    #[test]
    fn parse_oriented_edge_false() {
        let map = parse_str(&step("#9=ORIENTED_EDGE('',*,*,#89,.F.);"));
        assert_eq!(map[&9], Entity::OrientedEdge { edge: 89, sense: false });
    }

    #[test]
    fn parse_oriented_edge_true() {
        let map = parse_str(&step("#9=ORIENTED_EDGE('',*,*,#89,.T.);"));
        assert_eq!(map[&9], Entity::OrientedEdge { edge: 89, sense: true });
    }

    #[test]
    fn parse_edge_loop() {
        let map = parse_str(&step("#10=EDGE_LOOP('',(#90,#99,#107,#114));"));
        assert_eq!(map[&10], Entity::EdgeLoop(vec![90, 99, 107, 114]));
    }

    #[test]
    fn parse_edge_loop_two_entries() {
        let map = parse_str(&step("#10=EDGE_LOOP('',(#655,#656));"));
        assert_eq!(map[&10], Entity::EdgeLoop(vec![655, 656]));
    }

    #[test]
    fn parse_face_outer_bound() {
        let map = parse_str(&step("#11=FACE_OUTER_BOUND('',#115,.T.);"));
        assert_eq!(map[&11], Entity::FaceBound(115));
    }

    #[test]
    fn parse_face_bound() {
        let map = parse_str(&step("#11=FACE_BOUND('',#657,.T.);"));
        assert_eq!(map[&11], Entity::FaceBound(657));
    }

    #[test]
    fn parse_advanced_face_single_bound() {
        let map = parse_str(&step("#12=ADVANCED_FACE('',(#116),#80,.F.);"));
        assert_eq!(
            map[&12],
            Entity::AdvancedFace { bounds: vec![116], surface: 80 }
        );
    }

    #[test]
    fn parse_advanced_face_multiple_bounds() {
        let map = parse_str(&step("#12=ADVANCED_FACE('',(#642,#646,#650,#654),#636,.F.);"));
        assert_eq!(
            map[&12],
            Entity::AdvancedFace { bounds: vec![642, 646, 650, 654], surface: 636 }
        );
    }

    #[test]
    fn parse_closed_shell() {
        let map = parse_str(&step(
            "#13=CLOSED_SHELL('',(#117,#159,#201,#243,#285,#327));",
        ));
        assert_eq!(
            map[&13],
            Entity::ClosedShell(vec![117, 159, 201, 243, 285, 327])
        );
    }

    #[test]
    fn parse_manifold_solid_brep() {
        let map = parse_str(&step("#14=MANIFOLD_SOLID_BREP('100',#668);"));
        assert_eq!(map[&14], Entity::ManifoldSolidBrep(668));
    }

    #[test]
    fn parse_item_defined_transformation() {
        let map =
            parse_str(&step("#15=ITEM_DEFINED_TRANSFORMATION('IDT1','',#61,#685);"));
        assert_eq!(
            map[&15],
            Entity::ItemDefinedTransformation { from: 61, to: 685 }
        );
    }

    // ── Skip rules ───────────────────────────────────────────────────────────

    #[test]
    fn compound_entity_is_skipped() {
        let map = parse_str(&step(
            "#8=(NAMED_UNIT(*)PLANE_ANGLE_UNIT()SI_UNIT($,.RADIAN.));",
        ));
        assert!(!map.contains_key(&8));
    }

    #[test]
    fn unknown_entity_is_skipped() {
        let map = parse_str(&step("#1=DRAUGHTING_PRE_DEFINED_COLOUR('red');"));
        assert!(!map.contains_key(&1));
    }

    #[test]
    fn header_lines_are_skipped() {
        // Lines before DATA; should never be parsed as entities
        let text = "ISO-10303-21;\nHEADER;\n#99=CARTESIAN_POINT('',(9.0,9.0,9.0));\nENDSEC;\nDATA;\nENDSEC;\n";
        let map = parse_str(text);
        assert!(!map.contains_key(&99));
    }

    // ── Multi-entity roundtrip ────────────────────────────────────────────────

    #[test]
    fn parse_multiple_entities() {
        let input = step(concat!(
            "#1=CARTESIAN_POINT('',(1.0,2.0,3.0));\n",
            "#2=VERTEX_POINT('',#1);\n",
            "#3=DIRECTION('',(0.0,1.0,0.0));\n",
        ));
        let map = parse_str(&input);
        assert_eq!(map.len(), 3);
        assert_eq!(map[&1], Entity::CartesianPoint([1.0, 2.0, 3.0]));
        assert_eq!(map[&2], Entity::VertexPoint(1));
        assert_eq!(map[&3], Entity::Direction([0.0, 1.0, 0.0]));
    }

    // ── Multi-line entities ───────────────────────────────────────────────────

    #[test]
    fn multiline_closed_shell() {
        // Mirrors the Unigraphics wrapping style: list split across two lines
        let text = "ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\n\
            #1=CLOSED_SHELL('',(#60,#61,#62,#63,#64,#65,#66,#67,#68,#69,#70,#71,#72,\n\
            #73,#74,#75,#76));\nENDSEC;\n";
        let map = parse_str(text);
        assert_eq!(
            map[&1],
            Entity::ClosedShell(vec![60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76])
        );
    }

    #[test]
    fn multiline_string_arg() {
        // Entity whose string argument wraps to a second line
        let text = "ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\n\
            #1=VERTEX_POINT('',\n\
            #42);\nENDSEC;\n";
        let map = parse_str(text);
        assert_eq!(map[&1], Entity::VertexPoint(42));
    }

    // ── Trailing-dot floats (Unigraphics style: `18.` not `18.0`) ────────────

    #[test]
    fn trailing_dot_float_circle() {
        let map = parse_str(&step("#1=CIRCLE('',#96,18.);"));
        assert_eq!(map[&1], Entity::Circle { placement: 96, radius: 18.0 });
    }

    #[test]
    fn trailing_dot_float_direction() {
        let map = parse_str(&step("#1=DIRECTION('',(0.,-1.,0.));"));
        assert_eq!(map[&1], Entity::Direction([0.0, -1.0, 0.0]));
    }

    // ── Integration: real files ───────────────────────────────────────────────

    #[test]
    fn real_file_as1_parses_without_panic() {
        let map = parse(".data/as1-ac-214.stp");
        assert!(map.contains_key(&669),  "missing BREP #669 (PLATE)");
        assert!(map.contains_key(&1227), "missing BREP #1227 (L-bracket 1)");
        assert!(map.contains_key(&1439), "missing BREP #1439 (L-bracket 2)");
        assert!(map.contains_key(&89));
        assert!(map.len() > 500, "expected many entities, got {}", map.len());
    }

    #[test]
    fn real_file_io1_parses_without_panic() {
        let map = parse(".data/io1-ug-214.stp");
        // Single-part file: BREP at #42, shell at #43
        assert!(map.contains_key(&42), "missing BREP #42");
        assert!(map.contains_key(&43), "missing ClosedShell #43");
        assert!(map.len() > 100, "expected many entities, got {}", map.len());
    }
}
