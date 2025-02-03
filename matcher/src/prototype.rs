#![allow(dead_code)] // TODO: Remove this

use std::{collections::HashMap, fmt::{self, Debug}, hash::Hash, sync::RwLock};

use bimap::BiHashMap;
use matcher_macros::DebugWithName;

use crate::debug::DebugWithName;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ProtoName {
    id: usize,
}

impl DebugWithName for ProtoName {
    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
        format!("ProtoName({} => {})", self.id, db.identifier_db.get_by_right(&self.id).unwrap_or(&"ERROR".to_string()))
    }
}

impl ProtoName {
    pub fn lookup(db: &ProtoDatabase, name: &str) -> Self {
        Self { id: *db.identifier_db.get_by_left(name).unwrap() }
    }

    pub fn name(&self, db: &ProtoDatabase) -> String {
        db.identifier_db.get_by_right(&self.id).unwrap().clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, DebugWithName)]
pub struct ProtoMessage {
    pub name: ProtoName,
    pub fields: Vec<ProtoField>,
}

#[derive(Debug)]
pub enum ProtoResolutionError {
    TypeIsPrimitive,
    TargetAlreadyResolved,
    SourceNotResolved, // TODO: Probably shouldn't be an error
}

pub trait LogIfErr {
    fn log_if_err(&self);
}

impl <T> LogIfErr for Result<T, ProtoResolutionError> {
    fn log_if_err(&self) {
        if let Err(e) = self {
            println!("Warning: {:?}", e);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DebugWithName)]
pub struct ProtoField {
    pub name: ProtoName,
    pub field_type: ProtoFieldKind,
    pub field_number: u32,
}

impl ProtoField {
    pub fn try_resolve_in(&self, self_db: &ProtoDatabase, other_db: &mut ProtoDatabase, other: &ProtoField) -> Result<(), ProtoResolutionError> {
        // First try to resolve the field type
        match self.field_type.inner_type().try_resolve_in(self_db, other_db, &other.field_type.inner_type()) {
            Ok(_) => (),
            Err(ProtoResolutionError::TypeIsPrimitive) => (),
            Err(ProtoResolutionError::TargetAlreadyResolved) => (), // TODO: Verify that names match
            Err(e) => return Err(e),
        }

        // Ensure source field is marked as resolved
        if *self_db.identifier_resolutions.get(&self.name.id).unwrap_or(&false) == false {
            return Err(ProtoResolutionError::SourceNotResolved);
        }

        // Ensure target field is not marked as resolved
        if *other_db.identifier_resolutions.get(&other.name.id).unwrap_or(&false) {
            return Err(ProtoResolutionError::TargetAlreadyResolved);
        }

        // Update target field name
        println!("Resolving field {} -> {}", self.name.debug_with_name(self_db), other.name.debug_with_name(other_db));
        other_db.identifier_db.insert(self_db.identifier_db.get_by_right(&self.name.id).unwrap().clone(), other.name.id);

        // Mark the target field as resolved
        other_db.identifier_resolutions.insert(other.name.id, true);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DebugWithName)]

pub enum ProtoFieldKind {
    Scalar(ProtoType),
    Map(ProtoType, ProtoType),
    Repeated(ProtoType),
}

impl ProtoFieldKind {
    pub fn inner_type(&self) -> &ProtoType {
        match self {
            ProtoFieldKind::Scalar(a) => a,
            ProtoFieldKind::Map(_, b) => b,
            ProtoFieldKind::Repeated(a) => a,
        }
    }

