use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use fnv::FnvHasher;

#[derive(Clone, PartialEq)]
pub struct UniformIdentifier {
    segments: Vec<IdentifierSegment>,
}

impl UniformIdentifier {
    pub fn from_string(string: &str) -> Self {
        let mut segments = Vec::new();

        for s in string.split(".") {
            let parts = s.split("[").collect::<Vec<_>>();
            let name = parts[0].to_string();

            segments.push(IdentifierSegment::Name(NameSegment::new(name)));

            if parts.len() == 2 {
                let index = parts[1].trim_right_matches("]").parse::<usize>().unwrap();

                segments.push(IdentifierSegment::ArrayIndex(index));
            }
        }

        UniformIdentifier { segments }
    }

    pub fn is_array_identifier(&self) -> bool {
        if let Some(segment) = self.segments.last() {
            segment.is_array_index()
        } else {
            false
        }
    }

    pub fn head(&self) -> Option<&IdentifierSegment> {
        self.segments.get(0)
    }

    pub fn tail(&self) -> IdentifierTail {
        IdentifierTail {
            identifier: self,
            head_index: 1,
        }
    }

    pub fn as_tail(&self) -> IdentifierTail {
        IdentifierTail {
            identifier: self,
            head_index: 0
        }
    }
}

impl Deref for UniformIdentifier {
    type Target = [IdentifierSegment];

    fn deref(&self) -> &Self::Target {
        &self.segments
    }
}

impl Display for UniformIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut segments_iter = self.segments.iter();

        if let Some(segment) = segments_iter.next() {
            write!(f, "{}", segment.to_string())?;
        }

        while let Some(segment) = segments_iter.next() {
            match segment {
                IdentifierSegment::Name(segment) => {
                    write!(f, ".{}", segment.name())?;
                }
                IdentifierSegment::ArrayIndex(index) => {
                    write!(f, "[{}]", index)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum IdentifierSegment {
    Name(NameSegment),
    ArrayIndex(usize),
}

impl IdentifierSegment {
    pub fn is_array_index(&self) -> bool {
        if let IdentifierSegment::ArrayIndex(_) = self {
            true
        } else {
            false
        }
    }
}

impl Into<UniformIdentifier> for IdentifierSegment {
    fn into(self) -> UniformIdentifier {
        UniformIdentifier {
            segments: vec![self],
        }
    }
}

impl Display for IdentifierSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IdentifierSegment::Name(segment) => write!(f, "{}", segment.name),
            IdentifierSegment::ArrayIndex(index) => write!(f, "[{}]", index),
        }
    }
}

#[derive(Clone)]
pub struct IdentifierTail<'a> {
    identifier: &'a UniformIdentifier,
    head_index: usize,
}

impl<'a> IdentifierTail<'a> {
    pub fn root(&self) -> &UniformIdentifier {
        self.identifier
    }

    pub fn head(&self) -> Option<&IdentifierSegment> {
        self.identifier.segments.get(self.head_index)
    }

    pub fn tail(&self) -> IdentifierTail {
        IdentifierTail {
            identifier: self.identifier,
            head_index: self.head_index + 1,
        }
    }

    pub fn is_array_terminus(&self) -> bool {
        self.head() == Some(&IdentifierSegment::ArrayIndex(0)) && self.tail().is_empty()
    }
}

impl<'a> Deref for IdentifierTail<'a> {
    type Target = [IdentifierSegment];

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
    name_hash_fnv64: u64,
}

impl NameSegment {
    fn new(name: String) -> Self {
        let mut hasher = FnvHasher::default();

        name.hash(&mut hasher);

        let name_hash_fnv64 = hasher.finish();

        NameSegment {
            name,
            name_hash_fnv64,
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
