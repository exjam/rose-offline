use legion::Entity;
use nalgebra::{Point2, Point3, Vector2};
use std::collections::{HashMap, HashSet};

use crate::{
    data::{ZoneData, ZoneDatabase},
    game::components::ClientEntity,
};

const MAX_CLIENT_ENTITY_ID: usize = 4096;

#[derive(Clone, Copy)]
pub struct ClientEntityId(pub u16);

#[derive(Clone, Default)]
pub struct ClientEntitySector {
    // The list of entities currently inside this sector
    entities: HashMap<Entity, Point3<f32>>,

    // The list of entities visible from this sector, this is a union of the entities of all adjacent sectors
    visible_entities: HashSet<Entity>,
}

impl ClientEntitySector {
    pub fn get_visible_entities(&self) -> &HashSet<Entity> {
        &self.visible_entities
    }

    fn add_entity(&mut self, entity: Entity, position: Point3<f32>) {
        self.entities.insert(entity, position);
    }

    fn remove_entity(&mut self, entity: &Entity) {
        assert_eq!(self.entities.remove(entity).is_some(), true);
    }

    fn add_visible_entity(&mut self, entity: Entity) {
        self.visible_entities.insert(entity);
    }

    fn remove_visible_entity(&mut self, entity: &Entity) {
        assert_eq!(self.visible_entities.remove(entity), true);
    }
}

pub struct ClientEntityZone {
    pub sector_size: f32,
    pub sector_limit_squared: f32,
    pub num_sectors_x: u32,
    pub num_sectors_y: u32,
    pub sectors_base_position: Point2<f32>,
    pub entity_list_by_id: Vec<Option<Entity>>,
    pub last_free_entity_index: Option<usize>,
    pub sectors: Vec<ClientEntitySector>,
}

impl ClientEntityZone {
    pub fn new(zone_info: &ZoneData) -> Self {
        let sector_size = zone_info.sector_size as f32;
        let sector_limit = (sector_size / 2.0) + (sector_size * 0.2);

        Self {
            sector_size,
            num_sectors_x: zone_info.num_sectors_x,
            num_sectors_y: zone_info.num_sectors_y,
            sectors_base_position: zone_info.sectors_base_position,
            entity_list_by_id: vec![None; MAX_CLIENT_ENTITY_ID],
            last_free_entity_index: Some(1),
            sector_limit_squared: sector_limit * sector_limit,
            sectors: vec![
                Default::default();
                (zone_info.num_sectors_x * zone_info.num_sectors_y) as usize
            ],
        }
    }

    fn calculate_sector(&self, position: Point3<f32>) -> Point2<u32> {
        let sector = (position.xy() - self.sectors_base_position) / self.sector_size;
        Point2::new(
            u32::min(
                i32::max(0i32, sector[0] as i32) as u32,
                self.num_sectors_x - 1,
            ),
            u32::min(
                i32::max(0i32, sector[1] as i32) as u32,
                self.num_sectors_y - 1,
            ),
        )
    }

    fn calculate_sector_midpoint(&self, sector: Point2<u32>) -> Point2<f32> {
        self.sectors_base_position
            + Vector2::new(sector[0] as f32 + 0.5, sector[1] as f32 + 0.5) * self.sector_size
    }

    fn add_sector_entity(&mut self, sector: Point2<u32>, entity: &Entity, position: Point3<f32>) {
        // Add to the sector
        self.get_sector_mut(sector).add_entity(*entity, position);

        // Add to visible list in all adjacent sectors
        let min_sector_x = sector.x.saturating_sub(1);
        let max_sector_x = u32::min(sector.x + 1, self.num_sectors_x - 1);
        let min_sector_y = sector.y.saturating_sub(1);
        let max_sector_y = u32::min(sector.y + 1, self.num_sectors_y - 1);

        for x in min_sector_x..=max_sector_x {
            for y in min_sector_y..=max_sector_y {
                self.get_sector_mut(Point2::new(x, y))
                    .add_visible_entity(*entity);
            }
        }
    }

    fn update_entity_position(
        &mut self,
        sector: Point2<u32>,
        entity: &Entity,
        position: Point3<f32>,
    ) {
        self.get_sector_mut(sector).add_entity(*entity, position);
    }

    fn remove_sector_entity(&mut self, sector: Point2<u32>, entity: &Entity) {
        // Remove from the sector
        self.get_sector_mut(sector).remove_entity(entity);

        // Remove from visible list in all adjacent sectors
        let min_sector_x = sector.x.saturating_sub(1);
        let max_sector_x = u32::min(sector.x + 1, self.num_sectors_x - 1);
        let min_sector_y = sector.y.saturating_sub(1);
        let max_sector_y = u32::min(sector.y + 1, self.num_sectors_y - 1);

        for x in min_sector_x..=max_sector_x {
            for y in min_sector_y..=max_sector_y {
                self.get_sector_mut(Point2::new(x, y))
                    .remove_visible_entity(entity);
            }
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

            let sector = self.calculate_sector(position);
            self.add_sector_entity(sector, &entity, position);
            Some(ClientEntity::new(id, sector))
        } else {
            None
        }
    }

    fn get_sector(&self, sector: Point2<u32>) -> &ClientEntitySector {
        &self.sectors[sector[0] as usize + (sector[1] * self.num_sectors_x) as usize]
    }

    fn get_sector_mut(&mut self, sector: Point2<u32>) -> &mut ClientEntitySector {
        &mut self.sectors[sector[0] as usize + (sector[1] * self.num_sectors_x) as usize]
    }

    pub fn get_sector_visible_entities(&self, sector: Point2<u32>) -> &HashSet<Entity> {
        self.get_sector(sector).get_visible_entities()
    }

    pub fn update_sector(
        &mut self,
        entity: &Entity,
        client_entity: &mut ClientEntity,
        position: Point3<f32>,
    ) {
        let midpoint = self.calculate_sector_midpoint(client_entity.sector);
        if (midpoint - position.xy()).magnitude_squared() > self.sector_limit_squared {
            let previous_sector = client_entity.sector;
            let new_sector = self.calculate_sector(position);
            self.remove_sector_entity(previous_sector, &entity);
            self.add_sector_entity(new_sector, &entity, position);
            client_entity.sector = new_sector;
        } else {
            self.update_entity_position(client_entity.sector, &entity, position);
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
    pub zones: HashMap<u16, ClientEntityZone>,
}

impl ClientEntityList {
    pub fn new(zone_database: &ZoneDatabase) -> Self {
        let mut zones = HashMap::new();
        for (id, zone) in zone_database.iter() {
            zones.insert(*id, ClientEntityZone::new(zone));
        }
        Self { zones }
    }

    pub fn get_zone(&self, zone: usize) -> Option<&ClientEntityZone> {
        self.zones.get(&(zone as u16))
    }

    pub fn get_zone_mut(&mut self, zone: usize) -> Option<&mut ClientEntityZone> {
        self.zones.get_mut(&(zone as u16))
    }
}
