use std::collections::BTreeMap;

use crate::primitives::Object;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(usize, usize);

#[derive(Default)]
pub struct ObjectEngine {
    pub(crate) object_matrix: BTreeMap<usize, Vec<Object>>,
}

pub fn clear(object_engine: &mut ObjectEngine) {
    object_engine.object_matrix.clear();
}

pub fn remove_layer(object_engine: &mut ObjectEngine, layer: usize) -> Option<Vec<Object>> {
    object_engine.object_matrix.remove(&layer)
}

pub fn remove_object(object_engine: &mut ObjectEngine, key: Key) -> Option<Object> {
    let object_row = object_engine.object_matrix.get_mut(&key.0)?;
    let object = object_row.remove(key.1);
    Some(object)
}

pub fn add_object(object_engine: &mut ObjectEngine, layer: usize, object: Object) -> Key {
    let object_row = object_engine
        .object_matrix
        .entry(layer)
        .or_insert_with(Vec::default);
    let index = object_row.len();
    object_row.push(object);
    Key(layer, index)
}

pub fn get_object(object_engine: &ObjectEngine, key: Key) -> Option<&Object> {
    let object_row = object_engine.object_matrix.get(&key.0)?;
    let object = object_row.get(key.1)?;
    Some(object)
}

pub fn get_object_mut(object_engine: &mut ObjectEngine, key: Key) -> Option<&mut Object> {
    let object_row = object_engine.object_matrix.get_mut(&key.0)?;
    let object = object_row.get_mut(key.1)?;
    Some(object)
}
