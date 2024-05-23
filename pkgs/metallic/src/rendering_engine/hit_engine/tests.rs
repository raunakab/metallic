use super::*;

#[test]
fn test_empty_hit_engine() {
    let hit_engine = HitEngine::default();
    let hit_ids = hit_engine.hit_search(AbsPoint(PhysicalPosition { x: 0.0, y: 0.0 }));
    assert!(hit_ids.is_empty());
}

#[test]
fn test_simple() {
    let mut hit_engine = HitEngine::default();
    let id = Uuid::new_v4();
    hit_engine.insert(BoundingBox {
        id,
        tl: AbsPoint(PhysicalPosition { x: 1.0, y: 1.0 }),
        br: AbsPoint(PhysicalPosition { x: 10.0, y: 10.0 }),
    });
    let hit_ids = hit_engine.hit_search(AbsPoint(PhysicalPosition { x: 0.0, y: 0.0 }));
    assert!(hit_ids.is_empty());
    let hit_ids = hit_engine.hit_search(AbsPoint(PhysicalPosition { x: 1.0, y: 1.0 }));
    assert_eq!(hit_ids, [id].into_iter().collect());
}

#[test]
fn test_multiple_disjoint_boxes() {
    let mut hit_engine = HitEngine::default();
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    hit_engine.insert(BoundingBox {
        id: id1,
        tl: AbsPoint(PhysicalPosition { x: 1.0, y: 1.0 }),
        br: AbsPoint(PhysicalPosition { x: 2.0, y: 2.0 }),
    });
    hit_engine.insert(BoundingBox {
        id: id2,
        tl: AbsPoint(PhysicalPosition { x: 3.0, y: 3.0 }),
        br: AbsPoint(PhysicalPosition { x: 4.0, y: 4.0 }),
    });
    let hit_ids = hit_engine.hit_search(AbsPoint(PhysicalPosition { x: 0.0, y: 0.0 }));
    assert!(hit_ids.is_empty());
    let hit_ids = hit_engine.hit_search(AbsPoint(PhysicalPosition { x: 1.0, y: 1.0 }));
    assert_eq!(hit_ids, [id1].into_iter().collect());
    let hit_ids = hit_engine.hit_search(AbsPoint(PhysicalPosition { x: 3.0, y: 3.0 }));
    assert_eq!(hit_ids, [id2].into_iter().collect());
}

#[test]
fn test_multiple_overlapping_boxes() {
    let mut hit_engine = HitEngine::default();
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    hit_engine.insert(BoundingBox {
        id: id1,
        tl: AbsPoint(PhysicalPosition { x: 1.0, y: 1.0 }),
        br: AbsPoint(PhysicalPosition { x: 3.0, y: 3.0 }),
    });
    hit_engine.insert(BoundingBox {
        id: id2,
        tl: AbsPoint(PhysicalPosition { x: 2.0, y: 2.0 }),
        br: AbsPoint(PhysicalPosition { x: 4.0, y: 4.0 }),
    });
    let hit_ids = hit_engine.hit_search(AbsPoint(PhysicalPosition { x: 2.5, y: 2.5 }));
    assert_eq!(hit_ids, [id1, id2].into_iter().collect());
}
