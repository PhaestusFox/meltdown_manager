struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: u32,
};

struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) block_type: u32,
    @location(3) scale: vec3<f32>,
    @location(4) vertex_normal: u32,
    @location(5) chunk_pos: vec3<u32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

const CHUNK_SIZE: f32 = f32(CHUNK_SIZE_U32);
const CHUNK_SIZE_U32: u32 = 10;

// we can import items from shader modules in the assets folder with a quoted path
const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.);

@group(2) @binding(0) var<uniform> atlas_size: vec2<u32>;
@group(2) @binding(1) var material_color_texture: texture_2d<f32>;
@group(2) @binding(2) var material_color_sampler: sampler;
@group(2) @binding(3) var<uniform> face_overrides: array<FaceOverride, 256 / 4>;
// @group(2) @binding(4) var<uniform> data: array<u32>;
@group(2) @binding(4) var<storage, read> data: array<FaceOverride, (CHUNK_SIZE_U32 * CHUNK_SIZE_U32 * CHUNK_SIZE_U32) / 8>;
@group(2) @binding(5) var<uniform> flags: u32;

struct FaceOverride {
    block_a: u32,
    block_b: u32,
    block_c: u32,
    block_d: u32,
}

const light: vec3<f32> = vec3(-0.57735027, 0.57735027, 0.57735027);

const r30: f32 = 1.0 / 29.0;
const rc: f32 = 1.0 / CHUNK_SIZE;
const r255: f32 = 1.0 / 255.0;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let world_normal = normalize( cross( dpdy( in.world_position.xyz ), dpdx( in.world_position.xyz ) ) );
    var dp = 0.7;

    if world_normal.y > 0.2 {
        dp = 1.;
    } else if world_normal.y < -0.2 {
        dp = 0.5;
    } else {
        if world_normal.x > 0.2 {
            dp += 0.05;   
        } else if world_normal.x < -0.2 {
            dp -= 0.1;
        }
        if world_normal.z > 0.2 {
            dp += 0.1;
        } else if world_normal.z < -0.2 {
            dp -= 0.05;
        }
    }

    // back, left, right, top, bottom
    var face: u32 = 10;
    if world_normal.x > 0.5 {
        face = 15;
    } else if world_normal.x < -0.5 {
        face = 10;
    } else if world_normal.y > 0.5 {
        face = 5;
    } else if world_normal.y < -0.5 {
        face = 0;
    } else if world_normal.z > 0.5 {
        face = 20;
    } else if world_normal.z < -0.5 {
        face = 25;
    };

    let faceovers = face_overrides[in.block_type / 4];
    var faceover: u32;
    let index = in.block_type % 4;
    if index == 0 {
        faceover = faceovers.block_a;
    } else if index == 1 {
        faceover = faceovers.block_b;
    } else if index == 2 {
        faceover = faceovers.block_c;
    } else if index == 3 {
        faceover = faceovers.block_d;
    } else {
        faceover = 0;
    }
    var stride = (faceover >> face) & 31;
    var block_type = in.block_type + stride;

    let x = (block_type) % atlas_size.x;
    let y = (block_type) / atlas_size.y;
    var uvx = f32(x) /  f32(atlas_size.x);
    var uvy = (1.+f32(y)) /  f32(atlas_size.x);
    
    let texture_step = 1. / vec2<f32>(atlas_size);

    
    var axis: f32;
    if abs(world_normal.y) < 0.5 {
        axis = (in.world_position.y * in.scale.y) % 1;
    } else {
        axis = (in.world_position.z * in.scale.z) % 1;
    };
    if axis < 0 {
        axis += 1;
    }
    uvy -= (axis * texture_step.y);
    
    if abs(world_normal.x) < 0.5 {
        axis = (in.world_position.x * in.scale.x) % 1;
    } else {
        axis = (in.world_position.z * in.scale.z) % 1;
    };
    if axis < 0 {
        axis += 1;
    }

    var fx = in.world_position.x % CHUNK_SIZE;
    if fx < 0 {
        fx += CHUNK_SIZE;
    }
    var fy = in.world_position.y % CHUNK_SIZE;
    if fy < 0 {
        fy += CHUNK_SIZE;
    }
    var fz = in.world_position.z % CHUNK_SIZE;
    if fz < 0 {
        fz += CHUNK_SIZE;
    }
    let ix = u32(fx);
    let iy = u32(fy);
    let iz = u32(fz);
    let aindex = in.chunk_pos.x + in.chunk_pos.z * CHUNK_SIZE_U32 + in.chunk_pos.y * CHUNK_SIZE_U32 * CHUNK_SIZE_U32;
    let ai = i32(aindex % 8);

    let autos = data[aindex / 8];
    var adata: u32;
    
    switch ai {
        case 1: {adata = autos.block_a >> 16;}
        case 2: {adata = autos.block_b;}
        case 3: {adata = autos.block_b >> 16;}
        case 4: {adata = autos.block_c;}
        case 5: {adata = autos.block_c >> 16;}
        case 6: {adata = autos.block_d;}
        case 7: {adata = autos.block_d >> 16;}
        default: {adata = autos.block_a;}
    }

    let temp = f32(adata & 0xFF) * r255;
    let phase = (adata >> 8) & 0xFF;

    uvx += axis * texture_step.x;
    var ts = textureSample(material_color_texture, material_color_sampler, vec2(uvx, uvy));
    let a = ts.a;
    ts = vec4(0);
    ts.r = temp * rc;

    if (flags & (1<<9)) > 0 {
        if phase == 0 {
            ts.b = 1.;
        }
        if (phase & 1) > 0 {
            ts.g = 1.;
        }
        if (phase & 2) > 0 {
            ts.r = 1.;
        }
    } else if (flags & (1<<8)) > 0 {
        ts.r = f32(in.chunk_pos.x) * rc;
        ts.b = f32(in.chunk_pos.z) * rc;
        ts.g = f32(in.chunk_pos.y) * rc;
    }

    // ts = vec4<f32>(0.0);
    
    // if (in.vertex_normal & 4) > 0 {
    //     ts.b = 1.;
    // } else {
    //     ts.b = 0.5;
    // }

    ts *= dp * COLOR_MULTIPLIER;
    if a < 0.2 {
        discard;
    } else {
        ts.a = a;
    };
    // return vec4(1.);
    return ts;
    // discard;
}

