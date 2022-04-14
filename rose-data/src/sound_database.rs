use serde::{Deserialize, Serialize};
use std::{num::NonZeroU16, str::FromStr};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct SoundId(NonZeroU16);

id_wrapper_impl!(SoundId, NonZeroU16, u16);
