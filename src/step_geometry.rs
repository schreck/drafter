use std::collections::HashMap;
use std::f64::consts::TAU;

use crate::step_parser::Entity;

const ARC_SEGS: usize = 32;

/// Extract all edge geometry from the entity map as a flat vertex list for `GL_LINES`.
/// Every consecutive pair `[verts[i], verts[i+1]]` is one line segment.
/// No assembly transforms are applied; all geometry is in file-local coordinates.
pub fn extract_segments(entities: &HashMap<u32, Entity>) -> Vec<[f32; 3]> {
    let mut out: Vec<[f32; 3]> = Vec::new();

    for entity in entities.values() {
        let Entity::EdgeCurve { start, end, geom } = entity else {
            continue;
        };
        let Some(p_start) = resolve_vertex(*start, entities) else {
            continue;
        };
        let Some(p_end) = resolve_vertex(*end, entities) else {
            continue;
        };

        match entities.get(geom) {
            Some(Entity::Line { .. }) => {
                out.push(to_f32(p_start));
                out.push(to_f32(p_end));
            }
            Some(Entity::Circle { placement, radius }) => {
                append_arc(&mut out, *placement, *radius, *start, *end, p_start, p_end, entities);
            }
            _ => {
                // Unknown or missing curve type — chord fallback
                out.push(to_f32(p_start));
                out.push(to_f32(p_end));
            }
        }
    }

    out
}

// ── Arc tessellation ──────────────────────────────────────────────────────────

fn append_arc(
    out: &mut Vec<[f32; 3]>,
    placement_id: u32,
    radius: f64,
    start_id: u32,
    end_id: u32,
    p_start: [f64; 3],
    p_end: [f64; 3],
    entities: &HashMap<u32, Entity>,
) {
    let Some((loc_id, axis_id, ref_id)) = get_axis_placement(placement_id, entities) else {
        out.push(to_f32(p_start));
        out.push(to_f32(p_end));
        return;
    };
    let Some(center) = get_cartesian(loc_id, entities) else { return };
    let Some(z_raw) = get_direction(axis_id, entities) else { return };
    let Some(x_raw) = get_direction(ref_id, entities) else { return };

    let x_hat = normalize(x_raw);
    let y_hat = cross(normalize(z_raw), x_hat);

    let (theta_start, theta_end) = if start_id == end_id {
        // Full circle: start and end are the same vertex
        (0.0_f64, TAU)
    } else {
        let a = project_angle(p_start, center, x_hat, y_hat);
        let b = project_angle(p_end, center, x_hat, y_hat);
        // Ensure arc sweeps CCW: b must be strictly greater than a
        let b = if b <= a { b + TAU } else { b };
        (a, b)
    };

    for i in 0..ARC_SEGS {
        let t0 = lerp(theta_start, theta_end, i as f64 / ARC_SEGS as f64);
        let t1 = lerp(theta_start, theta_end, (i + 1) as f64 / ARC_SEGS as f64);
        out.push(to_f32(circle_pt(center, x_hat, y_hat, radius, t0)));
        out.push(to_f32(circle_pt(center, x_hat, y_hat, radius, t1)));
    }
}

/// Angle of world-space point `p` within the circle's local 2-D frame.
fn project_angle(p: [f64; 3], center: [f64; 3], x_hat: [f64; 3], y_hat: [f64; 3]) -> f64 {
    let v = sub(p, center);
    f64::atan2(dot(v, y_hat), dot(v, x_hat))
}

fn circle_pt(center: [f64; 3], x_hat: [f64; 3], y_hat: [f64; 3], r: f64, theta: f64) -> [f64; 3] {
    let (c, s) = (theta.cos(), theta.sin());
    [
        center[0] + r * (c * x_hat[0] + s * y_hat[0]),
        center[1] + r * (c * x_hat[1] + s * y_hat[1]),
        center[2] + r * (c * x_hat[2] + s * y_hat[2]),
    ]
}

// ── Vec3 arithmetic ───────────────────────────────────────────────────────────

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = dot(v, v).sqrt();
    if len < 1e-12 { v } else { [v[0] / len, v[1] / len, v[2] / len] }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

fn to_f32(v: [f64; 3]) -> [f32; 3] {
    [v[0] as f32, v[1] as f32, v[2] as f32]
}

// ── Entity lookups ────────────────────────────────────────────────────────────

fn resolve_vertex(id: u32, entities: &HashMap<u32, Entity>) -> Option<[f64; 3]> {
    match entities.get(&id) {
        Some(Entity::VertexPoint(cp_id)) => get_cartesian(*cp_id, entities),
        _ => None,
    }
}

