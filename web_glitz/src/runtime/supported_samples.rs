use bitflags::bitflags;

bitflags! {
    /// Returned from [RenderingContext::supported_samples], describes which sampling grid sizes
    /// are available.
    ///
    /// See [RenderingContext::supported_samples] for details.
    pub struct SupportedSamples: u8 {
        const NONE = 0b00000000;
        const SAMPLES_2 = 0b00000010;
        const SAMPLES_4 = 0b00000100;
        const SAMPLES_8 = 0b00001000;
        const SAMPLES_16 = 0b00010000;
    }
}

impl SupportedSamples {
    /// The maximum sampling grid size that is supported.
    pub fn max_samples(&self) -> Option<u8> {
        if self.contains(SupportedSamples::SAMPLES_16) {
            Some(16)
        } else if self.contains(SupportedSamples::SAMPLES_8) {
            Some(8)
        } else if self.contains(SupportedSamples::SAMPLES_4) {
            Some(4)
        } else if self.contains(SupportedSamples::SAMPLES_2) {
            Some(2)
        } else {
            None
        }
    }
}

impl IntoIterator for SupportedSamples {
    type Item = u8;
    type IntoIter = SupportedSamplesIter;

    fn into_iter(self) -> Self::IntoIter {
        let current = self.max_samples().unwrap_or(0);

        SupportedSamplesIter {
            supported_samples: self,
            current,
        }
    }
}

/// Iterator over the available sampling grid sizes in a [SupportedSamples] value.
pub struct SupportedSamplesIter {
    supported_samples: SupportedSamples,
    current: u8,
}

impl Iterator for SupportedSamplesIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > 0 {
            let current = self.current;

            loop {
                self.current >>= 1;

                if self.current & self.supported_samples.bits() != 0 || self.current == 0 {
                    break;
                }
            }

            Some(current)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_samples() {
        let supported_samples = SupportedSamples::SAMPLES_8 | SupportedSamples::SAMPLES_4;

        assert_eq!(supported_samples.max_samples(), Some(8));
        assert_eq!(
            supported_samples.into_iter().collect::<Vec<_>>(),
            vec![8, 4]
        )
    }

    #[test]
    fn test_supported_samples_none() {
        let supported_samples = SupportedSamples::NONE;

        assert_eq!(supported_samples.max_samples(), None);
        assert!(supported_samples
            .into_iter()
            .collect::<Vec<u8>>()
            .is_empty())
    }
}
