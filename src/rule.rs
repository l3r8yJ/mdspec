use serde::{Serialize, Serializer};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub enum Rule {
    Mds001,
    Mds002,
    Mds003,
    Mds004,
    Mds005,
    Mds006,
    Mds007,
    Mds008,
    Mds009,
    Mds010,
    Mds011,
    Mds012,
    Mds013,
    Mds014,
    Mds015,
    Mds016,
    Mds017,
    Mds018,
    Mds019,
    Mds900,
    Mds901,
    Mds902,
}

impl Rule {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mds001 => "MDS001",
            Self::Mds002 => "MDS002",
            Self::Mds003 => "MDS003",
            Self::Mds004 => "MDS004",
            Self::Mds005 => "MDS005",
            Self::Mds006 => "MDS006",
            Self::Mds007 => "MDS007",
            Self::Mds008 => "MDS008",
            Self::Mds009 => "MDS009",
            Self::Mds010 => "MDS010",
            Self::Mds011 => "MDS011",
            Self::Mds012 => "MDS012",
            Self::Mds013 => "MDS013",
            Self::Mds014 => "MDS014",
            Self::Mds015 => "MDS015",
            Self::Mds016 => "MDS016",
            Self::Mds017 => "MDS017",
            Self::Mds018 => "MDS018",
            Self::Mds019 => "MDS019",
            Self::Mds900 => "MDS900",
            Self::Mds901 => "MDS901",
            Self::Mds902 => "MDS902",
        }
    }

    pub const fn is_operational(self) -> bool {
        matches!(self, Self::Mds900 | Self::Mds901 | Self::Mds902)
    }
}

impl Display for Rule {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for Rule {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
