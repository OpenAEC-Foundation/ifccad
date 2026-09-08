use super::*;

/// Compare semantic content through the same interface for either backing.
pub(crate) fn assert_resource_eq<A: IfcdrResourceAccess, B: IfcdrResourceAccess>(a: &A, b: &B) {
    assert_eq!(a.resource_id(), b.resource_id());
    assert_eq!(a.unit(), b.unit());
    assert_eq!(a.next_entity_id(), b.next_entity_id());
    assert_eq!(a.bounds(), b.bounds());
    assert_eq!(a.scopes(), b.scopes());
    assert_eq!(a.layers(), b.layers());
    assert_eq!(a.appearances(), b.appearances());
    assert_eq!(a.overrides(), b.overrides());
    assert_eq!(a.orders(), b.orders());
    let (al, bl) = (a.lines(), b.lines());
    assert_eq!(al.len(), bl.len());
    assert_eq!(al.is_empty(), bl.is_empty());
    for row in 0..al.len() {
        assert_eq!(al.get(row).unwrap(), bl.get(row).unwrap());
    }
    assert!(al.get(al.len()).is_none());
    assert!(bl.get(bl.len()).is_none());
    let (ap, bp) = (a.polylines(), b.polylines());
    assert_eq!(ap.len(), bp.len());
    assert_eq!(ap.is_empty(), bp.is_empty());
    for row in 0..ap.len() {
        let (a, b) = (ap.get(row).unwrap(), bp.get(row).unwrap());
        assert_eq!(a.entity(), b.entity());
        assert_eq!(a.closed(), b.closed());
        assert_eq!(a.vertex_count(), b.vertex_count());
        for vertex in 0..a.vertex_count() {
            assert_eq!(a.vertex(vertex).unwrap(), b.vertex(vertex).unwrap());
        }
        assert!(a.vertex(a.vertex_count()).is_none());
        assert!(b.vertex(b.vertex_count()).is_none());
    }
    assert!(ap.get(ap.len()).is_none());
    assert!(bp.get(bp.len()).is_none());
}
