use std::collections::HashMap;

use bevy_ecs::prelude::Entity;
use bitvec::prelude::*;
use nalgebra::{Point2, Point3, Vector2};

use crate::{
    data::{ZoneData, ZoneDatabase, ZoneId},
    game::components::{ClientEntity, ClientEntityId, ClientEntityType},
};

const MAX_CLIENT_ENTITY_ID: usize = 4096;

pub type ClientEntitySet = BitArr!(for MAX_CLIENT_ENTITY_ID);

#[derive(Clone, Default)]
struct ClientEntityZoneSector {
    // The list of entities currently inside this sector
    entities: ClientEntitySet,

    // The list of entities visible from this sector, this is a union of the entities of all adjacent sectors
    visible_entities: ClientEntitySet,
}

#[allow(dead_code)]
impl ClientEntityZoneSector {
    pub fn get_visible_entities(&self) -> &ClientEntitySet {
        &self.visible_entities
    }

    fn join_sector(&mut self, id: ClientEntityId) {
        self.entities.set(id.0, true);
    }

    fn leave_sector(&mut self, id: ClientEntityId) {
        self.entities.set(id.0, false);
    }

    fn add_visible_entity(&mut self, id: ClientEntityId) {
        self.visible_entities.set(id.0, true);
    }

    fn remove_visible_entity(&mut self, id: ClientEntityId) {
        self.visible_entities.set(id.0, false);
    }
}

pub struct ClientEntityZone {
    // Current zone id
    zone_id: ZoneId,

    // The size (width and height) of a sector
    sector_size: f32,

    // Distance from middle of sector before leaving sector
    sector_leave_distance_squared: f32,

    // X, Y position of the first sector
    sector_base_position: Point2<f32>,

    // Number of sectors in X and Y direction
    sector_count: Point2<u32>,

    // The list of sectors
    sectors: Vec<ClientEntityZoneSector>,

    // The list of entities currently inside this zone
    entities: Vec<Option<(Entity, ClientEntity, Point3<f32>)>>,

    // The list of entities leaving the zone, this is so we can process any
    // visibility changes before freeing the entity id
    leaving_entities: Vec<ClientEntityId>,
}

impl ClientEntityZone {
    pub fn new(zone_info: &ZoneData) -> Self {
        let sector_size = zone_info.sector_size as f32;
        let sector_limit = (sector_size / 2.0) + (sector_size * 0.2);

        Self {
            zone_id: zone_info.id,
            sector_size,
            sector_count: Point2::new(zone_info.num_sectors_x, zone_info.num_sectors_y),
            sector_base_position: zone_info.sectors_base_position,
            sector_leave_distance_squared: sector_limit * sector_limit,
            sectors: vec![
                Default::default();
                (zone_info.num_sectors_x * zone_info.num_sectors_y) as usize
            ],
            entities: vec![None; MAX_CLIENT_ENTITY_ID],
            leaving_entities: Vec::new(),
        }
    }

    pub fn calculate_sector(&self, position: Point2<f32>) -> Point2<u32> {
        let sector = (position - self.sector_base_position) / self.sector_size;
        Point2::new(
            u32::min(
                i32::max(0i32, sector[0] as i32) as u32,
                self.sector_count.x - 1,
            ),
            u32::min(
                i32::max(0i32, sector[1] as i32) as u32,
                self.sector_count.y - 1,
            ),
        )
    }

    fn calculate_sector_midpoint(&self, sector: Point2<u32>) -> Point2<f32> {
        self.sector_base_position
            + Vector2::new(sector[0] as f32 + 0.5, sector[1] as f32 + 0.5) * self.sector_size
    }

    fn get_sector(&self, sector: Point2<u32>) -> &ClientEntityZoneSector {
        &self.sectors[sector[0] as usize + (sector[1] * self.sector_count.x) as usize]
    }

    fn get_sector_mut(&mut self, sector: Point2<u32>) -> &mut ClientEntityZoneSector {
        &mut self.sectors[sector[0] as usize + (sector[1] * self.sector_count.x) as usize]
    }

    pub fn get_sector_visible_entities(&self, sector: Point2<u32>) -> &ClientEntitySet {
        self.get_sector(sector).get_visible_entities()
    }

    pub fn get_entity(&self, id: ClientEntityId) -> Option<&(Entity, ClientEntity, Point3<f32>)> {
        self.entities[id.0].as_ref()
    }

    fn for_each_visible_sector<F>(&mut self, sector: Point2<u32>, mut f: F)
    where
        F: FnMut(&mut ClientEntityZoneSector),
    {
        let min_sector_x = sector.x.saturating_sub(1);
        let max_sector_x = u32::min(sector.x + 1, self.sector_count.x - 1);
        let min_sector_y = sector.y.saturating_sub(1);
        let max_sector_y = u32::min(sector.y + 1, self.sector_count.y - 1);

        for x in min_sector_x..=max_sector_x {
            for y in min_sector_y..=max_sector_y {
                f(self.get_sector_mut(Point2::new(x, y)))
            }
        }
    }

    fn join_sector(&mut self, sector: Point2<u32>, id: ClientEntityId) {
        // Join the sector
        self.get_sector_mut(sector).join_sector(id);

        // Join the visible set of adjacent sectors
        self.for_each_visible_sector(sector, |zone_sector| zone_sector.add_visible_entity(id));
    }

