#[derive(PartialEq, Hash)]
pub struct UniformIdentifier {
    segments: Vec<UniformIdentifierSegment>,
}

impl UniformIdentifier {
    pub fn from_string(string: &str) -> Self {
        UniformIdentifier {
            segments: string
                .split(".")
                .map(|s| UniformIdentifierSegment::from_string(s))
                .collect(),
        }
    }

    pub fn is_array_identifier(&self) -> bool {
        if let Some(segment) = self.segments.last() {
            segment.is_array_identifier()
        } else {
            false
        }
    }
}

impl Display for UniformIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.segments
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

#[derive(Clone, PartialEq, Hash)]
pub enum UniformIdentifierSegment {
    Simple(String),
    ArrayElement(String, u32),
}

impl UniformIdentifierSegment {
    pub fn from_string(string: &str) -> Self {
        let parts = string.split("[").collect::<Vec<_>>();

        if parts.len() == 1 {
            UniformIdentifierSegment::Simple(parts[0].to_string())
        } else {
            let index = parts[1].trim_right_matches("]").parse::<u32>().unwrap();

            UniformIdentifierSegment::ArrayElement(parts[0].to_string(), index)
        }
    }

    pub fn is_array_identifier(&self) -> bool {
        if let UniformIdentifierSegment::ArrayElement(_, _) = self {
            true
        } else {
            false
        }
    }
}

impl Into<UniformIdentifier> for UniformIdentifierSegment {
    fn into(self) -> UniformIdentifier {
        UniformIdentifier {
            segments: vec![self],
        }
    }
}

impl Display for UniformIdentifierSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UniformIdentifierSegment::Simple(name) => write!(f, "{}", name),
            UniformIdentifierSegment::ArrayElement(array_name, index) => {
                write!(f, "{}[{}]", array_name, index)
            }
        }
    }
}
