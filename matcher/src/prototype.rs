#![allow(dead_code)] // TODO: Remove this

use std::{fmt::{self, Debug}, hash::Hash};

use bimap::BiHashMap;

use crate::debug::DebugWithName;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtoName {
    id: usize,
}

impl DebugWithName for ProtoName {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        format!("ProtoName({} => {})", self.id, db.identifier_db.get_by_right(&self.id).unwrap_or(&"ERROR".to_string()))
    }
}

impl ProtoName {
    pub fn lookup(identifier_db: &BiHashMap<String, usize>, name: &str) -> Self {
        Self { id: *identifier_db.get_by_left(name).unwrap() }
    }

    pub fn name<'a>(&self, db: &'a BiHashMap<String, usize>) -> &'a str {
        db.get_by_right(&self.id).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtoMessage {
    pub name: ProtoName,
    pub fields: Vec<ProtoField>,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtoField {
    pub name: ProtoName,
    pub field_type: ProtoFieldKind,
    pub field_number: u32,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum ProtoFieldKind {
    Scalar(ProtoType),
    Map(Box<ProtoType>, Box<ProtoType>),
    Repeated(Box<ProtoType>),
}



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtoType {
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
    Type(ProtoName),
}



pub struct ProtoDatabase {
    pub identifier_counter: usize,
    pub identifier_db: BiHashMap<String, usize>,
    pub message_db: BiHashMap<ProtoName, ProtoMessage>,
}


impl Debug for ProtoDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProtoDatabase {{ identifier_counter: {}, identifier_db: {}, message_db: {} }}", self.identifier_counter, self.identifier_db.debug_with_name(self), self.message_db.debug_with_name(self))
    }
}

impl ProtoDatabase {
    pub fn new() -> Self {

        Self {
            identifier_counter: 0,
            identifier_db: BiHashMap::new(),
            message_db: BiHashMap::new(),
        }
    }

    pub fn register_identifier(&mut self, text: String) -> usize {
        if let Some(&id) = self.identifier_db.get_by_left(&text) {
            id
        } else {
            let id = self.identifier_counter;
            self.identifier_db.insert(text, id);
            self.identifier_counter += 1;
            id
        }
    }

    pub fn lookup_name(&self, text: &str) -> ProtoName {
        ProtoName::lookup(&self.identifier_db, text)
    }

    pub fn register_message(&mut self, message: ProtoMessage) {
        self.message_db.insert(message.name.clone(), message);
    }
}

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
