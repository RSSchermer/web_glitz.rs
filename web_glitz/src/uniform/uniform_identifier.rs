use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use fnv::FnvHasher;

#[derive(Clone, PartialEq)]
pub struct UniformValueIdentifier {
    segments: Vec<ValueIdentifierSegment>,
}

impl UniformValueIdentifier {
    pub fn from_string(string: &str) -> Self {
        let mut segments = Vec::new();

        for s in string.split(".") {
            let parts = s.split("[").collect::<Vec<_>>();
            let name = parts[0].to_string();

            segments.push(ValueIdentifierSegment::Name(NameSegment::new(name)));

            if parts.len() == 2 {
                let index = parts[1].trim_right_matches("]").parse::<usize>().unwrap();

                segments.push(ValueIdentifierSegment::ArrayIndex(index));
            }
        }

        UniformValueIdentifier {
            segments
        }
    }

    pub fn is_array_identifier(&self) -> bool {
        if let Some(segment) = self.segments.last() {
            segment.is_array_index()
        } else {
            false
        }
    }

    pub fn head(&self) -> Option<&ValueIdentifierSegment> {
        self.segments.get(0)
    }

    pub fn tail(&self) -> ValueIdentifierTail {
        ValueIdentifierTail {
            identifier: self,
            head_index: 1
        }
    }
}

impl Deref for UniformValueIdentifier {
    type Target = [ValueIdentifierSegment];

    fn deref(&self) -> &Self::Target {
        &self.segments
    }
}

impl Display for UniformValueIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut segments_iter = self.segments.iter();

        if let Some(segment) = segments_iter.next() {
            write!(f, "{}", segment.to_string())?;
        }

        while let Some(segment) = segments_iter.next() {
            match segment {
                ValueIdentifierSegment::Name(segment) => {
                    write!(f, ".{}", segment.name())?;
                },
                ValueIdentifierSegment::ArrayIndex(index) => {
                    write!(f, "[{}]", index)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum ValueIdentifierSegment {
    Name(NameSegment),
    ArrayIndex(usize),
}

impl ValueIdentifierSegment {
    pub fn is_array_index(&self) -> bool {
        if let ValueIdentifierSegment::ArrayIndex(_) = self {
            true
        } else {
            false
        }
    }
}

impl Into<UniformValueIdentifier> for ValueIdentifierSegment {
    fn into(self) -> UniformValueIdentifier {
        UniformValueIdentifier {
            segments: vec![self],
        }
    }
}

impl Display for ValueIdentifierSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueIdentifierSegment::Name(segment) => write!(f, "{}", segment.name),
            ValueIdentifierSegment::ArrayIndex(index) => write!(f, "[{}]", index)

        }
    }
}

#[derive(Clone)]
pub struct ValueIdentifierTail<'a> {
    identifier: &'a UniformValueIdentifier,
    head_index: usize
}

impl<'a> ValueIdentifierTail<'a> {
    pub fn root(&self) -> &UniformValueIdentifier {
        self.identifier
    }

    pub fn head(&self) -> Option<&ValueIdentifierSegment> {
        self.identifier.segments.get(self.head_index)
    }

    pub fn tail(&self) -> ValueIdentifierTail {
        ValueIdentifierTail {
            identifier: self.identifier,
            head_index: self.head_index + 1
        }
    }
}

impl<'a> Deref for ValueIdentifierTail<'a> {
    type Target = [ValueIdentifierSegment];

    fn deref(&self) -> &Self::Target {
        let head_index = self.head_index;
        let segments = &self.identifier.segments;

        if head_index >= segments.len() {
            &segments[0..0]
        } else {
            &segments[head_index..]
        }
    }
}

#[derive(Clone)]
pub struct NameSegment {
    name: String,
    name_hash_fnv64: u64
}

impl NameSegment {
    fn new(name: String) -> Self {
        let mut hasher = FnvHasher::default();

        name.hash(&mut hasher);

        let name_hash_fnv64 = hasher.finish();

        NameSegment {
            name,
            name_hash_fnv64
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_hash_fnv64(&self) -> u64 {
        self.name_hash_fnv64
    }
}

impl PartialEq for NameSegment {
    fn eq(&self, other: &Self) -> bool {
        self.name_hash_fnv64 == other.name_hash_fnv64
    }
}

pub struct UniformBlockIdentifier {
    name: String,
    name_hash_fnv64: u64
}

impl UniformBlockIdentifier {
    fn new(name: String) -> Self {
        let mut hasher = FnvHasher::default();

        name.hash(&mut hasher);

        let name_hash_fnv64 = hasher.finish();

        UniformBlockIdentifier {
            name,
            name_hash_fnv64
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_hash_fnv64(&self) -> u64 {
        self.name_hash_fnv64
    }
}

impl PartialEq for UniformBlockIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.name_hash_fnv64 == other.name_hash_fnv64
    }
}
