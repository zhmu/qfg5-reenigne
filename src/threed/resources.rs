/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use wgpu::util::DeviceExt;

use crate::threed::{model, texture};
use crate::qfg5resource::qfg5mdl;

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let data = std::fs::read_to_string(std::path::Path::new(file_name))?;
    Ok(data)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let data = std::fs::read(std::path::Path::new(file_name))?;
    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_qfg5model(
    model: &qfg5mdl::Qfg5Model,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let mut materials = Vec::new();
    for (n, subbitmap) in model.subbitmaps.iter().enumerate() {
        let name = format!("subbitmap-{}", n);
        let texture = texture::Texture::from_palettized_raw(device, queue, &subbitmap.bitmap, &model.palette, subbitmap.height, subbitmap.width, name.as_str())?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(model::Material {
            name,
            diffuse_texture: texture,
            bind_group,
        })
    }

    let mut meshes: Vec<model::Mesh> = Vec::new();
    for (submesh_index, submesh) in model.submeshes.iter().enumerate() {

        let mut vertices: Vec<model::Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for face in &submesh.faces {
            // Every face is a triangle
            let v1 = &submesh.vertices[face.vertex1];
            let v2 = &submesh.vertices[face.vertex2];
            let v3 = &submesh.vertices[face.vertex3];
            let uv1 = &submesh.texcoords[face.uv1];
            let uv2 = &submesh.texcoords[face.uv2];
            let uv3 = &submesh.texcoords[face.uv3];
            // TODO: Is this correct? We should have normals _per vertex_, not per _face_ ... ?
            let nx = face.normal_x;
            let ny = face.normal_y;
            let nz = face.normal_z;
            vertices.push(model::Vertex {
                position: [ v1.x, v1.y, v1.z ],
                tex_coords: [ uv1.u, uv1.v ],
                normal: [ nx, ny, nz ]
            });
            indices.push((vertices.len() - 1) as u32);
            vertices.push(model::Vertex {
                position: [ v2.x, v2.y, v2.z ],
                tex_coords: [ uv2.u, uv2.v ],
                normal: [ nx, ny, nz ]
            });
            indices.push((vertices.len() - 1) as u32);
            vertices.push(model::Vertex {
                position: [ v3.x, v3.y, v3.z ],
                tex_coords: [ uv3.u, uv3.v ],
                normal: [ nx, ny, nz ]
            });
            indices.push((vertices.len() - 1) as u32);
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer {}", model.name, submesh_index)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer {}", model.name, submesh_index)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        meshes.push(model::Mesh{
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0, // TODO face.subbitmap
        });
    }

    Ok(model::Model { meshes, materials })
}
