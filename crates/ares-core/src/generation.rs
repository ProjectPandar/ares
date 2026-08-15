use crate::SliceError;

pub const ORCA_SLICER_COMPATIBILITY_VERSION: &str = "2.4.2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenerationMetadata {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl GenerationMetadata {
    pub(crate) const fn timestamp(self) -> (u16, u8, u8, u8, u8, u8) {
        (
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the approved public API names all six local calendar fields"
    )]
    pub fn new_local(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, SliceError> {
        let max_day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        };
        if year == 0 || day == 0 || day > max_day || hour > 23 || minute > 59 || second > 59 {
            return Err(SliceError::InvalidInput(
                "invalid local generation timestamp".to_owned(),
            ));
        }
        Ok(Self::deterministic(year, month, day, hour, minute, second))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the approved deterministic API mirrors the local constructor"
    )]
    pub const fn deterministic(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

const fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
