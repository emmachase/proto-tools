#![allow(dead_code)] // TODO: Remove this

use std::fmt;

use bimap::BiHashMap;

#[derive(Clone, PartialEq, Eq)]
pub struct ProtoName<'db> {
    id: usize,
    db: &'db BiHashMap<String, usize>,
}

impl<'db> ProtoName<'db> {
    pub fn lookup(identifier_db: &'db BiHashMap<String, usize>, name: &str) -> Self {
        Self { id: *identifier_db.get_by_left(name).unwrap(), db: identifier_db }
    }

    pub fn name(&self) -> &str {
        self.db.get_by_right(&self.id).unwrap()
    }
}

impl<'db> fmt::Debug for ProtoName<'db> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProtoName({} => {})", self.id, self.name())
    }
}

impl<'db> fmt::Display for ProtoName<'db> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtoMessage<'db> {
    pub name: ProtoName<'db>,
    pub fields: Vec<ProtoField<'db>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtoField<'db> {
    pub name: ProtoName<'db>,
    pub field_type: ProtoFieldKind<'db>,
    pub field_number: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtoFieldKind<'db> {
    Scalar(ProtoType<'db>),
    Map(Box<ProtoType<'db>>, Box<ProtoType<'db>>),
    Repeated(Box<ProtoType<'db>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtoType<'db> {
    Bool,
    Float,
    Double,
    Int32,
    Int64,
    Uint32,
    Uint64,

    Sint32,
    Sint64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    String,
    Bytes,
    Type(ProtoName<'db>), // message, enum, etc.
    // Map(Box<ProtoType<'db>>, Box<ProtoType<'db>>),
}

// impl<'db> ProtoField<'db> {
//     pub fn has_same_type(&self, other: &ProtoField<'db>) -> bool {
//         self.field_type == other.field_type && self.repeated == other.repeated
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_eq() {
        let map1 = ProtoFieldKind::Map(Box::new(ProtoType::String), Box::new(ProtoType::Uint32));
        let map2 = ProtoFieldKind::Map(Box::new(ProtoType::String), Box::new(ProtoType::Uint32));
        assert_eq!(map1, map2);
    }

}
