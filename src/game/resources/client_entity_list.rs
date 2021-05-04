use legion::Entity;
use nalgebra::{Point2, Point3, Vector2};
use std::collections::HashSet;

use crate::game::{
    components::ClientEntity,
    data::{zone::ZoneInfo, ZONE_LIST},
};

const MAX_CLIENT_ENTITY_ID: usize = 4096;

#[derive(Clone, Copy)]
pub struct ClientEntityId(pub u16);

#[derive(Clone, Default)]
pub struct ClientEntitySector {
    pub entities: HashSet<Entity>,
}

impl ClientEntitySector {
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    pub fn remove_entity(&mut self, entity: &Entity) {
        assert_eq!(self.entities.remove(entity), true);
    }
}

pub struct ClientEntityZone {
    pub sector_limit_squared: f32,
    pub zone_info: &'static ZoneInfo,
    pub entity_list_by_id: Vec<Option<Entity>>,
    pub last_free_entity_index: Option<usize>,
    pub sectors: Vec<ClientEntitySector>,
}

fn calculate_sector(zone_info: &'static ZoneInfo, position: Point3<f32>) -> Point2<u32> {
    let sector_size = zone_info.sector_size as f32;
    let sector = (position.xy() - zone_info.sectors_base_position) / (sector_size as f32);
    Point2::new(
        u32::min(
            i32::max(0i32, sector[0] as i32) as u32,
            zone_info.num_sectors_x - 1,
        ),
        u32::min(
            i32::max(0i32, sector[1] as i32) as u32,
            zone_info.num_sectors_y - 1,
        ),
    )
}

fn calculate_sector_midpoint(zone_info: &'static ZoneInfo, sector: Point2<u32>) -> Point2<f32> {
    let sector_size = zone_info.sector_size as f32;
    zone_info.sectors_base_position
        + Vector2::new(sector[0] as f32 + 0.5, sector[1] as f32 + 0.5) * sector_size
}

impl ClientEntityZone {
    pub fn new(zone_info: &'static ZoneInfo) -> Self {
        let sector_size = zone_info.sector_size as f32;
        let sector_limit = (sector_size / 2.0) + (sector_size * 0.2);
        Self {
            zone_info: zone_info,
            entity_list_by_id: vec![None; MAX_CLIENT_ENTITY_ID],
            last_free_entity_index: Some(1),
            sector_limit_squared: sector_limit * sector_limit,
            sectors: vec![
                Default::default();
                (zone_info.num_sectors_x * zone_info.num_sectors_y) as usize
            ],
        }
    }

    pub fn allocate(&mut self, entity: Entity, position: Point3<f32>) -> Option<ClientEntity> {
        if let Some(last_free_entity_index) = self.last_free_entity_index {
            let id = ClientEntityId(last_free_entity_index as u16);
            self.entity_list_by_id[last_free_entity_index] = Some(entity);
            self.last_free_entity_index = self
                .entity_list_by_id
                .iter()
                .enumerate()
                .skip(last_free_entity_index)
                .find(|(_, entity)| entity.is_none())
                .map(|(index, _)| index);

            let sector = calculate_sector(self.zone_info, position);
            self.get_sector_mut(sector).add_entity(entity);
            return Some(ClientEntity::new(id, sector));
        } else {
            None
        }
    }

    pub fn get_sector(&self, sector: Point2<u32>) -> &ClientEntitySector {
        &self.sectors[sector[0] as usize + (sector[1] * self.zone_info.num_sectors_x) as usize]
    }

    pub fn get_sector_mut(&mut self, sector: Point2<u32>) -> &mut ClientEntitySector {
        &mut self.sectors[sector[0] as usize + (sector[1] * self.zone_info.num_sectors_x) as usize]
    }

    pub fn update_sector(
        &mut self,
        entity: &Entity,
        client_entity: &mut ClientEntity,
        position: Point3<f32>,
    ) {
        let midpoint = calculate_sector_midpoint(self.zone_info, client_entity.sector);
        if (midpoint - position.xy()).magnitude_squared() > self.sector_limit_squared {
            let previous_sector = client_entity.sector;
            let new_sector = calculate_sector(self.zone_info, position);
            self.get_sector_mut(previous_sector).remove_entity(entity);
            self.get_sector_mut(new_sector).add_entity(*entity);
            client_entity.sector = new_sector;
        }
    }

    pub fn get_entity(&self, entity_id: ClientEntityId) -> Option<Entity> {
        *self
            .entity_list_by_id
            .get(entity_id.0 as usize)
            .unwrap_or(&None)
    }

    pub fn free(&mut self, entity_id: ClientEntityId) {
        let index = entity_id.0 as usize;
        self.entity_list_by_id[index] = None;
        self.last_free_entity_index = self
            .last_free_entity_index
            .map_or(Some(index), |last_free_index| {
                Some(usize::min(index, last_free_index))
            });
    }
}

pub struct ClientEntityList {
    pub zones: Vec<Option<ClientEntityZone>>,
}

impl ClientEntityList {
    pub fn new() -> Self {
        let mut zones = Vec::new();
        for zone in ZONE_LIST.zones.iter() {
            if let Some(zone) = zone.as_ref() {
                zones.push(Some(ClientEntityZone::new(zone)));
            } else {
                zones.push(None);
            }
        }
        Self { zones }
    }

    pub fn get_zone(&self, zone: usize) -> Option<&ClientEntityZone> {
        self.zones[zone].as_ref()
    }

    pub fn get_zone_mut(&mut self, zone: usize) -> Option<&mut ClientEntityZone> {
        self.zones[zone].as_mut()
    }
}