    pub fn is_type_ref(&self) -> bool {
        matches!(self.inner_type(), ProtoType::Type(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DebugWithName, Ord, PartialOrd)]

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

impl ProtoType {
    pub fn try_resolve_in(&self, self_db: &ProtoDatabase, other_db: &mut ProtoDatabase, other: &ProtoType) -> Result<(), ProtoResolutionError> {
        match (self, other) {
            (ProtoType::Type(name), ProtoType::Type(other_name)) => {
                // Ensure the source type is marked as resolved
                if *self_db.identifier_resolutions.get(&name.id).unwrap_or(&false) == false {
                    return Err(ProtoResolutionError::SourceNotResolved);
                }

                // Ensure the target type is not marked as resolved
                if *other_db.identifier_resolutions.get(&other_name.id).unwrap_or(&false) {
                    return Err(ProtoResolutionError::TargetAlreadyResolved);
                }

                // Update target type name
                println!("Resolving type {} -> {}", name.debug_with_name(self_db), other_name.debug_with_name(other_db));
                other_db.identifier_db.insert(self_db.identifier_db.get_by_right(&name.id).unwrap().clone(), other_name.id);

                // Mark the target type as resolved
                other_db.identifier_resolutions.insert(other_name.id, true);

                return Ok(());
            }
            _ => Err(ProtoResolutionError::TypeIsPrimitive),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct WeakProtoType(ProtoType);

impl From<ProtoType> for WeakProtoType {
    fn from(value: ProtoType) -> Self {
        WeakProtoType(value)
    }
}

impl WeakProtoType {
    pub fn into_inner(self) -> ProtoType {
        self.0
    }
}

impl PartialEq for WeakProtoType {

    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (ProtoType::Type(_), ProtoType::Type(_)) => true, // We don't want to compare names

            (a, b) => a == b,
        }
    }
}

impl Hash for WeakProtoType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We only consider the unique variant for the hash
        std::mem::discriminant(&self.0).hash(state)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeakProtoFieldKind {
    Scalar(WeakProtoType),
    Map(WeakProtoType, WeakProtoType),
    Repeated(WeakProtoType),
}

impl WeakProtoFieldKind {    
    pub fn into_inner(self) -> ProtoFieldKind {
        match self {
            WeakProtoFieldKind::Scalar(a) => ProtoFieldKind::Scalar(a.into_inner()),
            WeakProtoFieldKind::Map(a, b) => ProtoFieldKind::Map(a.into_inner(), b.into_inner()),
            WeakProtoFieldKind::Repeated(a) => ProtoFieldKind::Repeated(a.into_inner()),
        }
    }
}

impl From<ProtoFieldKind> for WeakProtoFieldKind {
    fn from(value: ProtoFieldKind) -> Self {

        match value {
            ProtoFieldKind::Scalar(a) => WeakProtoFieldKind::Scalar(a.into()),
            ProtoFieldKind::Map(a, b) => WeakProtoFieldKind::Map(a.into(), b.into()),
            ProtoFieldKind::Repeated(a) => WeakProtoFieldKind::Repeated(a.into()),
        }
    }
}

// impl<'a> PartialEq for WeakProtoFieldKind<'a> {
//     fn eq(&self, other: &Self) -> bool {
//         match (&self.0, &other.0) {
//             (ProtoFieldKind::Scalar(a), ProtoFieldKind::Scalar(b)) => WeakProtoType(a) == WeakProtoType(b),
//             (ProtoFieldKind::Map(a, b), ProtoFieldKind::Map(c, d)) => WeakProtoType(a) == WeakProtoType(c) && WeakProtoType(b) == WeakProtoType(d),
//             (ProtoFieldKind::Repeated(a), ProtoFieldKind::Repeated(b)) => WeakProtoType(a) == WeakProtoType(b),
//             _ => false,
//         }
//     }
// }

// impl<'a> Hash for WeakProtoFieldKind<'a> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         match &self.0 {
//             ProtoFieldKind::Scalar(a) => WeakProtoType(a).hash(state),
//             ProtoFieldKind::Map(a, b) => {
//                 WeakProtoType(a).hash(state);
//                 WeakProtoType(b).hash(state);
//             }
//             ProtoFieldKind::Repeated(a) => WeakProtoType(a).hash(state),
//         }
//     }
// }

pub struct ProtoDatabase {
    pub identifier_counter: usize,
    pub identifier_db: BiHashMap<String, usize>,
    pub identifier_db_original: BiHashMap<String, usize>,
    pub identifier_resolutions: HashMap<usize, bool>,
    pub message_db: BiHashMap<ProtoName, ProtoMessage>,
}

impl Debug for ProtoDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProtoDatabase {{ identifier_counter: {}, identifier_db: {}, identifier_db_original: {}, message_db: {} }}", self.identifier_counter, self.identifier_db.len(), self.identifier_db_original.len(), self.message_db.len())
    }
}

impl ProtoDatabase {
    pub fn new() -> Self {
        Self {
            identifier_counter: 0,
            identifier_db: BiHashMap::new(),
            identifier_db_original: BiHashMap::new(),
            identifier_resolutions: HashMap::new(),
            message_db: BiHashMap::new(),
        }
    }

    fn guess_resolution(text: &str) -> bool {
        !(
            text.len() == 11 && 
            text.chars().all(|c| c.is_ascii_uppercase())
        )
    }

    pub fn register_identifier(&mut self, text: String) -> usize {
        if let Some(&id) = self.identifier_db.get_by_left(&text) {
            id
        } else {
            let id = self.identifier_counter;
            self.identifier_resolutions.insert(id, Self::guess_resolution(&text));
            self.identifier_db_original.insert(text.clone(), id);
            self.identifier_db.insert(text, id);
            self.identifier_counter += 1;
            id
        }
    }

    pub fn is_resolved(&self, proto_name: &ProtoName) -> bool {
        *self.identifier_resolutions.get(&proto_name.id).unwrap_or(&false)
    }

    pub fn lookup_name_by_text(&self, text: &str) -> ProtoName {
        ProtoName::lookup(&self, text)
    }

    pub fn register_message(&mut self, message: ProtoMessage) {
        self.message_db.insert(message.name.clone(), message);
    }

    pub fn get_message(&self, name: &str) -> Option<ProtoMessage> {
        self.message_db.get_by_left(&ProtoName::lookup(&self, name)).map(|m| m.clone())
    }

    pub fn generate_nametranslation(&self) -> HashMap<String, String> {
        self.identifier_db_original.iter().map(|(k, v)| (k.clone(), self.identifier_db.get_by_right(v).unwrap().clone())).collect()
    }
}

#[cfg(test)]


mod tests {
    use super::*;
    
    #[test]
    fn test_type_eq() {
        let map1 = ProtoFieldKind::Map(ProtoType::String, ProtoType::Uint32);
        let map2 = ProtoFieldKind::Map(ProtoType::String, ProtoType::Uint32);
        assert_eq!(map1, map2);
    }
}
