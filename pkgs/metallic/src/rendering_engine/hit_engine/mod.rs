#[cfg(test)]
mod tests;

use hashbrown::HashSet;
use uuid::Uuid;
use winit::dpi::PhysicalPosition;

use crate::primitives::AbsPoint;

type IdsAndPoints = Vec<(Uuid, f32)>;

fn collect(slice: &[(Uuid, f32)]) -> HashSet<Uuid> {
    slice.into_iter().map(|&(id, _)| id).collect()
}

fn search_front(list: &IdsAndPoints, point: f32) -> HashSet<Uuid> {
    let index = match list.binary_search_by(|(_, el)| el.total_cmp(&point)) {
        Ok(index) => index + 1,
        Err(index) => index,
    };
    collect(&list.as_slice()[..index])
}

fn search_behind(list: &IdsAndPoints, point: f32) -> HashSet<Uuid> {
    let index = match list.binary_search_by(|(_, el)| el.total_cmp(&point)) {
        Ok(index) => index,
        Err(index) => index,
    };
    collect(&list.as_slice()[index..])
}

#[derive(Default, Clone, PartialEq)]
pub struct HitEngine {
    x_start: IdsAndPoints,
    x_end: IdsAndPoints,
    y_start: IdsAndPoints,
    y_end: IdsAndPoints,
}

impl HitEngine {
    pub fn insert(&mut self, bounding_box: BoundingBox) {
        let determine_insertion_point = |list: &IdsAndPoints, point: f32| -> usize {
            match list.binary_search_by(|(_, el)| el.total_cmp(&point)) {
                Ok(index) => index,
                Err(index) => index,
            }
        };
        let BoundingBox {
            id,
            tl: AbsPoint(PhysicalPosition { x: x1, y: y1 }),
            br: AbsPoint(PhysicalPosition { x: x2, y: y2 }),
        } = bounding_box;
        let x_start_index = determine_insertion_point(&self.x_start, x1);
        let x_end_index = determine_insertion_point(&self.x_end, x2);
        let y_start_index = determine_insertion_point(&self.y_start, y1);
        let y_end_index = determine_insertion_point(&self.y_end, y2);
        self.x_start.insert(x_start_index, (id, x1));
        self.x_end.insert(x_end_index, (id, x2));
        self.y_start.insert(y_start_index, (id, y1));
        self.y_end.insert(y_end_index, (id, y2));
    }

    pub fn hit_search(&self, abs_point: AbsPoint) -> HashSet<Uuid> {
        let AbsPoint(PhysicalPosition { x, y }) = abs_point;
        let x_start_hit_ids = search_front(&self.x_start, x);
        let x_end_hit_ids = search_behind(&self.x_end, x);
        let y_start_hit_ids = search_front(&self.y_start, y);
        let y_end_hit_ids = search_behind(&self.y_end, y);
        x_start_hit_ids
            .into_iter()
            .filter(|id| {
                x_end_hit_ids.contains(id)
                    && y_start_hit_ids.contains(id)
                    && y_end_hit_ids.contains(id)
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.x_start.clear();
        self.x_end.clear();
        self.y_start.clear();
        self.y_end.clear();
    }
}

pub struct BoundingBox {
    pub id: Uuid,
    pub tl: AbsPoint,
    pub br: AbsPoint,
}
