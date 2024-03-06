use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use clap::Command;

use rose_file_readers::{
    AruaVfsIndex, ChrFile, EftFile, IfoFile, IrosePhVfsIndex, LitFile, PtlFile, StbFile,
    TitanVfsIndex, VfsFile, VfsIndex, VfsPath, VfsPathBuf, VirtualFilesystem,
    VirtualFilesystemDevice, ZonFile, ZscFile,
};

pub enum VfsType {
    Base,
    AruaVfs,
    TitanVfs,
    IrosePh,
}

fn main() {
    let command = Command::new("rose-vfs-dump")
        .about("ROSE VFS extractor")
        .arg(
            clap::Arg::new("print-paths")
                .long("print-paths")
                .help("Just print the discovered file paths instead of dumping them."),
        )
        .arg(
            clap::Arg::new("input-path")
                .long("input-path")
                .help("Directory of ROSE installation to extract VFS from, defaults to current directory.")
                .takes_value(true),
        )
        .arg(
            clap::Arg::new("output-path")
                .long("output-path")
                .help("Set the path to dump files to, defaults to 'extracted'.")
                .takes_value(true),
        )
        .arg(
            clap::Arg::new("vfs-type")
                .long("vfs-type")
                .help("Which format to read the VFS as")
                .takes_value(true)
                .value_parser(["rose", "aruarose", "titanrose", "iroseph"]),
        );
    let matches = command.get_matches();

    let print_paths = matches.is_present("print-paths");
    let output_path = PathBuf::from(
        matches
            .value_of("output-path")
            .unwrap_or("extracted")
            .to_string(),
    );
    let vfs_base_path = matches
        .value_of("input-path")
        .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    // Try to auto-detect which VFS format is in use when type is not specified
    let mut vfs_type = VfsType::Base;
    if let Some(vfs_type_str) = matches.value_of("vfs-type") {
        vfs_type = match vfs_type_str {
            "rose" => VfsType::Base,
            "aruarose" => VfsType::AruaVfs,
            "titanrose" => VfsType::TitanVfs,
            "iroseph" => VfsType::IrosePh,
            _ => panic!("Unxepected vfs-type {}", vfs_type_str),
        };
    } else if Path::exists(&vfs_base_path.join("data.prf")) {
        eprintln!("Detected iRosePH VFS data.prf");
        vfs_type = VfsType::IrosePh;
    } else if Path::exists(&vfs_base_path.join("data.trf")) {
        eprintln!("Detected TitanVFS data.trf");
        vfs_type = VfsType::TitanVfs;
    } else if Path::exists(Path::new("data.rose")) {
        eprintln!("Detected AruaVFS data.rose");
        vfs_type = VfsType::AruaVfs;
    } else if Path::exists(Path::new("data.idx")) {
        eprintln!("Detected base VFS data.ids");
        vfs_type = VfsType::Base;
    }

    let mut vfs_devices: Vec<Box<dyn VirtualFilesystemDevice + Send + Sync>> = Vec::new();
    match vfs_type {
        VfsType::Base => {
            vfs_devices.push(Box::new(
                VfsIndex::load(&vfs_base_path.join("data.idx")).unwrap_or_else(|_| {
                    panic!("Failed to load VFS at {}/data.idx", vfs_base_path.display())
                }),
            ));
        }
        VfsType::AruaVfs => {
            vfs_devices.push(Box::new(
                AruaVfsIndex::load(
                    &vfs_base_path.join("data.idx"),
                    &vfs_base_path.join("data.rose"),
                )
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to load AruaVFS at {}/data.idx",
                        vfs_base_path.display()
                    )
                }),
            ));
        }
        VfsType::TitanVfs => {
            vfs_devices.push(Box::new(
                TitanVfsIndex::load(
                    &vfs_base_path.join("data.idx"),
                    &vfs_base_path.join("data.trf"),
                )
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to load TitanVFS at {}/data.idx",
                        vfs_base_path.display()
                    )
                }),
            ));
        }
        VfsType::IrosePh => {
            vfs_devices.push(Box::new(
                IrosePhVfsIndex::load(&vfs_base_path.join("data.idx")).unwrap_or_else(|_| {
                    panic!(
                        "Failed to load iRosePH at {}/data.idx",
                        vfs_base_path.display()
                    )
                }),
            ));
        }
    }

    let mut file_list = FoundFiles::new(VirtualFilesystem::new(vfs_devices));
    for file in BASE_FILE_LIST {
        file_list.try_add_file(file);
    }

    eprintln!("Scanning .STB files...");
    for stb_path in file_list.get_with_extension("STB") {
        let Ok(stb) = file_list.vfs.read_file::<StbFile, _>(&stb_path) else {
            continue;
        };

        for x in 0..stb.rows() {
            for y in 0..stb.columns() {
                let value = stb.get(x, y);
                if !value.is_empty() {
                    file_list.try_add_file(value);
                }
            }
        }
    }

    eprintln!("Scanning .CHR files...");
    for chr_path in file_list.get_with_extension("CHR") {
        let Ok(chr) = file_list.vfs.read_file::<ChrFile, _>(&chr_path) else {
            continue;
        };

        for skeleton_path in &chr.skeleton_files {
            file_list.try_add_file(skeleton_path);
        }

        for motion_path in &chr.motion_files {
            file_list.try_add_file(motion_path);
        }

        for effect_path in &chr.effect_files {
            file_list.try_add_file(effect_path);
        }
    }

    eprintln!("Scanning .ZSC files...");
    for zsc_path in file_list.get_with_extension("ZSC") {
        let Ok(zsc) = file_list.vfs.read_file::<ZscFile, _>(&zsc_path) else {
            continue;
        };

        for mesh_path in zsc.meshes.iter() {
            file_list.try_add_file(mesh_path);
        }

        for material in zsc.materials.iter() {
            file_list.try_add_file(&material.path);
        }

        for effect_path in zsc.effects.iter() {
            file_list.try_add_file(effect_path);
        }

        for object in zsc.objects.iter() {
            for part in object.parts.iter() {
                if let Some(animation_path) = part.animation_path.as_ref() {
                    file_list.try_add_file(animation_path);
                }
            }
        }
    }

    eprintln!("Scanning .ZON files...");
    for zon_path in file_list.get_with_extension("ZON") {
        if let Ok(zon) = file_list.vfs.read_file::<ZonFile, _>(&zon_path) {
            for texture_path in &zon.tile_textures {
                file_list.try_add_file(texture_path);
            }
        }

        let zone_directory = zon_path.path().parent().unwrap_or_else(|| Path::new(""));
        for block_y in 0..64 {
            for block_x in 0..64 {
                file_list.try_add_file(zone_directory.join(format!("{}_{}.HIM", block_x, block_y)));
                file_list.try_add_file(zone_directory.join(format!("{}_{}.TIL", block_x, block_y)));
                file_list.try_add_file(zone_directory.join(format!("{}_{}.IFO", block_x, block_y)));
                file_list.try_add_file(zone_directory.join(format!("{}_{}.MOV", block_x, block_y)));
                file_list.try_add_file(zone_directory.join(format!(
                    "{}_{}/LIGHTMAP/BUILDINGLIGHTMAPDATA.LIT",
                    block_x, block_y
                )));
                file_list.try_add_file(zone_directory.join(format!(
                    "{}_{}/LIGHTMAP/OBJECTLIGHTMAPDATA.LIT",
                    block_x, block_y
                )));
                file_list.try_add_file(zone_directory.join(format!(
                    "{0:}_{1:}/{0:}_{1:}_/LIGHTMAP/_PLANELIGHTINGMAP.DDS",
                    block_x, block_y
                )));
            }
        }
    }

    eprintln!("Scanning .LIT files...");
    for lit_path in file_list.get_with_extension("LIT") {
        let Ok(lit) = file_list.vfs.read_file::<LitFile, _>(&lit_path) else {
            continue;
        };

        for object in &lit.objects {
            for part in &object.parts {
                file_list.try_add_file(&part.filename);
            }
        }
    }

    eprintln!("Scanning .IFO files...");
    for ifo_path in file_list.get_with_extension("IFO") {
        let Ok(ifo) = file_list.vfs.read_file::<IfoFile, _>(&ifo_path) else {
            continue;
        };

        for effect_object in &ifo.effect_objects {
            file_list.try_add_file(&effect_object.effect_path);
        }

        for sound_object in &ifo.sound_objects {
            file_list.try_add_file(&sound_object.sound_path);
        }

        for npc in &ifo.npcs {
            file_list.try_add_file(&npc.quest_file_name);
        }
    }

    eprintln!("Scanning .EFT files...");
    for eft_path in file_list.get_with_extension("EFT") {
        let Ok(eft) = file_list.vfs.read_file::<EftFile, _>(&eft_path) else {
            continue;
        };

        if let Some(sound_file) = &eft.sound_file {
            file_list.try_add_file(sound_file);
        }

        for particle in &eft.particles {
            file_list.try_add_file(&particle.particle_file);

            if let Some(animation_file) = &particle.animation_file {
                file_list.try_add_file(animation_file);
            }
        }

        for mesh in &eft.meshes {
            file_list.try_add_file(&mesh.mesh_texture_file);

            if let Some(mesh_animation_file) = &mesh.mesh_animation_file {
                file_list.try_add_file(mesh_animation_file);
            }

            if let Some(animation_file) = &mesh.animation_file {
                file_list.try_add_file(animation_file);
            }
        }
    }

    eprintln!("Scanning .PTL files...");
    for ptl_path in file_list.get_with_extension("PTL") {
        let Ok(ptl) = file_list.vfs.read_file::<PtlFile, _>(&ptl_path) else {
            continue;
        };

        for sequence in &ptl.sequences {
            file_list.try_add_file(&sequence.texture_path);
        }
    }

    eprintln!("Discovered {} files", file_list.all_files.len());

    if print_paths {
        for name in &file_list.all_files {
            println!("{}", name.path().display());
        }
    } else {
        eprintln!("Extracting files...");
        for name in &file_list.all_files {
            if let Ok(vfs_file) = file_list.vfs.open_file(name) {
                let output_file_path = output_path.join(name.path());
                std::fs::create_dir_all(output_file_path.parent().unwrap()).ok();
                match &vfs_file {
                    VfsFile::Buffer(buffer) => std::fs::write(output_file_path, buffer).ok(),
                    VfsFile::View(view) => std::fs::write(output_file_path, view).ok(),
                };
            }
        }
    }
}

