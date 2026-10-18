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
fn reach_shapes_holes_and_detail_change_real_geometry() {
    for shape in [FontShape::Meso, FontShape::Lean] {
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
