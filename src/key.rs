use anyhow::{anyhow, Result};
use serde_with::SerializeDisplay;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(SerializeDisplay, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Debug)]
pub enum RootNote {
    A,
    Bb,
    B,
    C,
    Db,
    D,
    Eb,
    E,
    F,
    Gb,
    G,
    Ab,
}

impl Display for RootNote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RootNote::A => write!(f, "A"),
            RootNote::Bb => write!(f, "A# / Bb"),
            RootNote::B => write!(f, "B"),
            RootNote::C => write!(f, "C"),
            RootNote::Db => write!(f, "C# / Db"),
            RootNote::D => write!(f, "D"),
            RootNote::Eb => write!(f, "D# / Eb"),
            RootNote::E => write!(f, "E"),
            RootNote::F => write!(f, "F"),
            RootNote::Gb => write!(f, "F# / Gb"),
            RootNote::G => write!(f, "G"),
            RootNote::Ab => write!(f, "G# / Ab"),
        }
    }
}

impl FromStr for RootNote {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "A" => Ok(RootNote::A),
            "A♯" => Ok(RootNote::Bb),
            "B♭" => Ok(RootNote::Bb),
            "B" => Ok(RootNote::B),
            "C" => Ok(RootNote::C),
            "C♯" => Ok(RootNote::Db),
            "D♭" => Ok(RootNote::Db),
            "D" => Ok(RootNote::D),
            "D♯" => Ok(RootNote::Eb),
            "E♭" => Ok(RootNote::Eb),
            "E" => Ok(RootNote::E),
            "F" => Ok(RootNote::F),
            "F♯" => Ok(RootNote::Gb),
            "G♭" => Ok(RootNote::Gb),
            "G" => Ok(RootNote::G),
            "G♯" => Ok(RootNote::Ab),
            "A♭" => Ok(RootNote::Ab),
            _ => Err(anyhow!("Invalid root note")),
        }
    }
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Debug)]
pub enum Scale {
    Minor,
    Major,
}

impl FromStr for Scale {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "min" => Ok(Scale::Minor),
            "maj" => Ok(Scale::Major),
            _ => Err(anyhow!("Invalid scale")),
        }
    }
}

#[derive(SerializeDisplay, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Debug)]
pub struct Key {
    pub root_note: RootNote,
    pub scale: Scale,
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Regarding to the Circle of Fifths
        match self.scale {
            Scale::Minor => match self.root_note {
                RootNote::A => write!(f, "A min"),
                RootNote::Bb => write!(f, "Bb min"),
                RootNote::B => write!(f, "B min"),
                RootNote::C => write!(f, "C min"),
                RootNote::Db => write!(f, "C# min"),
                RootNote::D => write!(f, "D min"),
                RootNote::Eb => write!(f, "Eb min"),
                RootNote::E => write!(f, "E min"),
                RootNote::F => write!(f, "F min"),
                RootNote::Gb => write!(f, "F# min"),
                RootNote::G => write!(f, "G min"),
                RootNote::Ab => write!(f, "G# min"),
            },
            Scale::Major => match self.root_note {
                RootNote::A => write!(f, "A maj"),
                RootNote::Bb => write!(f, "Bb maj"),
                RootNote::B => write!(f, "B maj"),
                RootNote::C => write!(f, "C maj"),
                RootNote::Db => write!(f, "Db maj"),
                RootNote::D => write!(f, "D maj"),
                RootNote::Eb => write!(f, "Eb maj"),
                RootNote::E => write!(f, "E maj"),
                RootNote::F => write!(f, "F maj"),
                RootNote::Gb => write!(f, "F# maj"),
                RootNote::G => write!(f, "G maj"),
                RootNote::Ab => write!(f, "Ab maj"),
            },
        }
    }
}

impl FromStr for Key {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        let mut value_iter = value.split_ascii_whitespace();

        match (value_iter.next(), value_iter.next()) {
            (Some(root_note_str), Some(scale_str)) => {
                let root_note = RootNote::from_str(root_note_str)?;
                let scale = Scale::from_str(scale_str)?;
                Ok(Self { root_note, scale })
            }
            _ => Err(anyhow!("Invalid key")),
        }
    }
}
