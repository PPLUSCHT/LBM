use wgpu::{CommandEncoder, util::BufferInitDescriptor, BufferUsages, ShaderStages,BindGroupDescriptor};
use std::{mem, borrow::Cow};
use wgpu::{Device, BindGroupEntry, util::DeviceExt, BindGroupLayout, ShaderModuleDescriptor, vertex_attr_array, VertexBufferLayout};

use crate::{driver::Driver, barrier_shapes::{Shape, merge_shapes::get_points_vector, blob::Blob, line::Line}};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(PartialEq, Clone, Copy)]
pub enum SummaryStat {
    Curl,
    Ux,
    Uy, 
    Rho,
    Speed
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Copy)]
pub enum ColorMap {
    Inferno,
    Viridis,
    Jet,
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Copy)]
pub enum Preset{
    Welcome
}

pub struct LBM{
    //Bind Groups
    pub collide_bg: wgpu::BindGroup,
    pub output_bg: wgpu::BindGroup,
    pub density_bg: wgpu::BindGroup,
    pub dimension_bg: wgpu::BindGroup,
    pub dimension_bg_vertex: wgpu::BindGroup,
    pub size_bg: wgpu::BindGroup,
    pub barrier_bg: wgpu::BindGroup,
    pub color_bg: wgpu::BindGroup,

    //Directional BGS
    pub ne_sw_bgs: Vec<wgpu::BindGroup>,
    pub nw_se_bgs: Vec<wgpu::BindGroup>,
    pub n_s_bgs: Vec<wgpu::BindGroup>,
    pub e_w_bgs: Vec<wgpu::BindGroup>,

    //needed Buffers
    data_buffers: Vec<Vec<wgpu::Buffer>>,
    barrier_buffer: wgpu::Buffer,
    omega_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,

    //Compute Pipelines
    cardinal_pre_collision: wgpu::ComputePipeline,
    corner_pre_collision: wgpu::ComputePipeline,
    corner_collide: wgpu::ComputePipeline,
    cardinal_collide: wgpu::ComputePipeline,
    e_w_stream: wgpu::ComputePipeline,
    n_s_stream: wgpu::ComputePipeline,
    ne_sw_stream: wgpu::ComputePipeline,
    nw_se_stream: wgpu::ComputePipeline,

    //Summary/ColorMap Pipelines
    curl: wgpu::ComputePipeline,
    ux: wgpu::ComputePipeline,
    uy: wgpu::ComputePipeline,
    rho: wgpu::ComputePipeline,
    speed: wgpu::ComputePipeline,
    pub color_map: ColorMap,
    viridis: wgpu::ComputePipeline,
    jet: wgpu::ComputePipeline,
    inferno: wgpu::ComputePipeline,
    summary_stat: SummaryStat,

    //Render Pipeline
    render: wgpu::RenderPipeline,

    //Barrier Update Pipelines
    barrier_draw: wgpu::ComputePipeline,

    //Barrier BGS
    draw_bg: wgpu::BindGroup,

    //Update Barrier Buffers
    draw_num: wgpu::Buffer,
    draw_points: wgpu::Buffer,

    //tracking variables
    pub compute_step: usize,
    frame_number: usize,
    work_group_size: usize,

    x: u32,
    y: u32,

}

impl LBM{
    
    fn calculate_work_group_size(x:u32, y:u32) -> usize{
        let x_dim = x as f32;
        let y_dim = y as f32;
        (x_dim * y_dim/256.0).ceil() as usize
    }

    fn create_vertex_buffer(driver: &Driver, x: u32, y:u32) -> wgpu::Buffer{
        driver.device.create_buffer_init(&BufferInitDescriptor{
             label: None,
             contents: bytemuck::cast_slice(&[-1.0, 1.0, -1.0, 1.0 - 2.0/y as f32, -1.0 + 2.0/x as f32, 1.0 - 2.0/y as f32, -1.0, 1.0, -1.0 + 2.0/x as f32, 1.0, -1.0 + 2.0/x as f32, 1.0 - 2.0/y as f32]),
             usage: wgpu::BufferUsages::VERTEX
         })
     }