    fn leave_sector(&mut self, sector: Point2<u32>, id: ClientEntityId) {
        // Leave the sector
        self.get_sector_mut(sector).leave_sector(id);

        // Leave the visible set of adjacent sectors
        self.for_each_visible_sector(sector, |zone_sector| zone_sector.remove_visible_entity(id));
    }

    pub fn join_zone(
        &mut self,
        entity_type: ClientEntityType,
        entity: Entity,
        position: Point3<f32>,
    ) -> Option<ClientEntity> {
        let sector = self.calculate_sector(position.xy());

        // Allocate an entity id
        let (free_index, free_slot) = self
            .entities
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())?;
        let client_entity_id = ClientEntityId(free_index);
        let client_entity = ClientEntity::new(entity_type, client_entity_id, self.zone_id, sector);

        // Join zone
        *free_slot = Some((entity, client_entity.clone(), position));

        // Join sector
        self.join_sector(sector, client_entity_id);

        Some(client_entity)
    }

    pub fn leave_zone(&mut self, entity: Entity, client_entity: &ClientEntity) {
        // Validate entity list
        assert_eq!(
            self.entities[client_entity.id.0].as_ref().map(|x| &x.0),
            Some(&entity)
        );

        // Leave sector
        self.leave_sector(client_entity.sector, client_entity.id);

        // Set as leaving zone
        self.leaving_entities.push(client_entity.id);
    }

    pub fn update_position(
        &mut self,
        entity: Entity,
        client_entity: &mut ClientEntity,
        position: Point3<f32>,
    ) {
        // Validate entity list
        assert_eq!(
            self.entities[client_entity.id.0].as_ref().map(|x| &x.0),
            Some(&entity)
        );

        // Update sector
        let midpoint = self.calculate_sector_midpoint(client_entity.sector);
        if (midpoint - position.xy()).magnitude_squared() > self.sector_leave_distance_squared {
            let previous_sector = client_entity.sector;
            let new_sector = self.calculate_sector(position.xy());
            self.leave_sector(previous_sector, client_entity.id);
            self.join_sector(new_sector, client_entity.id);
            client_entity.sector = new_sector;
        }

        // Update the entity data storage
        self.entities[client_entity.id.0] = Some((entity, client_entity.clone(), position));
    }

    pub fn process_zone_leavers(&mut self) {
        // Free the entity id
        for id in self.leaving_entities.iter() {
            self.entities[id.0] = None;
        }

        self.leaving_entities.clear();
    }

    pub fn iter_entities_within_distance(
        &self,
        origin: Point2<f32>,
        distance: f32,
    ) -> ClientEntityZoneEntityIterator<'_> {
        let min_sector = self.calculate_sector(origin - Vector2::new(distance, distance));
        let max_sector = self.calculate_sector(origin + Vector2::new(distance, distance));

        ClientEntityZoneEntityIterator {
            zone: &self,
            min_sector,
            max_sector,
            current_sector: min_sector,
            current_iter: self.get_sector(min_sector).entities.iter_ones(),
            origin,
            max_distance_squared: distance * distance,
        }
    }
}

pub struct ClientEntityZoneEntityIterator<'a> {
    zone: &'a ClientEntityZone,
    min_sector: Point2<u32>,
    max_sector: Point2<u32>,
    current_sector: Point2<u32>,
    current_iter: bitvec::slice::IterOnes<'a, Lsb0, usize>,
    origin: Point2<f32>,
    max_distance_squared: f32,
}

impl Iterator for ClientEntityZoneEntityIterator<'_> {
    type Item = (Entity, Point3<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(index) = self.current_iter.next() {
                if let Some((entity, _, position)) = self.zone.entities[index].as_ref() {
                    let distance_squared = (position.xy() - self.origin).magnitude_squared();
                    if distance_squared <= self.max_distance_squared {
                        break Some((*entity, *position));
                    } else {
                        continue;
                    }
                }
            }

            self.current_sector.x += 1;

            if self.current_sector.x > self.max_sector.x {
                self.current_sector.x = self.min_sector.x;
                self.current_sector.y += 1;
            }

            if self.current_sector.y > self.max_sector.y {
                break None;
            }

            self.current_iter = self
                .zone
                .get_sector(self.current_sector)
                .entities
                .iter_ones();
        }
    }
}

pub struct ClientEntityList {
    pub zones: HashMap<ZoneId, ClientEntityZone>,
}

impl ClientEntityList {
    pub fn new(zone_database: &ZoneDatabase) -> Self {
        let mut zones = HashMap::new();
        for (id, zone) in zone_database.iter() {
            zones.insert(*id, ClientEntityZone::new(zone));
        }
        Self { zones }
    }

    pub fn get_zone(&self, zone_id: ZoneId) -> Option<&ClientEntityZone> {
        self.zones.get(&zone_id)
    }

    pub fn get_zone_mut(&mut self, zone_id: ZoneId) -> Option<&mut ClientEntityZone> {
        self.zones.get_mut(&zone_id)
    }

    pub fn process_zone_leavers(&mut self) {
        self.zones
            .values_mut()
            .for_each(ClientEntityZone::process_zone_leavers);
    }
}
