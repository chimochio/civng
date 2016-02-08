// Copyright 2015 Virgil Dupras
//
// This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
// which should be included with this package. The terms are also available at
// http://www.gnu.org/licenses/gpl-3.0.html
//

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::iter::FromIterator;

use byteorder::{LittleEndian, ReadBytesExt};

use terrain::{Terrain, TerrainMap};

#[allow(dead_code)]
struct MapHeader {
    version: u8,
    width: u32,
    height: u32,
    playercount: u8,
    flags: u32,
    terrain: Vec<String>,
    features1: Vec<String>,
    features2: Vec<String>,
    resources: Vec<String>,
    name: String,
    description: String,
    unknown: String,
}

#[allow(dead_code)]
struct MapTile {
    terrain_id: u8,
    resource_id: u8,
    feature1_id: u8,
    river_flags: u8,
    elevation: u8, // 0 = flat, 1 = hill, 2 = mountain
    unknown1: u8,
    feature2_id: u8,
    unknown2: u8,
}

fn read_str(fp: &mut File, len: u32) -> String {
    let bytes = fp.bytes().take(len as usize).map(|x| x.unwrap()).collect::<Vec<u8>>();
    let s = String::from_utf8(bytes).unwrap();
    s
}

fn read_str_list(fp: &mut File, len: u32) -> Vec<String> {
    let s = read_str(fp, len);
    let result: Vec<String> = s.split('\0').map(|s| s.to_string()).collect();
    result
}

fn load_map_header(fp: &mut File) -> MapHeader {
    let version = fp.read_u8().unwrap();
    let width = fp.read_u32::<LittleEndian>().unwrap();
    let height = fp.read_u32::<LittleEndian>().unwrap();
    let playercount = fp.read_u8().unwrap();
    let flags = fp.read_u32::<LittleEndian>().unwrap();
    let terrain_len = fp.read_u32::<LittleEndian>().unwrap();
    let feature1_len = fp.read_u32::<LittleEndian>().unwrap();
    let feature2_len = fp.read_u32::<LittleEndian>().unwrap();
    let resource_len = fp.read_u32::<LittleEndian>().unwrap();
    let _ = fp.read_u32::<LittleEndian>().unwrap();
    let mapname_len = fp.read_u32::<LittleEndian>().unwrap();
    let mapdesc_len = fp.read_u32::<LittleEndian>().unwrap();
    let terrain_list = read_str_list(fp, terrain_len);
    let feature1_list = read_str_list(fp, feature1_len);
    let feature2_list = read_str_list(fp, feature2_len);
    let resource_list = read_str_list(fp, resource_len);
    let mapname = read_str(fp, mapname_len);
    let mapdesc = read_str(fp, mapdesc_len);
    let unknown_len = fp.read_u32::<LittleEndian>().unwrap();
    let unknown = read_str(fp, unknown_len);
    MapHeader {
        version: version,
        width: width,
        height: height,
        playercount: playercount,
        flags: flags,
        terrain: terrain_list,
        features1: feature1_list,
        features2: feature2_list,
        resources: resource_list,
        name: mapname,
        description: mapdesc,
        unknown: unknown,
    }
}

fn load_map_tiles(fp: &mut File, len: u32) -> Vec<MapTile> {
    let mut result: Vec<MapTile> = Vec::new();
    for _ in 0..len {
        let mut bytes: [u8; 8] = [0; 8];
        let _ = fp.read(&mut bytes);
        result.push(MapTile {
            terrain_id: bytes[0],
            resource_id: bytes[1],
            feature1_id: bytes[2],
            river_flags: bytes[3],
            elevation: bytes[4],
            unknown1: bytes[5],
            feature2_id: bytes[6],
            unknown2: bytes[7],
        });
    }
    result
}

pub fn load_civ5map(path: &Path) -> TerrainMap {
    let mut fp = File::open(path).unwrap();
    let mh = load_map_header(&mut fp);
    let tiles = load_map_tiles(&mut fp, mh.width * mh.height);
    let mut mapdata: Vec<Terrain> = Vec::new();
    let name2terrain = HashMap::<&str, Terrain>::from_iter(vec![
            ("TERRAIN_COAST", Terrain::Water),
            ("TERRAIN_OCEAN", Terrain::Water),
            ("TERRAIN_GRASS", Terrain::Grassland),
            ("TERRAIN_PLAINS", Terrain::Plain),
            ("TERRAIN_DESERT", Terrain::Desert),
        ]);
    for tile in tiles.iter() {
        let name = &mh.terrain[tile.terrain_id as usize];
        let terrain = match tile.elevation {
            1 => Terrain::Hill,
            2 => Terrain::Mountain,
            _ => {
                match name2terrain.get(&name[..]) {
                    Some(t) => *t,
                    None => Terrain::Desert,
                }
            }
        };
        mapdata.push(terrain);
    }
    TerrainMap::new(mh.width as i32, mh.height as i32, mapdata)
}
