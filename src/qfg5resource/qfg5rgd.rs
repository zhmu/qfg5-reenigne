/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::{anyhow, Result};
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::io::{Cursor, Seek, SeekFrom};

pub struct RgdDecoder {
}

pub struct RgdPoint {
    pub x: f64,
    pub y: f64,
}

pub struct RgdVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct RgdSegment {
    pub point1: usize,
    pub point2: usize,
    pub regionid_offset: u64,
}

#[derive(Debug)]
pub struct RgdRegion {
    pub vector_index: usize,
    pub offset_segment_ids: u64,
}

impl RgdDecoder {
    pub fn new(rgd_data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&rgd_data);
        let a = cursor.read_u32::<LittleEndian>()?;
        if a != 0 { return Err(anyhow!("a mismatch")); }
        let b = cursor.read_u32::<LittleEndian>()?;
        if b != 2 { return Err(anyhow!("b mismatch")); }
        // [ok] total number of regions;
        let num_regions = cursor.read_u32::<LittleEndian>()? as usize;
        // [ok] region data offset (each region includes among other things a 3-D vector index and an offset to a list of segment IDs);
        let offset_region_data = cursor.read_u32::<LittleEndian>()? as u64;

        // TODO: offset to a list of offsets, data those offsets has a list of vector indices
        let _offset_to_list_of_offsets = cursor.read_u32::<LittleEndian>()? as u64;
        // some ignored offset;
        let _ignored_offset = cursor.read_u32::<LittleEndian>()? as u64;
        // TODO: offset to an array of some region positioning information
        let _offset_region_pos_info = cursor.read_u32::<LittleEndian>()? as u64;

        // [ok] total number of regions
        let num_regions2 = cursor.read_u32::<LittleEndian>()? as usize;
        // [ok] offset to a full list of region IDs
        let offset_full_list_regionids = cursor.read_u32::<LittleEndian>()? as u64;

        // TODO: total number of region IDs
        let _num_regions3 = cursor.read_u32::<LittleEndian>()? as usize;
        // data start offset (seems to be always 0x5C)?
        let offset_data_start = cursor.read_u32::<LittleEndian>()? as u64;
        if offset_data_start != 0x5c { return Err(anyhow!("invalid data start offset {:x}", offset_data_start)); }
        // [ok] number of segments
        let num_segments = cursor.read_u32::<LittleEndian>()? as usize;
        // [ok] offset to segment data (which includes two point indices and an offset to region ID list);
        let offset_segment_data = cursor.read_u32::<LittleEndian>()? as u64;
        // [ok] number of points 
        let num_points = cursor.read_u32::<LittleEndian>()? as usize;
        // [ok] offset to point data (two doubles per point)
        let offset_point_data = cursor.read_u32::<LittleEndian>()? as u64;
        // [ok] number of vectors
        let num_vectors = cursor.read_u32::<LittleEndian>()? as usize;
        // [ok] offset to vector data (three doubles per vector)
        let offset_vector_data = cursor.read_u32::<LittleEndian>()? as u64;
        // TODO: flag signalling that the following fields are meaningful
        let _flag = cursor.read_u32::<LittleEndian>()?;
        // TODO: number of special (walkable?) regions
        let _num_special_regions = cursor.read_u32::<LittleEndian>()? as usize;
        // TODO: connectivity matrix offset (that number of regions squared, -1 and -2 mean there’s no connection)
        let _connectivity_matrix1_offset = cursor.read_u32::<LittleEndian>()? as u64;
        // TODO: another connectivity matrix (in the same format) offset
        let _connectivity_matrix2_offset = cursor.read_u32::<LittleEndian>()? as u64;
        // TODO: offset to the list of special region IDs.
        let _offset_special_region_ids = cursor.read_u32::<LittleEndian>()? as u64;

        cursor.seek(SeekFrom::Start(offset_point_data))?;
        let mut points = Vec::with_capacity(num_points);
        for _ in 0..num_points {
            let x = cursor.read_f64::<LittleEndian>()?;
            let y = cursor.read_f64::<LittleEndian>()?;
            points.push(RgdPoint{ x, y });
        }

        cursor.seek(SeekFrom::Start(offset_vector_data))?;
        let mut vectors = Vec::with_capacity(num_vectors);
        for _ in 0..num_vectors {
            let x = cursor.read_f64::<LittleEndian>()?;
            let y = cursor.read_f64::<LittleEndian>()?;
            let z = cursor.read_f64::<LittleEndian>()?;
            vectors.push(RgdVector{ x, y, z });
        }

        cursor.seek(SeekFrom::Start(offset_segment_data))?;
        let mut segments = Vec::with_capacity(num_segments);
        for _ in 0..num_segments {
            let point1 = cursor.read_u32::<LittleEndian>()? as usize;
            let point2 = cursor.read_u32::<LittleEndian>()? as usize;
            let regionid_offset = cursor.read_f64::<LittleEndian>()? as u64;
            segments.push(RgdSegment{ point1, point2, regionid_offset });
        }

        cursor.seek(SeekFrom::Start(offset_full_list_regionids))?;
        let mut region_ids = Vec::with_capacity(num_regions2);
        for _ in 0..num_regions2 {
            let region_id = cursor.read_u32::<LittleEndian>()?;
            region_ids.push(region_id);
        }

        cursor.seek(SeekFrom::Start(offset_region_data))?;
        let mut regions = Vec::with_capacity(num_regions);
        for _ in 0..num_regions {
            let vector_index = cursor.read_u32::<LittleEndian>()? as usize;
            let offset_segment_ids = cursor.read_u32::<LittleEndian>()? as u64;
            regions.push(RgdRegion{ vector_index, offset_segment_ids });
        }
        println!("{:x?}", regions);
        Ok(Self{})
    }
}