struct FoundFiles {
    pub vfs: VirtualFilesystem,
    pub all_files: HashSet<VfsPathBuf>,
    pub by_extension: HashMap<String, HashSet<VfsPathBuf>>,
}

impl FoundFiles {
    pub fn new(vfs: VirtualFilesystem) -> FoundFiles {
        FoundFiles {
            vfs,
            all_files: Default::default(),
            by_extension: Default::default(),
        }
    }

    pub fn get_with_extension(&self, extension: &str) -> Vec<VfsPathBuf> {
        let extension = extension.to_ascii_uppercase();
        self.by_extension
            .get(&extension)
            .map_or(Vec::default(), |list| Vec::from_iter(list.iter().cloned()))
    }

    pub fn try_add_file<'a, P: Into<VfsPath<'a>>>(&mut self, path: P) -> bool {
        let path: VfsPath = path.into();
        if !self.vfs.exists(&path) {
            return false;
        }

        let path: VfsPathBuf = (&path).into();
        if self.all_files.contains(&path) {
            return false;
        }

        let Some(extension) = Path::new(path.path()).extension() else {
            return false;
        };

        let extension = extension.to_string_lossy().to_string();
        if extension.is_empty() {
            return false;
        }

        self.all_files.insert(path.clone());
        match self.by_extension.entry(extension) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().insert(path);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut set = HashSet::new();
                set.insert(path);
                entry.insert(set);
            }
        };

        true
    }
}

