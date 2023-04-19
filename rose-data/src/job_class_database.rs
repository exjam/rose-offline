use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU16, str::FromStr, sync::Arc};

use crate::StringDatabase;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct JobId(u16);

id_wrapper_impl!(JobId, u16);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct JobClassId(NonZeroU16);

id_wrapper_impl!(JobClassId, NonZeroU16, u16);

pub struct JobClassData {
    pub id: JobClassId,
    pub name: &'static str,
    pub jobs: ArrayVec<JobId, 8>,
}

pub struct JobClassDatabase {
    _string_database: Arc<StringDatabase>,
    job_classes: Vec<Option<JobClassData>>,
}

impl JobClassDatabase {
    pub fn new(
        string_database: Arc<StringDatabase>,
        job_classes: Vec<Option<JobClassData>>,
    ) -> Self {
        Self {
            _string_database: string_database,
            job_classes,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &JobClassData> {
        self.job_classes.iter().filter_map(|data| data.as_ref())
    }

    pub fn get(&self, id: JobClassId) -> Option<&JobClassData> {
        self.job_classes
            .get(id.get() as usize)
            .and_then(|x| x.as_ref())
    }
}
