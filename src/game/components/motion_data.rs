use crate::data::MotionFileData;

#[derive(Default)]
pub struct MotionData {
    pub attack: Option<MotionFileData>,
    pub die: Option<MotionFileData>,
    pub pickup_dropped_item: Option<MotionFileData>,
}
