use super::*;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;

// Check welded topology rather than just comparing a builder with itself:
// interior edges have two faces, each boundary vertex has two neighbors,
// and Euler's formula gives the expected holes and connected bodies.
fn topology(mesh: &TriangleMesh) -> (usize, usize, f64) {
    let mut edges = BTreeMap::new();
    let mut parents: Vec<usize> = (0..mesh.vertices.len()).collect();
    fn root(parents: &mut [usize], mut p: usize) -> usize {
        while parents[p] != p {
            parents[p] = parents[parents[p]];
            p = parents[p];
        }
        p
    }
    let mut area = 0.0;
    assert_eq!(mesh.indices.len() % 3, 0);
    for t in mesh.indices.chunks_exact(3) {
        let [a, b, c] = [t[0], t[1], t[2]];
        let [pa, pb, pc] = [a, b, c].map(|i| mesh.vertices[i as usize]);
        let cross = (pb[0] - pa[0]) as f64 * (pc[1] - pa[1]) as f64
            - (pb[1] - pa[1]) as f64 * (pc[0] - pa[0]) as f64;
        assert!(cross > 0.0, "inverted or degenerate triangle");
        area += cross * 0.5;
        for (a, b) in [(a, b), (b, c), (c, a)] {
            *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            let ra = root(&mut parents, a as usize);
            let rb = root(&mut parents, b as usize);
            parents[ra] = rb;
        }
    }
    let mut boundary_degree = vec![0; mesh.vertices.len()];
    for (&(a, b), &count) in &edges {
        assert!(count == 1 || count == 2, "nonmanifold edge");
        if count == 1 {
            boundary_degree[a as usize] += 1;
            boundary_degree[b as usize] += 1;
        }
    }
    assert!(boundary_degree.iter().all(|&d| d == 0 || d == 2), "open boundary or T-junction");
    let components = (0..parents.len())
        .map(|i| root(&mut parents, i))
        .collect::<BTreeSet<_>>()
        .len();
    let holes = components + edges.len() - mesh.vertices.len() - mesh.indices.len() / 3;
    (components, holes, area)
}

#[test]
fn printable_basic_has_closed_triangle_geometry_in_both_shapes() {
    for ch in '!'..='~' {
        for shape in [FontShape::Meso, FontShape::Lean] {
            let mesh = dista_triangles(ch, shape, FontDetail::Fine).unwrap();
            assert!(!mesh.indices.is_empty(), "missing {ch:?}");
            for &[x, y] in &mesh.vertices {
                assert!(x.is_finite() && y.is_finite());
                assert!(
                    (0.0..=6.0).contains(&x) && (0.0..=11.0).contains(&y),
                    "outside cell: {ch:?} {x} {y}"
                );
            }
            topology(&mesh);
        }
    }
}

#[test]
fn sparse_y_seed_matches_edited_pattern_and_stays_connected() {
    let seed = basic_seed('y').unwrap();
    let sites: Vec<_> = (0..66)
        .filter(|&i| seed.pixels & (1u128 << (65 - i)) != 0)
        .map(|i| (i % 6, i / 6))
        .collect();
    assert_eq!(sites, [(0, 4), (4, 4), (0, 6), (4, 6), (1, 7), (3, 7), (2, 8), (1, 10)]);
    assert!(seed.long_reach);
    for shape in [FontShape::Meso, FontShape::Lean] {
        let mesh = dista_triangles('y', shape, FontDetail::Fine).unwrap();
        let (bodies, holes, _) = topology(&mesh);
        assert_eq!((bodies, holes), (1, 0), "sparse long-reach y must stay connected");
    }
}

#[test]
fn reach_shapes_holes_and_detail_change_real_geometry() {
    assert!(!basic_seed('A').unwrap().long_reach);
    for shape in [FontShape::Meso, FontShape::Lean] {
        let a = dista_triangles('A', shape, FontDetail::Fine).unwrap();
        let (bodies, holes, _) = topology(&a);
        assert_eq!((bodies, holes), (1, 1), "edited A must keep its counter and connected strokes");
        let slash = dista_triangles('/', shape, FontDetail::Fine).unwrap();
        let (bodies, holes, _) = topology(&slash);
        assert_eq!((bodies, holes), (1, 0), "sparse long-reach slash must be one ribbon");
        let equals = dista_triangles('=', shape, FontDetail::Fine).unwrap();
        assert_eq!(topology(&equals).0, 2, "separated strokes must stay separated");
        let ring = dista_triangles('0', shape, FontDetail::Fine).unwrap();
        assert_eq!(topology(&ring).1, 1, "zero must retain its counter");
        let high = dista_triangles('0', shape, FontDetail::High).unwrap();
        assert!(high.indices.len() > ring.indices.len());
        assert!((topology(&high).2 / topology(&ring).2 - 1.0).abs() < 0.03);
    }
    let meso = dista_triangles('A', FontShape::Meso, FontDetail::Fine).unwrap();
    let lean = dista_triangles('A', FontShape::Lean, FontDetail::Fine).unwrap();
    assert!(topology(&meso).2 > topology(&lean).2);
    for ch in ['\0', '\n', ' ', '\u{7f}', 'ü', '文'] {
        assert!(dista_triangles(ch, FontShape::Meso, FontDetail::Fine).is_none());
    }
}