const BASE_FILE_LIST: [&str; 215] = [
    "3DDATA/AVATAR/FEMALE.ZMD",
    "3DDATA/AVATAR/LIST_BACK.ZSC",
    "3DDATA/AVATAR/LIST_FACEIEM.ZSC",
    "3DDATA/AVATAR/LIST_MARMS.ZSC",
    "3DDATA/AVATAR/LIST_MBODY.ZSC",
    "3DDATA/AVATAR/LIST_MCAP.ZSC",
    "3DDATA/AVATAR/LIST_MFACE.ZSC",
    "3DDATA/AVATAR/LIST_MFOOT.ZSC",
    "3DDATA/AVATAR/LIST_MHAIR.ZSC",
    "3DDATA/AVATAR/LIST_WARMS.ZSC",
    "3DDATA/AVATAR/LIST_WBODY.ZSC",
    "3DDATA/AVATAR/LIST_WCAP.ZSC",
    "3DDATA/AVATAR/LIST_WFACE.ZSC",
    "3DDATA/AVATAR/LIST_WFOOT.ZSC",
    "3DDATA/AVATAR/LIST_WHAIR.ZSC",
    "3DDATA/AVATAR/MALE.ZMD",
    "3DDATA/CONTROL/RES/CLANBACK.TSI",
    "3DDATA/CONTROL/RES/CLANCENTER.TSI",
    "3DDATA/CONTROL/RES/ITEM1.TSI",
    "3DDATA/CONTROL/RES/MINIMAP_ARROW.TGA",
    "3DDATA/CONTROL/RES/SKILLICON.TSI",
    "3DDATA/CONTROL/RES/SOKET.DDS",
    "3DDATA/CONTROL/RES/SOKETJAM.TSI",
    "3DDATA/CONTROL/RES/STATEICON.TSI",
    "3DDATA/CONTROL/RES/TARGETMARK.TSI",
    "3DDATA/CONTROL/XML/EXUI_STRID.ID",
    "3DDATA/CONTROL/XML/UI_STRID.ID",
    "3DDATA/EFFECT/LEVELUP_01.EFT",
    "3DDATA/EFFECT/SPECIAL/DIGITNUMBER01.DDS",
    "3DDATA/EFFECT/SPECIAL/DIGITNUMBER02.DDS",
    "3DDATA/EFFECT/SPECIAL/DIGITNUMBERMISS.DDS",
    "3DDATA/EFFECT/SPECIAL/HIT_FIGURE_01.ZMO",
    "3DDATA/EFFECT/TRAIL.DDS",
    "3DDATA/EVENT/OBJECT001.CON",
    "3DDATA/EVENT/OBJECT002.CON",
    "3DDATA/EVENT/OBJECT003.CON",
    "3DDATA/EVENT/OBJECT004.CON",
    "3DDATA/EVENT/OBJECT005.CON",
    "3DDATA/EVENT/OBJECT006.CON",
    "3DDATA/EVENT/OBJECT007.CON",
    "3DDATA/EVENT/OBJECT008.CON",
    "3DDATA/EVENT/OBJECT009.CON",
    "3DDATA/EVENT/ULNGTB_CON.LTB",
    "3DDATA/ITEM/LIST_FIELDITEM.ZSC",
    "3DDATA/MOTION/AVATAR/EVENT_SELECT_M1.ZMO",
    "3DDATA/MOTION/ITEM_ANI.ZMO",
    "3DDATA/NPC/LIST_NPC.CHR",
    "3DDATA/NPC/PART_NPC.ZSC",
    "3DDATA/PAT/CART/CART01.ZMD",
    "3DDATA/PAT/CASTLEGEAR/CASTLEGEAR02/CASTLEGEAR02.ZMD",
    "3DDATA/PAT/LIST_PAT.ZSC",
    "3DDATA/SPECIAL/EVENT_OBJECT.ZSC",
    "3DDATA/SPECIAL/LIST_DECO_SPECIAL.ZSC",
    "3DDATA/STB/BADNAMES.STB",
    "3DDATA/STB/BADWORDS.STB",
    "3DDATA/STB/EVENT_OBJECT.STB",
    "3DDATA/STB/EVENTBUTTON.STB",
    "3DDATA/STB/FILE_AI.STB",
    "3DDATA/STB/FILE_EFFECT.STB",
    "3DDATA/STB/FILE_MOTION.STB",
    "3DDATA/STB/FILE_SKEL.STB",
    "3DDATA/STB/FILE_SOUND.STB",
    "3DDATA/STB/FILE_SUFFIX_COLOR.STB",
    "3DDATA/STB/FILE_TUTORIAL.STB",
    "3DDATA/STB/HELP.STB",
    "3DDATA/STB/HELP_S.STL",
    "3DDATA/STB/INIT_AVATAR.STB",
    "3DDATA/STB/ITEM_DROP.STB",
    "3DDATA/STB/LEVELUPEVENT.STB",
    "3DDATA/STB/LIST_APPRAISAL_STAT.STB",
    "3DDATA/STB/LIST_ARMS.STB",
    "3DDATA/STB/LIST_ARMS_S.STL",
    "3DDATA/STB/LIST_BACK.STB",
    "3DDATA/STB/LIST_BACK_S.STL",
    "3DDATA/STB/LIST_BODY.STB",
    "3DDATA/STB/LIST_BODY_S.STL",
    "3DDATA/STB/LIST_BREAK.STB",
    "3DDATA/STB/LIST_BULLET.STB",
    "3DDATA/STB/LIST_CAMERA.STB",
    "3DDATA/STB/LIST_CAP.STB",
    "3DDATA/STB/LIST_CAP_S.STL",
    "3DDATA/STB/LIST_CLAN_COLOR.STB",
    "3DDATA/STB/LIST_CLASS.STB",
    "3DDATA/STB/LIST_CLASS_S.STL",
    "3DDATA/STB/LIST_CNST_EJ.STB",
    "3DDATA/STB/LIST_CNST_JD.STB",
    "3DDATA/STB/LIST_CNST_JDT.STB",
    "3DDATA/STB/LIST_CNST_JG.STB",
    "3DDATA/STB/LIST_CNST_JPT.STB",
    "3DDATA/STB/LIST_CNST_LMT.STB",
    "3DDATA/STB/LIST_CNST_ODD.STB",
    "3DDATA/STB/LIST_CNST_ODT.STB",
    "3DDATA/STB/LIST_CURRENCY.STB",
    "3DDATA/STB/LIST_CURRENCY_S.STL",
    "3DDATA/STB/LIST_DUEL_CONSUMABLES.STB",
    "3DDATA/STB/LIST_EFFECT.STB",
    "3DDATA/STB/LIST_EVENT.STB",
    "3DDATA/STB/LIST_EVENTSTRING.STL",
    "3DDATA/STB/LIST_FACE.STB",
    "3DDATA/STB/LIST_FACEITEM.STB",
    "3DDATA/STB/LIST_FACEITEM_S.STL",
    "3DDATA/STB/LIST_FIELDITEM.STB",
    "3DDATA/STB/LIST_FOOT.STB",
    "3DDATA/STB/LIST_FOOT_S.STL",
    "3DDATA/STB/LIST_GAMEARENA.STB",
    "3DDATA/STB/LIST_GAMEARENA_S.STL",
    "3DDATA/STB/LIST_GEMITEM.STB",
    "3DDATA/STB/LIST_GEMITEM_S.STL",
    "3DDATA/STB/LIST_GRADE.STB",
    "3DDATA/STB/LIST_GRADE_COLOR.STB",
    "3DDATA/STB/LIST_HAIR.STB",
    "3DDATA/STB/LIST_HELP.STB",
    "3DDATA/STB/LIST_HITSOUND.STB",
    "3DDATA/STB/LIST_ITEM_RESTRICTION.STB",
    "3DDATA/STB/LIST_ITEM_RESTRICTION_S.STL",
    "3DDATA/STB/LIST_JEMITEM.STB",
    "3DDATA/STB/LIST_JEMITEM_S.STL",
    "3DDATA/STB/LIST_JEWEL.STB",
    "3DDATA/STB/LIST_JEWEL_S.STL",
    "3DDATA/STB/LIST_LANGUAGE.STB",
    "3DDATA/STB/LIST_LANGUAGE_S.STL",
    "3DDATA/STB/LIST_LOADING.STB",
    "3DDATA/STB/LIST_MACRO.STB",
    "3DDATA/STB/LIST_MESH_EFFECT.STB",
    "3DDATA/STB/LIST_MORPH_OBJECT.STB",
    "3DDATA/STB/LIST_MOUNT.STB",
    "3DDATA/STB/LIST_MOUNT_S.STL",
    "3DDATA/STB/LIST_NATURAL.STB",
    "3DDATA/STB/LIST_NATURAL_S.STL",
    "3DDATA/STB/LIST_NPC.STB",
    "3DDATA/STB/LIST_NPC_S.STL",
    "3DDATA/STB/LIST_NPCFACE.STB",
    "3DDATA/STB/LIST_PARTICLES.STB",
    "3DDATA/STB/LIST_PAT.STB",
    "3DDATA/STB/LIST_PAT_S.STL",
    "3DDATA/STB/LIST_PATWPN.STB",
    "3DDATA/STB/LIST_PRODUCT.STB",
    "3DDATA/STB/LIST_QUEST.STB",
    "3DDATA/STB/LIST_QUEST_S.STL",
    "3DDATA/STB/LIST_QUESTDATA.STB",
    "3DDATA/STB/LIST_QUESTIMAGE.STB",
    "3DDATA/STB/LIST_QUESTITEM.STB",
    "3DDATA/STB/LIST_QUESTITEM_S.STL",
    "3DDATA/STB/LIST_REFINE.STB",
    "3DDATA/STB/LIST_SELL.STB",
    "3DDATA/STB/LIST_SELL_S.STL",
    "3DDATA/STB/LIST_SET.STB",
    "3DDATA/STB/LIST_SET_S.STL",
    "3DDATA/STB/LIST_SKILL.STB",
    "3DDATA/STB/LIST_SKILL_P.STB",
    "3DDATA/STB/LIST_SKILL_S.STL",
    "3DDATA/STB/LIST_SKY.STB",
    "3DDATA/STB/LIST_STATUS.STB",
    "3DDATA/STB/LIST_STATUS_ITEMMALL.STB",
    "3DDATA/STB/LIST_STATUS_ITEMMALL_S.STL",
    "3DDATA/STB/LIST_STATUS_S.STL",
    "3DDATA/STB/LIST_STEPSOUND.STB",
    "3DDATA/STB/LIST_STRING.STB",
    "3DDATA/STB/LIST_STRING.STL",
    "3DDATA/STB/LIST_SUBWPN.STB",
    "3DDATA/STB/LIST_SUBWPN_S.STL",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_EJ.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_EZ.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_JD.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_JDT.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_JG.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_JPT.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_JZ.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_JZC.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_JZP.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_LP.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_LZ.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_ODD.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_ODG.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_ODT.STB",
    "3DDATA/STB/LIST_TERRAIN_OBJECT_SPECIAL.STB",
    "3DDATA/STB/LIST_UNION.STB",
    "3DDATA/STB/LIST_UNION_S.STL",
    "3DDATA/STB/LIST_UPGRADE.STB",
    "3DDATA/STB/LIST_USEITEM.STB",
    "3DDATA/STB/LIST_USEITEM_S.STL",
    "3DDATA/STB/LIST_WEAPON.STB",
    "3DDATA/STB/LIST_WEAPON_S.STL",
    "3DDATA/STB/LIST_ZONE.STB",
    "3DDATA/STB/LIST_ZONE_BLOCKUSEITEM.STB",
    "3DDATA/STB/LIST_ZONE_S.STL",
    "3DDATA/STB/PART_NPC.STB",
    "3DDATA/STB/PRODUCT.STB",
    "3DDATA/STB/QUEST_TRACKER.STB",
    "3DDATA/STB/QUEST_TRACKER_ITEM.STB",
    "3DDATA/STB/RANGESET.STB",
    "3DDATA/STB/RESOLUTION.STB",
    "3DDATA/STB/STR_ABILITY.STL",
    "3DDATA/STB/STR_CLAN.STL",
    "3DDATA/STB/STR_ITEMGRADE.STL",
    "3DDATA/STB/STR_ITEMGRADECOLOR.STL",
    "3DDATA/STB/STR_ITEMMALL_CATEGORY.STL",
    "3DDATA/STB/STR_ITEMMALL_COMMENT.STL",
    "3DDATA/STB/STR_ITEMPREFIX.STL",
    "3DDATA/STB/STR_ITEMSUFFIX.STL",
    "3DDATA/STB/STR_ITEMTYPE.STL",
    "3DDATA/STB/STR_JOB.STL",
    "3DDATA/STB/STR_PLANET.STL",
    "3DDATA/STB/STR_SKILLFORMULA.STL",
    "3DDATA/STB/STR_SKILLTARGET.STL",
    "3DDATA/STB/STR_SKILLTYPE.STL",
    "3DDATA/STB/TYPE_MOTION.STB",
    "3DDATA/STB/WARP.STB",
    "3DDATA/TITLE/CAMERA01_CREATE01.ZMO",
    "3DDATA/TITLE/CAMERA01_INGAME01.ZMO",
    "3DDATA/TITLE/CAMERA01_INSELECT01.ZMO",
    "3DDATA/TITLE/CAMERA01_INTRO01.ZMO",
    "3DDATA/TITLE/CAMERA01_OUTCREATE01.ZMO",
    "3DDATA/WEAPON/LIST_SUBWPN.ZSC",
    "3DDATA/WEAPON/LIST_WEAPON.ZSC",
];