#import bevy_pbr::{
    in_bindings::in,
    mesh_functions,
    skinning,
    morph::morph,
    view_transformations::position_world_to_clip,
}

fn affine3_to_square(affine: mat3x4<f32>) -> mat4x4<f32> {
    return transpose(mat4x4<f32>(
        affine[0],
        affine[1],
        affine[2],
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    ));
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let in_world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);

    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = in_world_from_local;
    let x = vertex.position & 31;
    let y = (vertex.position >> 5) & 31;
    let z = (vertex.position >> 10) & 31;
    out.block_type = (vertex.position >> 15) & 255;
    let n = (vertex.position >> 23) & 7;
    var pos = vec3(f32(x), f32(y), f32(z));
    // out.vertex_normal = vec3(0);
    out.chunk_pos = vec3<u32>(x, y, z);
    out.vertex_normal = n;
    if (n & 4) > 0 { // is right face
        out.chunk_pos.x -= 1;
    }
    if (n & 2) > 0 { // is top face
        out.chunk_pos.y -= 1;
    }
    if (n & 1) == 0 {
        out.chunk_pos.z -= 1;
    }



    // calculate scale
    out.scale = vec3<f32>(1.);
    // 1. calculate the determinant of the affine matrix
    // determinant = dot(z, cross(x, y))
    let determinant = determinant(in_world_from_local);
    // 2. x = length of the first column of the affine matrix
    out.scale.x = 1. / length(in_world_from_local[0]);
    // 3. is the determinant negative? if so, negate the x of the scale
    if determinant < 0. {
        out.scale.x = -out.scale.x;
    }
    // 4. y = length of the second column of the affine matrix
    out.scale.y = 1. / length(in_world_from_local[1]);
    // 5. z = length of the third column of the affine matrix
    out.scale.z = 1. / length(in_world_from_local[2]);
    // let scale = vec3<f32>(det, det, det);


    /// set pos
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(pos, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    /// end set pos

    return out;
}