use bimap::BiHashMap;

use crate::prototype::{ProtoDatabase, ProtoField, ProtoFieldKind, ProtoMessage, ProtoType};
use std::hash::Hash;

pub trait DebugWithName {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String;
}

impl DebugWithName for String {
    fn debug_with_name(&self, _db: &ProtoDatabase) -> String {
        format!("{}", self)
    }
}

impl DebugWithName for usize {
    fn debug_with_name(&self, _db: &ProtoDatabase) -> String {
        format!("{}", self)
    }
}

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


// My types below

impl DebugWithName for ProtoMessage {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        format!("ProtoMessage {{ name: {}, fields: {} }}", self.name.debug_with_name(db), self.fields.debug_with_name(db))
    }
}

impl DebugWithName for ProtoField {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        format!("ProtoField {{ name: {}, field_type: {}, field_number: {} }}", self.name.debug_with_name(db), self.field_type.debug_with_name(db), self.field_number)
    }
}

impl DebugWithName for ProtoFieldKind {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        match self {
            ProtoFieldKind::Scalar(t) => format!("Scalar({})", t.debug_with_name(db)),
            ProtoFieldKind::Map(k, v) => format!("Map({}, {})", k.debug_with_name(db), v.debug_with_name(db)),
            ProtoFieldKind::Repeated(t) => format!("Repeated({})", t.debug_with_name(db)),
        }
    }
}

impl DebugWithName for ProtoType {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        match self {
            ProtoType::Type(n) => format!("Type({})", n.debug_with_name(db)),
            _ => format!("{:?}", self),
        }
    }

}