    fn create_data_pair_bgl(device : &Device, x: u32, y:u32) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor 
        {   entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<f32>()) as _,) 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<f32>()) as _,) 
                    },
                    count: None,
                }
            ],
            label: None
        })
    }

    fn create_data_triple_bgl(device : &Device, x: u32, y:u32) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor 
        {  
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<f32>()) as _,) 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<f32>()) as _,) 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<f32>()) as _,) 
                    },
                    count: None,
                }
            ],
            label: None
        })
    }

    fn create_color_bgl(device : &Device, x: u32, y:u32) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor 
            {  
                entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                            has_dynamic_offset: false, 
                            min_binding_size: wgpu::BufferSize::new((3 * x as usize * y as usize * mem::size_of::<f32>()) as _,) 
                        },
                        count: None,
                    }
                ],
                label: None
        })
    }

    fn create_color_bg(device : &Device, color_bgl: &wgpu::BindGroupLayout, x: u32, y:u32) -> wgpu::BindGroup{
        let color_vec = vec![0.0; 3 * x as usize * y as usize];
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&color_vec),
            usage: wgpu::BufferUsages::STORAGE,
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor{ 
            label: None, 
            layout: &color_bgl, 
            entries: &[wgpu::BindGroupEntry{
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }] 
        })
    }

    fn create_data_single_bgl(device : &Device, x: u32, y:u32) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<f32>()) as _,) 
                    },
                    count: None,
            }],
            label: None
        })
    }

    fn create_barrier_bgl(device : &Device, x: u32, y:u32) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<u32>()) as _,) 
                    },
                    count: None,
            }],
            label: None
        })
    }

    fn create_collide_bgl(
        device : &Device, x: u32, y:u32
    ) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{ 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((mem::size_of::<u32>()) as _,) 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((mem::size_of::<f32>()) as _,) 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<f32>()) as _,) 
                    },
                    count: None,
                }
            ] 
        })
    }

    fn create_collide_bg(
        device : &Device, 
        origin: &wgpu::Buffer, 
        omega_buffer: &wgpu::Buffer,
        size_buffer: &wgpu::Buffer,
        collide_bgl: &wgpu::BindGroupLayout
        ) -> wgpu::BindGroup{

        device.create_bind_group(&wgpu::BindGroupDescriptor{ 
            label: None, 
            layout: &collide_bgl, 
            entries: &[
                BindGroupEntry{
                    binding: 0,
                    resource: size_buffer.as_entire_binding(),
                }, 
                BindGroupEntry{
                    binding: 1,
                    resource: omega_buffer.as_entire_binding(),
                },
                BindGroupEntry{
                    binding: 2, 
                    resource: origin.as_entire_binding(),
                }
            ]
        })
    }

    fn create_size_bgl(device : &Device) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{ 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((mem::size_of::<u32>()) as _,) 
                    },
                    count: None,
                }
            ] 
        })
    }

    fn create_size_bg(device : &Device, size_buffer: &wgpu::Buffer, size_bgl: &BindGroupLayout) -> wgpu::BindGroup{
        device.create_bind_group(&wgpu::BindGroupDescriptor{ 
            label: None, 
            layout: size_bgl, 
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: size_buffer.as_entire_binding(),
                }
            ]
        })
    }

    fn create_dimension_bgl(device : &Device) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{ 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((3 * mem::size_of::<u32>()) as _,) 
                    },
                    count: None,
                }
            ] 
        })
    }

    fn create_dimension_bg(device : &Device, bgl: &BindGroupLayout, x: u32, y:u32) -> wgpu::BindGroup{
        let dimension_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&[x, y, x * y]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor{ 
            label: None, 
            layout: bgl, 
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: dimension_buffer.as_entire_binding(),
                }
            ]
        })
    }

    fn create_vertex_dimension_bgl(device : &Device) -> wgpu::BindGroupLayout{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{ 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new((3 * mem::size_of::<u32>()) as _,) 
                    },
                    count: None,
                }
            ] 
        })
    }

    fn create_vertex_dimension_bg(device : &Device, bgl: &BindGroupLayout, x: u32, y:u32) -> wgpu::BindGroup{
        
        let dimension_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&[x, y, x * y]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::VERTEX,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor{ 
            label: None, 
            layout: bgl, 
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: dimension_buffer.as_entire_binding(),
                }
            ]
        })
    }

    fn create_omega_buffer(device : &Device, omega: f32) -> wgpu::Buffer{
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::bytes_of(&omega),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_size_buffer(device : &Device, x: u32, y:u32) -> wgpu::Buffer{
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::bytes_of(&(x * y)),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }

    fn create_pre_collision_pl(device : &Device, 
                               data_pair: &wgpu::BindGroupLayout, 
                               data_triple: &wgpu::BindGroupLayout,
                               size: &wgpu::BindGroupLayout
                            ) -> wgpu::PipelineLayout{
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{ 
            label: None, 
            bind_group_layouts: &[data_pair, data_pair, data_triple, size], 
            push_constant_ranges: &[]
        })
    }

    fn create_collision_pl(device : &Device,
        data_pair: &wgpu::BindGroupLayout,
        data_triple: &wgpu::BindGroupLayout,
        collide_bgl: &wgpu::BindGroupLayout
    ) -> wgpu::PipelineLayout{
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{ 
            label: None, 
            bind_group_layouts: &[data_pair, data_pair, data_triple, collide_bgl], 
            push_constant_ranges: &[] 
        })
    }

    fn create_stream_pl(
        device : &Device,
        dimensions: &wgpu::BindGroupLayout,
        data_pair: &wgpu::BindGroupLayout,
        barrier: &wgpu::BindGroupLayout,
    ) -> wgpu::PipelineLayout{
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{ 
            label: None, 
            bind_group_layouts: &[dimensions, data_pair, data_pair, barrier], 
            push_constant_ranges: &[] 
        })
    }

    fn create_summary_pl(
        device : &Device,
        dimensions: &wgpu::BindGroupLayout,
        data_triple: &wgpu::BindGroupLayout,
        data_single: &wgpu::BindGroupLayout,
    ) -> wgpu::PipelineLayout{
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{ 
            label: None, 
            bind_group_layouts: &[dimensions, data_triple, data_single], 
            push_constant_ranges: &[]
        })
    }

    fn create_color_map_pl(
        device : &Device,
        data_single: &wgpu::BindGroupLayout,
        color: &wgpu::BindGroupLayout,
        barrier: &wgpu::BindGroupLayout,
        size: &wgpu::BindGroupLayout
    ) -> wgpu::PipelineLayout{
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{ 
            label: None, 
            bind_group_layouts: &[color, data_single, barrier, size], 
            push_constant_ranges: &[] 
        })
    }

    fn create_compute_pipeline(
        device : &Device,
        module: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout
    ) -> wgpu::ComputePipeline{
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{ 
            label: None, 
            layout: Some(layout), 
            module: module, 
            entry_point: "main"
        })
    }

    fn create_data_buffers(device : &Device, data: &Vec<Vec<f32>>) -> Vec<wgpu::Buffer>{
        data.iter().map(|x| device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&x),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        }))
        .collect()
    }

    fn create_data_bg_from_buffers(device : &Device, 
        buffers: &Vec<&wgpu::Buffer>,
        data_bgl: &wgpu::BindGroupLayout) -> wgpu::BindGroup{
        let mut data_entry = Vec::<wgpu::BindGroupEntry>::new();
        for i in 0..buffers.len(){
            data_entry.push(wgpu::BindGroupEntry{binding: i as u32, resource: buffers[i].as_entire_binding()})
        }
        device.create_bind_group(&wgpu::BindGroupDescriptor{ 
            label: None, 
            layout: data_bgl, 
            entries: &data_entry
        })
    }

    fn create_data_bg<const N:usize>(
        device : &Device,
        data: &[&Vec<f32>; N],
        data_bgl: &wgpu::BindGroupLayout
    ) -> wgpu::BindGroup{
        let data:[wgpu::Buffer; N] = data
            .iter()
            .map(|x| device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
                label: None,
                contents: bytemuck::cast_slice(&x),
                usage: wgpu::BufferUsages::STORAGE,
            }))
            .collect::<Vec<wgpu::Buffer>>()
            .try_into()
            .unwrap();
        let mut data_entry = vec![wgpu::BindGroupEntry{binding: 0 as u32, resource: data[0].as_entire_binding()}; N];
        for i in 1..N{
            data_entry[i] = wgpu::BindGroupEntry{binding: i as u32, resource: data[i].as_entire_binding()}
        }
        device.create_bind_group(&wgpu::BindGroupDescriptor{ 
            label: None, 
            layout: data_bgl, 
            entries: &data_entry
        })
    }

    fn create_barrier_buffer(
        barrier: &Vec<u32>,
        device : &Device,
    ) -> wgpu::Buffer{ 
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(barrier),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_barrier_bg(
        device : &Device,
        barrier: &wgpu::Buffer,
        data_single_bgl: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup{
        device.create_bind_group(&wgpu::BindGroupDescriptor{ 
            label: None, 
            layout: data_single_bgl, 
            entries: &[wgpu::BindGroupEntry{
                binding: 0,
                resource: barrier.as_entire_binding(),
            }] 
        })
    }

    fn init_barrier(x: u32, y: u32) -> Vec<u32>{
        
        let mut border_vec = vec![0_u32; x as usize * y as usize];

        for i in 0..x{
            border_vec[Self::index_pre_init(i, 0, x) as usize] = 1;
            border_vec[Self::index_pre_init(i, y - 1, x) as usize] = 1;
        }

        border_vec
    }

    fn index_pre_init(x: u32, y:u32, max_x: u32) -> u32{
        x + y * max_x
    }

    fn set_equil(mut ux: f32, mut uy: f32, rho: f32, x: u32, y: u32) -> Vec<Vec<f32>>{
    
        let mut ux_2 = ux * ux;
        let mut uy_2 = uy * uy;
        let mut u_dot_product = ux_2 + uy_2;
        let mut u_sum_sq_pos = u_dot_product + 2.0 * (ux * uy);
        let mut u_sum_sq_neg = u_dot_product - 2.0 * (ux * uy);
    
        ux *= 3.0;
        uy *= 3.0;
        ux_2 *= 4.5;
        uy_2 *= 4.5;
        u_dot_product *= 1.5;
        u_sum_sq_neg *= 4.5;
        u_sum_sq_pos *= 4.5;
    
        let rho_ninth = rho/9.0_f32;
        let rho_36th = rho/36.0_f32;
    
        let mut vec = Vec::with_capacity(9);
    
        vec.push(vec![rho_36th * (1.0 - ux + uy + u_sum_sq_neg - u_dot_product); x as usize * y as usize]);
        vec.push(vec![rho_ninth * (1.0 + uy + uy_2 - u_dot_product);  x as usize * y as usize]);
        vec.push(vec![rho_36th * (1.0 + ux + uy + u_sum_sq_pos - u_dot_product);  x as usize * y as usize]);
        vec.push(vec![rho_ninth * (1.0 - ux + ux_2 - u_dot_product);  x as usize * y as usize]);
        vec.push(vec![4.0 * rho_ninth * (1.0 - u_dot_product);  x as usize * y as usize]);
        vec.push(vec![rho_ninth * (1.0 + ux + ux_2 - u_dot_product);  x as usize * y as usize]);
        vec.push(vec![rho_36th * (1.0 - ux - uy + u_sum_sq_pos - u_dot_product);  x as usize * y as usize]);
        vec.push(vec![rho_ninth * (1.0 - uy - uy_2 - u_dot_product);  x as usize * y as usize]);
        vec.push(vec![rho_36th * (1.0 + ux - uy + u_sum_sq_neg - u_dot_product);  x as usize * y as usize]);
        vec
        
    }

    fn create_render_pipeline(driver: &Driver,
                              colors: &BindGroupLayout,
                              dimension_params: &BindGroupLayout
                            ) -> wgpu::RenderPipeline{

        let render_shader = driver.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/render.wgsl"))),
        });

        let render_pipeline_layout = driver.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&colors, &dimension_params],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = driver.surface.get_capabilities(&driver.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        driver.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout{
                    array_stride: 4 * 2,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    fn create_barrier_update_bgl(driver: &Driver, x: u32, y: u32) -> wgpu::BindGroupLayout{
        driver.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{ 
            label: None, 
            entries: &[
            wgpu::BindGroupLayoutEntry{
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer{ 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: wgpu::BufferSize::new((1 * mem::size_of::<u32>()) as _,)
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry{
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer{ 
                    ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                    has_dynamic_offset: false, 
                    min_binding_size: wgpu::BufferSize::new((x as usize * y as usize * mem::size_of::<u32>()) as _,)
                },
                count: None,
            }
            ]
        })
    }

    fn create_barrier_update_pl(driver: &Driver, 
        update_bgl: &wgpu::BindGroupLayout, 
        barrier_bgl: &wgpu::BindGroupLayout) -> wgpu::PipelineLayout{
        driver.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{ 
            label: None, 
            bind_group_layouts: &[update_bgl, barrier_bgl], 
            push_constant_ranges: &[]
        })
    }

    pub fn new(driver: &Driver, omega: f32, x: u32, y:u32) -> LBM{
        
        //Create Bindgroup Layouts
        let data_single_bgl = Self::create_data_single_bgl(&driver.device, x, y);
        let data_pair_bgl = Self::create_data_pair_bgl(&driver.device, x, y);
        let data_triple_bgl = Self::create_data_triple_bgl(&driver.device, x, y);
        let collide_bgl = Self::create_collide_bgl(&driver.device, x, y);
        let color_bgl = Self::create_color_bgl(&driver.device, x, y);
        let size_bgl = Self::create_size_bgl(&driver.device);
        let dimension_bgl = Self::create_dimension_bgl(&driver.device);
        let dimension_vertex_bgl = Self::create_vertex_dimension_bgl(&driver.device);
        let barrier_bgl = Self::create_barrier_bgl(&driver.device, x, y);

        //Create Initial Conditions
        let init_data = Self::set_equil(0.1, 0.0, 1.0, x, y);
        let mut data_buffers = Vec::<Vec<wgpu::Buffer>>::new();
        
        for _ in 0..2{
            data_buffers.push(Self::create_data_buffers(&driver.device, &init_data));
        }

        //Create Needed Buffers
        let barrier_vec = Self::init_barrier(x, y);
        let barrier_buffer = Self::create_barrier_buffer(&barrier_vec, &driver.device);
        let omega_buffer = Self::create_omega_buffer(&driver.device , omega);
        let size_buffer = Self::create_size_buffer(&driver.device, x, y);

        //Create Bindgroups
        let mut ne_sw_bgs = Vec::<wgpu::BindGroup>::with_capacity(2);
        let mut nw_se_bgs = Vec::<wgpu::BindGroup>::with_capacity(2);
        let mut n_s_bgs = Vec::<wgpu::BindGroup>::with_capacity(2);
        let mut e_w_bgs = Vec::<wgpu::BindGroup>::with_capacity(2);

        for i in 0..2{
            ne_sw_bgs.push(Self::create_data_bg_from_buffers(&driver.device, 
                &vec![&data_buffers[i][2], &data_buffers[i][6]], 
            &data_pair_bgl));
            nw_se_bgs.push(Self::create_data_bg_from_buffers(&driver.device, 
            &vec![&data_buffers[i][0], &data_buffers[i][8]], 
        &data_pair_bgl));
            n_s_bgs.push(Self::create_data_bg_from_buffers(&driver.device, 
            &vec![&data_buffers[i][1], &data_buffers[i][7]], 
        &data_pair_bgl));
            e_w_bgs.push(Self::create_data_bg_from_buffers(&driver.device, 
            &vec![&data_buffers[i][5], &data_buffers[i][3]], 
            &data_pair_bgl));
        }

        let zero_vec = vec![0.0; x as usize * y as usize];
        let collide_bg = Self::create_collide_bg(&driver.device, &data_buffers[0][4], 
            &omega_buffer, 
            &size_buffer,
            &collide_bgl);
        let density_bg = Self::create_data_bg(&driver.device, 
            &[&zero_vec, &zero_vec, &zero_vec], 
            &data_triple_bgl);
        let output_bg = Self::create_data_bg(&driver.device, 
            &[&zero_vec], 
            &data_single_bgl);
        let color_bg = Self::create_color_bg(&driver.device, &color_bgl, x, y);
        let size_bg = Self::create_size_bg(&driver.device, &size_buffer, &size_bgl);
        let dimension_bg = Self::create_dimension_bg(&driver.device, &dimension_bgl, x, y);
        let vertex_dimension_bg = Self::create_vertex_dimension_bg(&driver.device, &dimension_vertex_bgl, x, y);
        let barrier_bg = Self::create_barrier_bg(&driver.device, 
            &barrier_buffer, 
            &barrier_bgl);

        //Create Pipeline Layouts
        let pre_collision_pl = Self::create_pre_collision_pl(&driver.device, 
            &data_pair_bgl, 
            &data_triple_bgl, 
            &size_bgl);

        let collision_pl = Self::create_collision_pl(&driver.device, 
            &data_pair_bgl, 
            &data_triple_bgl, 
            &collide_bgl);

        let stream_pl = Self::create_stream_pl(&driver.device, 
            &dimension_bgl, 
            &data_pair_bgl, 
            &barrier_bgl);

        let summary_pl = Self::create_summary_pl(&driver.device, 
            &dimension_bgl, 
            &data_triple_bgl, 
            &data_single_bgl);

        let color_map_pl = Self::create_color_map_pl(&driver.device,
            &data_single_bgl,
            &color_bgl,
            &barrier_bgl,
            &size_bgl
        );

        //Create all shader modules
        let corner_pre_collision_s = driver.device.create_shader_module(ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/pre_collision/corner_pre_collision.wgsl")))
        });

        let cardinal_pre_collision_s = driver.device.create_shader_module(ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/pre_collision/cardinal_pre_collision.wgsl")))
        });

        let cardinal_collision_s = driver.device.create_shader_module(ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/collision/cardinal_collision.wgsl")))
        });

        let corner_collision_s = driver.device.create_shader_module(ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/collision/corner_collision.wgsl")))
        });

        let ne_sw_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/stream/ne_sw_stream.wgsl")))
        });

        let nw_se_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/stream/se_nw_stream.wgsl")))
        });

        let n_s_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/stream/n_s_stream.wgsl")))
        });

        let e_w_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/stream/e_w_stream.wgsl")))
        });

        let ux_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/summary_stats/ux.wgsl")))
        });

        let uy_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/summary_stats/uy.wgsl")))
        });

        let rho_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/summary_stats/rho.wgsl")))
        });

        let speed_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/summary_stats/speed.wgsl")))
        });

        let curl_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/summary_stats/curl.wgsl")))
        });

        let inferno_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/color_map/inferno.wgsl")))
        });

        let viridis_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/color_map/viridis.wgsl")))
        });

        let jet_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/color_map/jet.wgsl")))
        });

        let corner_pre_collision = Self::create_compute_pipeline(&driver.device, 
            &corner_pre_collision_s, 
            &pre_collision_pl);
        let cardinal_pre_collision = Self::create_compute_pipeline(&driver.device, 
            &cardinal_pre_collision_s, 
            &pre_collision_pl);
        
        let corner_collision = Self::create_compute_pipeline(&driver.device, 
            &corner_collision_s, 
            &collision_pl);
        let cardinal_collision = Self::create_compute_pipeline(&driver.device, 
            &cardinal_collision_s, 
            &collision_pl);
        
        let e_w_stream = Self::create_compute_pipeline(&driver.device, 
            &e_w_s, 
            &stream_pl);
        let n_s_stream = Self::create_compute_pipeline(&driver.device, 
            &n_s_s, 
            &stream_pl);
        let ne_sw_stream = Self::create_compute_pipeline(&driver.device, 
            &ne_sw_s, 
            &stream_pl);
        let nw_se_stream = Self::create_compute_pipeline(&driver.device, 
            &nw_se_s, 
            &stream_pl);

        let curl = Self::create_compute_pipeline(&driver.device, 
            &curl_s, 
            &summary_pl);
        
        let ux = Self::create_compute_pipeline(&driver.device, 
            &ux_s, 
            &summary_pl);

        let uy = Self::create_compute_pipeline(&driver.device, 
            &uy_s, 
            &summary_pl);

        let rho = Self::create_compute_pipeline(&driver.device, 
            &rho_s, 
            &summary_pl);

        let speed = Self::create_compute_pipeline(&driver.device, 
            &speed_s, 
            &summary_pl);

        let viridis = Self::create_compute_pipeline(&driver.device, 
            &viridis_s, 
            &color_map_pl);

        let inferno = Self::create_compute_pipeline(&driver.device, 
            &inferno_s, 
            &color_map_pl);

        let jet = Self::create_compute_pipeline(&driver.device, 
            &jet_s, 
            &color_map_pl);

        let render = Self::create_render_pipeline(&driver, 
            &color_bgl, 
            &dimension_vertex_bgl);

        let vertex_buffer = Self::create_vertex_buffer(driver, x, y);

        let draw_s = driver.device.create_shader_module(ShaderModuleDescriptor{ 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("rewritten_shaders/update_barrier/barrier_draw.wgsl")))
        });

        let barrier_update_bgl = Self::create_barrier_update_bgl(driver, x, y);

        let barrier_update_pl = Self::create_barrier_update_pl(driver, &barrier_update_bgl, &barrier_bgl);

        let barrier_draw = Self::create_compute_pipeline(&driver.device, &draw_s, &barrier_update_pl);

        let draw_points = driver.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&vec![0 as u32;2 * x as usize * y as usize]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let draw_num = driver.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::bytes_of(&0),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let draw_bg = driver.device.create_bind_group(&BindGroupDescriptor{
            label: None,
            layout: &barrier_update_bgl,
            entries: &[BindGroupEntry{
                binding: 0,
                resource: draw_num.as_entire_binding(),
            },
            BindGroupEntry{
                binding: 1,
                resource: draw_points.as_entire_binding(),
            }]
        });

        LBM { 
            collide_bg, 
            output_bg, 
            density_bg, 
            dimension_bg, 
            dimension_bg_vertex: vertex_dimension_bg, 
            barrier_bg, 
            ne_sw_bgs, 
            nw_se_bgs, 
            n_s_bgs, 
            e_w_bgs, 
            barrier_buffer, 
            omega_buffer, 
            e_w_stream, 
            n_s_stream, 
            ne_sw_stream, 
            nw_se_stream, 
            curl, 
            ux, 
            uy,
            speed, 
            color_map: ColorMap::Jet, 
            render,
            cardinal_pre_collision,
            corner_pre_collision,
            corner_collide: corner_collision,
            cardinal_collide: cardinal_collision,
            compute_step: 0,
            frame_number: 0,
            work_group_size: Self::calculate_work_group_size(x, y),
            size_bg,
            color_bg,
            vertex_buffer,
            summary_stat: SummaryStat::Curl,
            barrier_draw,
            draw_bg,
            draw_num,
            draw_points,
            viridis,
            jet,
            inferno,
            rho,
            data_buffers,
            x,
            y,
        }
    }

    fn calculate_summary(&mut self, encoder: &mut CommandEncoder){
        match self.summary_stat {
            SummaryStat::Curl => self.curl(encoder),
            SummaryStat::Rho => self.rho(encoder),
            SummaryStat::Ux => self.ux( encoder),
            SummaryStat::Uy => self.uy(encoder),
            SummaryStat::Speed => self.speed(encoder)
        }
    }

    pub fn set_summary(&mut self, stat: SummaryStat){
        self.summary_stat = stat
    }

    pub fn iterate(&mut self, driver: &Driver, compute_steps: usize){
        for _ in 0..compute_steps{
            self.compute_step(driver);
        }
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.calculate_summary(&mut encoder);
        self.color_map(&mut encoder);
        driver.queue.submit(Some(encoder.finish()));
        self.render(driver);
    }

    pub fn reset_to_equilibrium(&mut self, driver : &Driver){
        let equilibrium_state = Self::set_equil(0.1, 0.0, 1.0, self.x, self.y);
        for i in 0..9{
            driver.queue.write_buffer(&self.data_buffers[0][i], 0, bytemuck::cast_slice(&equilibrium_state[i]));
            driver.queue.write_buffer(&self.data_buffers[1][i], 0, bytemuck::cast_slice(&equilibrium_state[i]));
        }
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.compute_step = 0;
        self.frame_number = 0;
        self.pre_collide_corner(&mut encoder);
        self.pre_collide_cardinal(&mut encoder);
        driver.queue.submit(Some(encoder.finish()));
    }

    pub fn custom_speed(&mut self, driver : &Driver, ux: f32){
        let equilibrium_state = Self::set_equil(ux, 0.0, 1.0, self.x, self.y);
        for i in 0..9{
            driver.queue.write_buffer(&self.data_buffers[0][i], 0, bytemuck::cast_slice(&equilibrium_state[i]));
            driver.queue.write_buffer(&self.data_buffers[1][i], 0, bytemuck::cast_slice(&equilibrium_state[i]));
        }
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.compute_step = 0;
        self.frame_number = 0;
        self.pre_collide_corner(&mut encoder);
        self.pre_collide_cardinal(&mut encoder);
        driver.queue.submit(Some(encoder.finish()));
    }

    pub fn rerender(&mut self, driver: &Driver){
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.calculate_summary(&mut encoder);
        self.color_map(&mut encoder);
        driver.queue.submit(Some(encoder.finish()));
        self.render(driver);
    }

    fn compute_step(&mut self, driver: &Driver){
        self.collide(driver);
        self.stream(driver);
        self.compute_step += 1;
    }

    pub fn collide(&mut self, driver: &Driver){
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.pre_collide_corner(&mut encoder);
        self.pre_collide_cardinal(&mut encoder);
        self.collide_corner(&mut encoder);
        self.collide_cardinal(&mut encoder);
        driver.queue.submit(Some(encoder.finish()));
    }

    pub fn stream(&mut self, driver: &Driver){
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.stream_e_w(&mut encoder);
        self.stream_n_s(&mut encoder);
        self.stream_nw_se(&mut encoder);
        self.stream_ne_sw(&mut encoder);
        driver.queue.submit(Some(encoder.finish()));
    }

    pub fn render(&mut self, driver: &Driver) {
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let frame = driver.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.color_bg, &[]);
            rpass.set_bind_group(1, &self.dimension_bg_vertex, &[]);
            rpass.draw(0..6, 0..self.x*self.y);
        }
        driver.queue.submit(Some(encoder.finish()));
        frame.present();
        self.frame_number += 1;
    }

    pub fn get_frame_num(&self) -> usize{
        self.frame_number
    }
    
    pub fn get_compute_num(&self) -> usize{
        self.compute_step
    }

    fn pre_collide_corner(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Precollision-corner") });
        cpass.set_pipeline(&self.corner_pre_collision);
        cpass.set_bind_group(0, &self.ne_sw_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(1, &self.nw_se_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(2, &self.density_bg, &[]);
        cpass.set_bind_group(3, &self.size_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn pre_collide_cardinal(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Precollision-cardinal") });
        cpass.set_pipeline(&self.cardinal_pre_collision);
        cpass.set_bind_group(0, &self.n_s_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(1, &self.e_w_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(2, &self.density_bg, &[]);
        cpass.set_bind_group(3, &self.size_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn collide_corner(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Collision-corner") });
        cpass.set_pipeline(&self.corner_collide);
        cpass.set_bind_group(0, &self.ne_sw_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(1, &self.nw_se_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(2, &self.density_bg, &[]);
        cpass.set_bind_group(3, &self.collide_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn collide_cardinal(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Collision-cardinal") });
        cpass.set_pipeline(&self.cardinal_collide);
        cpass.set_bind_group(0, &self.n_s_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(1, &self.e_w_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(2, &self.density_bg, &[]);
        cpass.set_bind_group(3, &self.collide_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn stream_nw_se(&mut self,  encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Stream_nw_se") });
        cpass.set_pipeline(&self.nw_se_stream);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.nw_se_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(2, &self.nw_se_bgs[(self.compute_step + 1) % 2], &[]);
        cpass.set_bind_group(3, &self.barrier_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn stream_ne_sw(&mut self,  encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Stream_ne_sw") });
        cpass.set_pipeline(&self.ne_sw_stream);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.ne_sw_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(2, &self.ne_sw_bgs[(self.compute_step + 1) % 2], &[]);
        cpass.set_bind_group(3, &self.barrier_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn stream_n_s(&mut self,  encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Stream_n_s") });
        cpass.set_pipeline(&self.n_s_stream);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.n_s_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(2, &self.n_s_bgs[(self.compute_step + 1) % 2], &[]);
        cpass.set_bind_group(3, &self.barrier_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn stream_e_w(&mut self,  encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Stream_e_w") });
        cpass.set_pipeline(&self.e_w_stream);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.e_w_bgs[self.compute_step % 2], &[]);
        cpass.set_bind_group(2, &self.e_w_bgs[(self.compute_step + 1) % 2], &[]);
        cpass.set_bind_group(3, &self.barrier_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn curl(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.curl);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.density_bg, &[]);
        cpass.set_bind_group(2, &self.output_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn ux(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.ux);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.density_bg, &[]);
        cpass.set_bind_group(2, &self.output_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn uy(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.uy);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.density_bg, &[]);
        cpass.set_bind_group(2, &self.output_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn rho(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.rho);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.density_bg, &[]);
        cpass.set_bind_group(2, &self.output_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn speed(&mut self, encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.speed);
        cpass.set_bind_group(0, &self.dimension_bg, &[]);
        cpass.set_bind_group(1, &self.density_bg, &[]);
        cpass.set_bind_group(2, &self.output_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    pub fn color_map(&mut self,  encoder: &mut CommandEncoder){
        match self.color_map {
            ColorMap::Inferno => self.inferno_map(encoder),
            ColorMap::Viridis => self.viridis_map(encoder),
            ColorMap::Jet => self.jet_map(encoder),
        }
    }

    fn viridis_map(&mut self,  encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.viridis);
        cpass.set_bind_group(0, &self.color_bg, &[]);
        cpass.set_bind_group(1, &self.output_bg, &[]);
        cpass.set_bind_group(2, &self.barrier_bg, &[]);
        cpass.set_bind_group(3, &self.size_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn jet_map(&mut self,  encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.jet);
        cpass.set_bind_group(0, &self.color_bg, &[]);
        cpass.set_bind_group(1, &self.output_bg, &[]);
        cpass.set_bind_group(2, &self.barrier_bg, &[]);
        cpass.set_bind_group(3, &self.size_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    fn inferno_map(&mut self,  encoder: &mut CommandEncoder){
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.inferno);
        cpass.set_bind_group(0, &self.color_bg, &[]);
        cpass.set_bind_group(1, &self.output_bg, &[]);
        cpass.set_bind_group(2, &self.barrier_bg, &[]);
        cpass.set_bind_group(3, &self.size_bg, &[]);
        cpass.dispatch_workgroups(self.work_group_size as u32, 1, 1);
    }

    pub fn draw_shape(&mut self, driver : &Driver, shape: &dyn Shape){
        self.draw_barrier_updates(driver, get_points_vector(shape, self.x as usize));
    }

    fn draw_barrier_updates(&mut self,  driver : &Driver, points : Vec<u32>){
        driver.queue.write_buffer(&self.draw_points, 0, bytemuck::cast_slice(&points));
        driver.queue.write_buffer(&self.draw_num, 0, bytemuck::bytes_of(&(points.len() as u32 - 1)));
        driver.queue.submit(None);

        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.barrier_draw);
        cpass.set_bind_group(0, &self.draw_bg, &[]);
        cpass.set_bind_group(1, &self.barrier_bg, &[]);
        let work_groups = (points.len()/2) as u32;
        cpass.dispatch_workgroups(work_groups, 1, 1);
        }
        driver.queue.submit(Some(encoder.finish()));
    }

    pub fn update_omega_buffer(&mut self,  driver : &Driver, omega: f32){
        driver.queue.write_buffer(&self.omega_buffer, 0, bytemuck::bytes_of(&omega));
    }

    pub fn reset_barrier(&mut self, driver : &Driver){
        let barrier_reset = Self::init_barrier(self.x, self.y);
        driver.queue.write_buffer(&self.barrier_buffer, 0, bytemuck::cast_slice(&barrier_reset));
    }

    pub fn curl_barrier(&mut self, driver : &Driver){
        let line = Line::new(((4 * self.x/10) as isize, (self.y/4) as isize), ((4 * self.x/10) as isize, (self.y/2) as isize), self.x as isize, self.y as isize);
        self.draw_shape(driver, &line.unwrap());
    }

    pub fn chaos_barrier(&mut self, driver : &Driver){
        
        let l1 = Line::new(((self.x/2) as isize, (9 * self.y/20) as isize), ((self.x/2) as isize, 0), self.x as isize, self.y as isize).unwrap();
        self.draw_shape(driver, &l1);

        let l2 = Line::new(((self.x/2) as isize, (11 * self.y/20) as isize), ((self.x/2) as isize, self.y as isize - 1), self.x as isize, self.y as isize).unwrap();
        self.draw_shape(driver, &l2);

        let c1 = Line::new(((3 * self.x/5) as isize, (self.y/2) as isize), ((3 * self.x/4) as isize, 3 * self.y as isize/4 ), self.x as isize, self.y as isize).unwrap();
        self.draw_shape(driver, &c1);

        let c2 = Line::new(((3 * self.x/5) as isize, (self.y/2) as isize), ((3 * self.x/4) as isize, self.y as isize/4), self.x as isize, self.y as isize).unwrap();
        self.draw_shape(driver, &c2);

    }

    pub fn welcome_barrier(&mut self, driver : &Driver){
        let curried_draw = |p1: (isize, isize), p2: (isize, isize)| {
            Line::new(p1, p2, self.x as isize, self.y as isize).unwrap()
        };

        let mut blob = Blob::new_empty();
        let height = -1 * (self.y/4) as isize;
        let bottom = (self.y/2) as isize;
        let space = (self.x/50) as isize;
        let mut current_x = (self.x/5) as isize;
        let letter_width = (self.x/13) as isize;
        // Draw W
        let v1 = curried_draw((current_x, bottom + height), (current_x, bottom));
        let d1 = curried_draw((current_x, bottom), (current_x + letter_width/2, bottom + height/2));
        current_x += letter_width/2;
        let d2 = curried_draw((current_x, bottom + height/2), (current_x + letter_width/2, bottom));
        current_x += letter_width/2 ;
        let v2 = curried_draw((current_x, bottom + height), (current_x, bottom));
        current_x += space;
        blob.join(&v1);
        blob.join(&d1);
        blob.join(&d2);
        blob.join(&v2);

        //Draw e
        let v1 = curried_draw((current_x, bottom + height/2), (current_x, bottom));
        let h1 = curried_draw((current_x, bottom), (current_x + letter_width, bottom));
        let h2 = curried_draw((current_x, bottom + height/4), (current_x + letter_width, bottom + height/4));
        let h3 = curried_draw((current_x, bottom + height/2), (current_x + letter_width, bottom + height/2));
        let v2 = curried_draw((current_x + letter_width, bottom + height/2), (current_x + letter_width, bottom + height/4));
        current_x += letter_width + space;
        blob.join(&v1);
        blob.join(&h1);
        blob.join(&h2);
        blob.join(&v2);
        blob.join(&h3);

        //Draw l
        blob.join(&curried_draw((current_x, bottom), (current_x, bottom + height)));
        current_x += space;

        //Draw c
        let v1 = curried_draw((current_x, bottom + height/2), (current_x, bottom));
        let h1 = curried_draw((current_x, bottom), (current_x + letter_width, bottom));
        let h2 = curried_draw((current_x, bottom + height/2), (current_x + letter_width, bottom + height/2));
        blob.join(&v1);
        blob.join(&h1);
        blob.join(&h2);
        current_x += letter_width + space;

        //Draw o
        let v1 = curried_draw((current_x, bottom + height/2), (current_x, bottom));
        let h1 = curried_draw((current_x, bottom), (current_x + letter_width, bottom));
        let h2 = curried_draw((current_x, bottom + height/2), (current_x + letter_width, bottom + height/2));
        let v2 = curried_draw((current_x + letter_width, bottom + height/2), (current_x + letter_width, bottom));
        blob.join(&v1);
        blob.join(&h1);
        blob.join(&h2);
        blob.join(&v2);
        current_x += letter_width + space;

        //Draw m
        let v1 = curried_draw((current_x, bottom + height/2), (current_x, bottom));
        let v3 = curried_draw((current_x + letter_width/2, bottom), (current_x + letter_width/2, bottom + height/2));
        let h1 = curried_draw((current_x, bottom + height/2), (current_x + letter_width, bottom + height/2));
        let v2 = curried_draw((current_x + letter_width, bottom + height/2), (current_x + letter_width, bottom));
        blob.join(&v1);
        blob.join(&h1);
        blob.join(&v2);
        blob.join(&v3);
        current_x += letter_width + space;

        //Draw e
        let v1 = curried_draw((current_x, bottom + height/2), (current_x, bottom));
        let h1 = curried_draw((current_x, bottom), (current_x + letter_width, bottom));
        let h2 = curried_draw((current_x, bottom + height/4), (current_x + letter_width, bottom + height/4));
        let h3 = curried_draw((current_x, bottom + height/2), (current_x + letter_width, bottom + height/2));
        let v2 = curried_draw((current_x + letter_width, bottom + height/2), (current_x + letter_width, bottom + height/4));
        current_x += letter_width + space;
        blob.join(&v1);
        blob.join(&h1);
        blob.join(&h2);
        blob.join(&v2);
        blob.join(&h3);

        //Draw exclamation
        let v1 = curried_draw((current_x, height/10 + bottom), (current_x, bottom));
        let v2 = curried_draw((current_x, height/5 + bottom), (current_x, bottom + height));
        blob.join(&v1);
        blob.join(&v2);
        self.draw_shape(driver, &blob);

    }

    fn set_single_cell(&self, index: usize)-> Vec<Vec<f32>>{

        let mut vec: Vec<Vec<f32>> = Self::set_equil(0.0, 0.0, 1.0, self.x, self.y);

        match index {
            0 => vec[0][self.index((self.x - 2) as usize, (self.y - 2) as usize)] = 4.0,
            1 => vec[1][self.index((3 * self.x/4) as usize, (self.y - 2) as usize)] = 4.0,
            2 => vec[2][self.index((self.x/3) as usize, (self.y - 2) as usize)] = 4.0,
            3 => vec[3][self.index((self.x - 2) as usize, (self.y/2) as usize)] = 4.0,
            4 => vec[4][self.index((3 * self.x/4) as usize, (self.y/2) as usize)] = 4.0,
            5 => vec[5][self.index((self.x/2) as usize, (self.y/2) as usize)] = 4.0,
            6 => vec[6][self.index((self.x - 2) as usize, 1 as usize)] = 4.0,
            7 => vec[7][self.index((3 * self.x/4) as usize, 1)] = 4.0,
            8 => vec[8][self.index((self.x/2) as usize, 1)] = 4.0,
            _ =>  ()  
        };

        vec
    }

    pub fn single_cell(&mut self, driver : &Driver, index: usize){

        let data = self.set_single_cell(index);

        for i in 0..9{
            driver.queue.write_buffer(&self.data_buffers[0][i], 0, bytemuck::cast_slice(&data[i]));
            driver.queue.write_buffer(&self.data_buffers[1][i], 0, bytemuck::cast_slice(&data[i]));
        }

        let encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.compute_step = 0;
        self.frame_number = 0;
        driver.queue.submit(Some(encoder.finish()));
    }

    fn index(&self, x: usize, y:usize) -> usize{
        x + y * self.x as usize
    }

}