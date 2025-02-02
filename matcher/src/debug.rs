use bimap::BiHashMap;

use crate::prototype::ProtoDatabase;
use std::{collections::HashMap, hash::Hash};

pub trait DebugWithName {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String;
}

macro_rules! impl_debug_with_name_display {
    ($($t:ty),*) => {
        $(
            impl DebugWithName for $t {
                fn debug_with_name(&self, _db: &ProtoDatabase) -> String {
                    format!("{}", self)
                }
            }
        )*
    }
}

impl_debug_with_name_display!(
    String, usize, u32, u64, i32, i64, f32, f64, bool
);

impl<K: DebugWithName + Eq + Hash, V: DebugWithName + Eq + Hash> DebugWithName for BiHashMap<K, V> {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        let mut formatter = "{".to_string();
        for (k, v) in self.iter() {
            formatter.push_str(&format!("{} <> {}, ", k.debug_with_name(db), v.debug_with_name(db)));
        }

        if formatter.len() > 1 {
            formatter.pop();
            formatter.pop();
        }

        formatter.push_str("}");

        formatter
    }
}

impl<V: DebugWithName> DebugWithName for Vec<V> {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        let mut formatter = "[".to_string();
        for v in self {
            formatter.push_str(&format!("{}, ", v.debug_with_name(db)));
        }

        if formatter.len() > 1 {
            formatter.pop();
            formatter.pop();
        }

        formatter.push_str("]");

        formatter
    }
}

impl<K: DebugWithName + Eq + Hash, V: DebugWithName + Eq + Hash> DebugWithName for HashMap<K, V> {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        let mut formatter = "{".to_string();
        for (k, v) in self.iter() {
            formatter.push_str(&format!("{}: {}, ", k.debug_with_name(db), v.debug_with_name(db)));
        }

        if formatter.len() > 1 {
            formatter.pop();
            formatter.pop();
        }

        formatter.push_str("}");

        formatter

    }
}