#[test]
fn braille_keeps_original_cells_and_exact_triangle_budgets() {
    // Fixed original Microfont centres, in Unicode bit order. They come from
    // the existing 1x1 pixels, not an evenly redistributed 6x11 dot grid.
    let centres = [
        [1.5, 1.5],
        [1.5, 3.5],
        [1.5, 5.5],
        [4.5, 1.5],
        [4.5, 3.5],
        [4.5, 5.5],
        [1.5, 7.5],
        [4.5, 7.5],
    ];
    for pattern in 0u32..256 {
        let ch = char::from_u32(0x2800 + pattern).unwrap();
        let dots = pattern.count_ones() as usize;
        let mut expected_pixels = 0u128;
        for (bit, &[x, y]) in centres.iter().enumerate() {
            if pattern & (1 << bit) != 0 {
                expected_pixels |= 1u128 << (65 - (y as usize * 6 + x as usize));
            }
        }
        assert_eq!(crate::glyph_cell_pixels(ch), expected_pixels);
        for shape in [FontShape::Meso, FontShape::Lean] {
            for (detail, triangles_per_dot) in [(FontDetail::Fine, 36), (FontDetail::High, 72)] {
                let mesh = dista_braille_triangles(ch, shape, detail).unwrap();
                assert_eq!(mesh.indices.len(), dots * triangles_per_dot * 3);
                assert_eq!(mesh.vertices.len(), dots * (triangles_per_dot + 1));
                let mut dot_index = 0;
                for (bit, &centre) in centres.iter().enumerate() {
                    if pattern & (1 << bit) == 0 {
                        continue;
                    }
                    let start = dot_index * (triangles_per_dot + 1);
                    assert_eq!(mesh.vertices[start], centre);
                    for &p in &mesh.vertices[start + 1..start + 1 + triangles_per_dot] {
                        assert!((p[0] - centre[0]).abs() < 0.5);
                        assert!((p[1] - centre[1]).abs() < 0.5);
                        let r2 = distance_squared(p.map(f64::from), centre.map(f64::from));
                        let level = libm::exp(-r2 / (2.0 * SIGMA * SIGMA));
                        assert!(
                            (level - shape.body_threshold()).abs() < 2e-6,
                            "dot weight differs from Dista's point envelope"
                        );
                    }
                    dot_index += 1;
                }
                let (bodies, holes, area) = topology(&mesh);
                assert_eq!((bodies, holes), (dots, 0));
                if dots != 0 {
                    let expected_area = dots as f64
                        * core::f64::consts::PI
                        * SIGMA
                        * SIGMA
                        * -2.0
                        * libm::log(shape.body_threshold());
                    let tolerance = if detail == FontDetail::Fine {
                        0.006
                    } else {
                        0.0015
                    };
                    assert!((area / expected_area - 1.0).abs() < tolerance);
                }
            }
        }
    }
}

#[test]
fn braille_dispatches_to_direct_circles_and_meso_is_larger() {
    for shape in [FontShape::Meso, FontShape::Lean] {
        for detail in [FontDetail::Fine, FontDetail::High] {
            let direct = dista_braille_triangles('⣿', shape, detail).unwrap();
            let routed = dista_triangles('⣿', shape, detail).unwrap();
            assert_eq!(direct.vertices, routed.vertices);
            assert_eq!(direct.indices, routed.indices);
        }
    }
    let meso = dista_braille_triangles('⠁', FontShape::Meso, FontDetail::Fine).unwrap();
    let lean = dista_braille_triangles('⠁', FontShape::Lean, FontDetail::Fine).unwrap();
    assert!(meso.vertices[1][0] > lean.vertices[1][0]);
    assert!(dista_braille_triangles('A', FontShape::Meso, FontDetail::Fine).is_none());
}
