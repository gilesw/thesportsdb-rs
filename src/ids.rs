use std::fmt;
use std::str::FromStr;

/// TheSportsDB league id, the `idLeague` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LeagueId(pub i64);

/// TheSportsDB event id, the `idEvent` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventId(pub i64);

macro_rules! id_impls {
    ($name:ident) => {
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = std::num::ParseIntError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.trim().parse().map(Self)
            }
        }

        impl From<$name> for i64 {
            fn from(id: $name) -> i64 {
                id.0
            }
        }
    };
}

id_impls!(LeagueId);
id_impls!(EventId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_parse_and_display() {
        assert_eq!("4443".parse::<LeagueId>(), Ok(LeagueId(4443)));
        assert_eq!(" 12 ".parse::<EventId>(), Ok(EventId(12)));
        assert!("ufc".parse::<LeagueId>().is_err());
        assert_eq!(LeagueId(4443).to_string(), "4443");
    }
}
