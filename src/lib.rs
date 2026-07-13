#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Volume(u8),
    Mute(bool),
}

impl Event {
    pub const SERIALIZED_LENGTH: usize = 2;

    pub fn serialize(self) -> [u8; Self::SERIALIZED_LENGTH] {
        match self {
            Self::Volume(volume) => [1, volume],
            Self::Mute(muted) => [2, muted as u8],
        }
    }

    pub fn deserialize(buf: [u8; Self::SERIALIZED_LENGTH]) -> Self {
        match buf[0] {
            1 => {
                let volume = buf[1];
                Self::Volume(volume)
            }
            2 => {
                let muted = bool::try_from(buf[1]).unwrap();
                Self::Mute(muted)
            }

            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Event;

    #[test]
    fn test_volume() {
        let e = Event::Volume(42);
        assert_eq!(Event::deserialize(e.serialize()), e);
    }

    #[test]
    fn test_muted() {
        let e = Event::Mute(true);
        assert_eq!(Event::deserialize(e.serialize()), e);

        let e = Event::Mute(false);
        assert_eq!(Event::deserialize(e.serialize()), e);
    }
}
