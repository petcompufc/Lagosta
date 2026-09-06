use std::{collections::HashMap, hash::Hash};

use godot::{
    meta::Element, obj::{Bounds, bounds}, prelude::*,
};
pub mod debug;
pub mod imgproc;
pub mod imgtools;

pub fn dict_to_hashmap<K, V>(dict: Dictionary<K, Gd<V>>) -> HashMap<K, V>
where
    K: Element + Hash + Eq,
    V: GodotClass + Bounds<Declarer = bounds::DeclUser> + Clone,
{
    dict.into_iter()
        .map(|t| (t.0, t.1.bind().clone()))
        .collect()
}

pub fn array_to_vec<V>(array: Array<Gd<V>>) -> Vec<V>
where
    V: GodotClass + Bounds<Declarer = bounds::DeclUser> + Clone,
{
    array.iter_shared().map(|t| t.bind().clone()).collect()
}
