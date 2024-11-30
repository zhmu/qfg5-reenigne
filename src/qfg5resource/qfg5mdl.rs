/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::{anyhow, Result};
use byteorder::{ReadBytesExt, LittleEndian};
use std::io::{Cursor, Read, Seek, SeekFrom};
use log::{info, debug};

#[derive(Default, Clone)]
pub struct Qfg5Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Default, Clone)]
pub struct Qfg5LightingVertex {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

#[derive(Default, Clone)]
pub struct Qfg5TexCoord {
    pub u: f32,
    pub v: f32,
}

#[derive(Default, Clone)]
pub struct Qfg5Face {
    pub vertex1: usize,
    pub vertex2: usize,
    pub vertex3: usize,
    pub uv1: usize,
    pub uv2: usize,
    pub uv3: usize,
    pub subbitmap: usize,
    pub normal_x: f32,
    pub normal_y: f32,
    pub normal_z: f32,
}

#[derive(Default, Clone)]
pub struct SubMesh {
    pub name: String,
    pub vertices: Vec<Qfg5Vertex>,
    pub texcoords: Vec<Qfg5TexCoord>,
    pub faces: Vec<Qfg5Face>,
    pub lighting_vertices: Vec<Qfg5LightingVertex>,
}

#[derive(Default, Clone)]
pub struct SubBitmap {
    pub width: u32,
    pub height: u32,
    pub bitmap: Vec<u8>
}

pub struct Qfg5Model {
    pub name: String,
    pub palette: Vec<u8>,
    pub submeshes: Vec<SubMesh>,
    pub subbitmaps: Vec<SubBitmap>,
}

impl Qfg5Model {
    pub fn new(data: &[u8]) -> Result<Qfg5Model> {
        let mut cursor = Cursor::new(data);
        cursor.seek(SeekFrom::Current(0xc))?;

        let mut name = vec![ 0u8; 16 ];
        cursor.read_exact(&mut name)?;
        let name = String::from_utf8(name)?;

        let num_submeshes = cursor.read_u16::<LittleEndian>()? as usize;
        info!("model '{}': {} submeshes", name, num_submeshes);
        cursor.seek(SeekFrom::Current(0xf))?;
        let mut palette = vec![ 0u8; 1019 ];
        cursor.read_exact(&mut palette)?;
        let bitmap_texture_offset = cursor.read_u32::<LittleEndian>()? as u64;
        let mut submesh_offset = vec![ 0u64; num_submeshes ];
        for n in 0..num_submeshes {
            submesh_offset[n] = cursor.read_u32::<LittleEndian>()? as u64;
        }

        let mut submeshes = Vec::with_capacity(num_submeshes);
        for n in 0..num_submeshes {
            cursor.seek(SeekFrom::Start(submesh_offset[n]))?;

            let mut name = vec![ 0u8; 16 ];
            cursor.read_exact(&mut name)?;
            let name = String::from_utf8(name)?;

            for _ in 0..20 {
                let _unk = cursor.read_f32::<LittleEndian>()? as usize;
                debug!("unknown float value {}", _unk);
            }

            //cursor.seek(SeekFrom::Current(0x50))?;
            let num_vertices = cursor.read_u32::<LittleEndian>()? as usize;
            let num_uv_coords = cursor.read_u32::<LittleEndian>()? as usize;
            let num_faces = cursor.read_u32::<LittleEndian>()? as usize;
            let vlist_addr = cursor.read_u32::<LittleEndian>()?;
            if vlist_addr != 0x7c { return Err(anyhow!("unexpected vertex list address {:x}", vlist_addr)); }
            let r1 = cursor.read_u32::<LittleEndian>()?;
            if r1 != vlist_addr + (12 * num_vertices as u32) { return Err(anyhow!("unexpected r1 {:x}", r1)); }
            let r2 = cursor.read_u32::<LittleEndian>()?;
            if r2 != r1 + (8 * num_uv_coords as u32) { return Err(anyhow!("unexpected r2 {:x}", r2)); }
            let r3 = cursor.read_u32::<LittleEndian>()?;
            if r3 != r2 + (40 * num_faces as u32) { return Err(anyhow!("unexpected r3 {:x}", r3)); }
            let mut vertices = vec![ Qfg5Vertex::default(); num_vertices ];
            for n in 0..num_vertices {
                vertices[n].x = cursor.read_f32::<LittleEndian>()?;
                vertices[n].y = cursor.read_f32::<LittleEndian>()?;
                vertices[n].z = cursor.read_f32::<LittleEndian>()?;
            }
            let mut texcoords = vec![ Qfg5TexCoord::default(); num_uv_coords ];
            for n in 0..num_uv_coords {
                texcoords[n].u = cursor.read_f32::<LittleEndian>()?;
                texcoords[n].v = cursor.read_f32::<LittleEndian>()?;
            }
            let mut faces = vec![ Qfg5Face::default(); num_faces ];
            for n in 0..num_faces {
                faces[n].vertex1 = cursor.read_u32::<LittleEndian>()? as usize;
                faces[n].vertex2 = cursor.read_u32::<LittleEndian>()? as usize;
                faces[n].vertex3 = cursor.read_u32::<LittleEndian>()? as usize;
                faces[n].uv1 = cursor.read_u32::<LittleEndian>()? as usize;
                faces[n].uv2 = cursor.read_u32::<LittleEndian>()? as usize;
                faces[n].uv3 = cursor.read_u32::<LittleEndian>()? as usize;
                faces[n].subbitmap = cursor.read_u32::<LittleEndian>()? as usize;
                faces[n].normal_x = cursor.read_f32::<LittleEndian>()?;
                faces[n].normal_y = cursor.read_f32::<LittleEndian>()?;
                faces[n].normal_z = cursor.read_f32::<LittleEndian>()?;
            }
            let mut lighting_vertices = vec![ Qfg5LightingVertex::default(); num_vertices ];
            for n in 0..num_vertices {
                lighting_vertices[n].a = cursor.read_f32::<LittleEndian>()?;
                lighting_vertices[n].b = cursor.read_f32::<LittleEndian>()?;
                lighting_vertices[n].c = cursor.read_f32::<LittleEndian>()?;
                lighting_vertices[n].d = cursor.read_f32::<LittleEndian>()?;
            }
            submeshes.push(SubMesh{ name, vertices, texcoords, faces, lighting_vertices });
        }

        cursor.seek(SeekFrom::Start(bitmap_texture_offset))?;
        let mut num_subbitmaps = cursor.read_u32::<LittleEndian>()? as usize;
        if (num_subbitmaps & 3) != 0 { return Err(anyhow!("corrupt number of subbitmaps {:x}", num_subbitmaps)); }
        num_subbitmaps = num_subbitmaps / 4;
        if num_subbitmaps > 1 {
            println!("Note: >1 subbitmaps: {}", num_subbitmaps);
            cursor.seek(SeekFrom::Current(((num_subbitmaps - 1) * 4) as i64))?;
        }

        let mut subbitmaps = Vec::with_capacity(num_subbitmaps);
        for n in 0..num_subbitmaps {
            let width = cursor.read_f32::<LittleEndian>()?;
            let height = cursor.read_f32::<LittleEndian>()?;
            let width_pow_2 = cursor.read_u32::<LittleEndian>()?;
            let height_pow_2 = cursor.read_u32::<LittleEndian>()?;
            let width_minus_1 = cursor.read_u32::<LittleEndian>()?;
            let height_minus_1 = cursor.read_u32::<LittleEndian>()?;
            if (width_minus_1 + 1) != width as u32 { return Err(anyhow!("subbitmap {} width corrupt: {} and {}", n, width_minus_1, width)); }
            if (height_minus_1 + 1) != height as u32 { return Err(anyhow!("subbitmap {} height corrupt: {} and {}", n, height_minus_1, height)); }
            if 1 << width_pow_2 != width as u32 { return Err(anyhow!("subbitmap {} 2-pow-height corrupt: {} and {}", n, width_pow_2, width)); }
            if 1 << height_pow_2 != height as u32 { return Err(anyhow!("subbitmap {} 2-pow-height corrupt: {} and {}", n, height_pow_2, height)); }
            let width = width_minus_1 + 1;
            let height = height_minus_1 + 1;

            let mut bitmap = vec![ 0u8; (width * height) as usize ];
            cursor.read_exact(&mut bitmap)?;
            subbitmaps.push(SubBitmap{ width, height, bitmap });
        }
        Ok(Qfg5Model{ name, palette, submeshes, subbitmaps })
    }
}