fn get_cartesian(id: u32, entities: &HashMap<u32, Entity>) -> Option<[f64; 3]> {
    match entities.get(&id) {
        Some(Entity::CartesianPoint(xyz)) => Some(*xyz),
        _ => None,
    }
}

fn get_direction(id: u32, entities: &HashMap<u32, Entity>) -> Option<[f64; 3]> {
    match entities.get(&id) {
        Some(Entity::Direction(xyz)) => Some(*xyz),
        _ => None,
    }
}

fn get_axis_placement(id: u32, entities: &HashMap<u32, Entity>) -> Option<(u32, u32, u32)> {
    match entities.get(&id) {
        Some(Entity::Axis2Placement3D { location, axis, ref_dir }) => {
            Some((*location, *axis, *ref_dir))
        }
        _ => None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_parser::Entity;

    // Build a minimal entity map with a circle placement at the origin,
    // z=(0,0,1), x=(1,0,0), radius=5.  y_hat = cross(z,x) = (0,1,0).
    // Vertices lie in the XY plane.
    fn circle_map(start_xyz: [f64; 3], end_xyz: [f64; 3], same_vertex: bool) -> HashMap<u32, Entity> {
        let mut m = HashMap::new();
        m.insert(1, Entity::CartesianPoint([0.0, 0.0, 0.0])); // center
        m.insert(2, Entity::Direction([0.0, 0.0, 1.0]));      // z axis
        m.insert(3, Entity::Direction([1.0, 0.0, 0.0]));      // x ref
        m.insert(4, Entity::Axis2Placement3D { location: 1, axis: 2, ref_dir: 3 });
        m.insert(5, Entity::Circle { placement: 4, radius: 5.0 });

        m.insert(10, Entity::CartesianPoint(start_xyz));
        m.insert(11, Entity::VertexPoint(10));

        if same_vertex {
            m.insert(20, Entity::EdgeCurve { start: 11, end: 11, geom: 5 });
        } else {
            m.insert(12, Entity::CartesianPoint(end_xyz));
            m.insert(13, Entity::VertexPoint(12));
            m.insert(20, Entity::EdgeCurve { start: 11, end: 13, geom: 5 });
        }
        m
    }

    fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    }

    // ── Line edges ───────────────────────────────────────────────────────────

    #[test]
    fn line_edge_emits_two_vertices() {
        let mut m = HashMap::new();
        m.insert(1, Entity::CartesianPoint([0.0, 0.0, 0.0]));
        m.insert(2, Entity::VertexPoint(1));
        m.insert(3, Entity::CartesianPoint([3.0, 0.0, 0.0]));
        m.insert(4, Entity::VertexPoint(3));
        m.insert(5, Entity::Line { point: 1, dir: 99 }); // dir ref irrelevant
        m.insert(6, Entity::EdgeCurve { start: 2, end: 4, geom: 5 });

        let segs = extract_segments(&m);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], [0.0_f32, 0.0, 0.0]);
        assert_eq!(segs[1], [3.0_f32, 0.0, 0.0]);
    }

    #[test]
    fn missing_geom_falls_back_to_chord() {
        let mut m = HashMap::new();
        m.insert(1, Entity::CartesianPoint([0.0, 0.0, 0.0]));
        m.insert(2, Entity::VertexPoint(1));
        m.insert(3, Entity::CartesianPoint([1.0, 1.0, 1.0]));
        m.insert(4, Entity::VertexPoint(3));
        m.insert(5, Entity::EdgeCurve { start: 2, end: 4, geom: 999 }); // missing

        let segs = extract_segments(&m);
        assert_eq!(segs.len(), 2);
    }

    // ── Full circle ───────────────────────────────────────────────────────────

    #[test]
    fn full_circle_vertex_count() {
        let m = circle_map([5.0, 0.0, 0.0], [5.0, 0.0, 0.0], true);
        assert_eq!(extract_segments(&m).len(), ARC_SEGS * 2);
    }

    #[test]
    fn full_circle_is_closed() {
        let m = circle_map([5.0, 0.0, 0.0], [5.0, 0.0, 0.0], true);
        let segs = extract_segments(&m);
        // First and last vertices should meet (closed loop)
        assert!(dist(segs[0], segs[segs.len() - 1]) < 1e-4);
    }

    #[test]
    fn full_circle_all_points_on_radius() {
        let m = circle_map([5.0, 0.0, 0.0], [5.0, 0.0, 0.0], true);
        for p in extract_segments(&m) {
            let r = (p[0] * p[0] + p[1] * p[1]).sqrt(); // z should be 0
            assert!((r - 5.0).abs() < 1e-4, "radius {r:.6} != 5");
            assert!(p[2].abs() < 1e-4, "z should be zero, got {}", p[2]);
        }
    }

    // ── Quarter-circle arc (0° → 90°) ────────────────────────────────────────

    #[test]
    fn quarter_arc_vertex_count() {
        let m = circle_map([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], false);
        assert_eq!(extract_segments(&m).len(), ARC_SEGS * 2);
    }

    #[test]
    fn quarter_arc_endpoints() {
        let m = circle_map([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], false);
        let segs = extract_segments(&m);

        // First vertex ≈ start point (5, 0, 0)
        assert!(dist(segs[0], [5.0, 0.0, 0.0]) < 1e-4);
        // Last vertex ≈ end point (0, 5, 0)
        assert!(dist(segs[segs.len() - 1], [0.0, 5.0, 0.0]) < 1e-4);
    }

    #[test]
    fn quarter_arc_midpoint_at_45_deg() {
        let m = circle_map([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], false);
        let segs = extract_segments(&m);
        // The segment boundary at i=ARC_SEGS/2 should be near 45°: (√2/2·5, √2/2·5, 0)
        let mid = segs[ARC_SEGS]; // end of segment ARC_SEGS/2
        let expected = 5.0_f32 * std::f32::consts::FRAC_1_SQRT_2;
        assert!((mid[0] - expected).abs() < 0.05, "mid x {:.4}", mid[0]);
        assert!((mid[1] - expected).abs() < 0.05, "mid y {:.4}", mid[1]);
    }

    // ── Arc crossing the atan2 branch cut (angle wrap) ───────────────────────
    //
    // Start at  90° (0, 5, 0), end at 270° (0, -5, 0).
    // atan2 gives b = -π/2 which is ≤ a = π/2, so b gets +2π → 3π/2.
    // The arc sweeps CCW through (-5, 0, 0) at 180°.

    #[test]
    fn arc_wraps_correctly_across_branch_cut() {
        let m = circle_map([0.0, 5.0, 0.0], [0.0, -5.0, 0.0], false);
        let segs = extract_segments(&m);
        assert_eq!(segs.len(), ARC_SEGS * 2);
        assert!(dist(segs[0], [0.0, 5.0, 0.0]) < 1e-4);
        assert!(dist(segs[segs.len() - 1], [0.0, -5.0, 0.0]) < 1e-4);
        // Midpoint of this 180° arc should be near (-5, 0, 0)
        let mid = segs[ARC_SEGS];
        assert!((mid[0] - (-5.0)).abs() < 0.05, "mid x {:.4}", mid[0]);
        assert!(mid[1].abs() < 0.05, "mid y {:.4}", mid[1]);
    }

    // ── Integration: real STEP file ───────────────────────────────────────────

    #[test]
    fn real_file_segments_are_finite_and_plentiful() {
        let entities = crate::step_parser::parse(".data/as1-ac-214.stp");
        let segs = extract_segments(&entities);

        assert!(segs.len() > 1000, "expected >1000 vertices, got {}", segs.len());

        for (i, v) in segs.iter().enumerate() {
            assert!(
                v[0].is_finite() && v[1].is_finite() && v[2].is_finite(),
                "non-finite vertex at index {i}: {v:?}"
            );
        }
    }

    #[test]
    fn real_file_bounding_box_is_sane() {
        let entities = crate::step_parser::parse(".data/as1-ac-214.stp");
        let segs = extract_segments(&entities);

        let (mut xmin, mut xmax) = (f32::MAX, f32::MIN);
        let (mut ymin, mut ymax) = (f32::MAX, f32::MIN);
        let (mut zmin, mut zmax) = (f32::MAX, f32::MIN);
        for v in &segs {
            xmin = xmin.min(v[0]); xmax = xmax.max(v[0]);
            ymin = ymin.min(v[1]); ymax = ymax.max(v[1]);
            zmin = zmin.min(v[2]); zmax = zmax.max(v[2]);
        }
        // Plate is ~180mm wide, ~150mm deep, ~20mm tall (all parts together)
        let x_span = xmax - xmin;
        let y_span = ymax - ymin;
        let z_span = zmax - zmin;
        assert!(x_span > 50.0, "x span too small: {x_span}");
        assert!(y_span > 50.0, "y span too small: {y_span}");
        assert!(z_span > 5.0,  "z span too small: {z_span}");
    }
}
