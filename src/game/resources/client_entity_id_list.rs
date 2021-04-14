use legion::Entity;

#[derive(Clone, Copy)]
pub struct ZoneEntityId(pub u16);

#[derive(Default)]
pub struct ZoneEntityList {
    pub entity_list: Vec<Option<Entity>>,
    pub last_free_idx: Option<usize>,
}

impl ZoneEntityList {
    pub fn new() -> Self {
        Self {
            entity_list: vec![None; 4096],
            last_free_idx: Some(1),
        }
    }

    pub fn allocate(&mut self, entity: Entity) -> Option<ZoneEntityId> {
        if let Some(last_free_idx) = self.last_free_idx {
            let id = ZoneEntityId(last_free_idx as u16);
            self.entity_list[last_free_idx] = Some(entity);
            self.last_free_idx = self
                .entity_list
                .iter()
                .enumerate()
                .skip(last_free_idx)
                .find(|(index, entity)| entity.is_none())
                .map(|(index, _)| index);
            return Some(id);
        } else {
            None
        }
    }

    pub fn get_entity(&self, entity_id: ZoneEntityId) -> Option<Entity> {
        *self.entity_list.get(entity_id.0 as usize).unwrap_or(&None)
    }

    pub fn free(&mut self, entity_id: ZoneEntityId) {
        let index = entity_id.0 as usize;
        self.entity_list[index] = None;
        self.last_free_idx = self.last_free_idx.map_or(Some(index), |last_free_index| {
            Some(usize::min(index, last_free_index))
        });
    }
}

pub struct ClientEntityIdList {
    pub zones: Vec<ZoneEntityList>,
}

impl ClientEntityIdList {
    pub fn new() -> Self {
        let mut zones = Vec::new();
        for i in 0..100 {
            zones.push(ZoneEntityList::new());
        }
        Self { zones }
    }

    pub fn get_zone(&self, zone: usize) -> &ZoneEntityList {
        &self.zones[zone]
    }

    pub fn get_zone_mut(&mut self, zone: usize) -> &mut ZoneEntityList {
        &mut self.zones[zone]
    }
}
