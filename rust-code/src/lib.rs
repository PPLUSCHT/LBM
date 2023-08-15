use barrier_shapes::{Shape, blob::Blob, line, curve::Curve, curve_collection::CurveCollection};
use driver::Driver;
use lbm::ColorMap;
use web_sys::console;
use winit::{event_loop::{EventLoop, ControlFlow}, dpi::LogicalSize, event::{Event, WindowEvent, ElementState}, window::Window};
use wasm_bindgen::prelude::*;

use lazy_static::lazy_static; // 1.4.0
use std::{sync::Mutex, collections::{HashSet, HashMap}, mem};
use crate::lbm::SummaryStat;

lazy_static! {
    static ref CURRENT_OUTPUT: Mutex<SummaryStat> = Mutex::new(SummaryStat::Curl);
    static ref BARRIER_CHANGE: Mutex<bool> = Mutex::new(false);
    static ref PAUSE: Mutex<bool> = Mutex::new(false);
    static ref OUTPUT_CHANGED: Mutex<bool> = Mutex::new(true);
    static ref COMPUTE_PER_RENDER: Mutex<u32> = Mutex::new(15);
    static ref VISCOSITY: Mutex<f32> = Mutex::new(0.1);
    static ref VISCOSITY_CHANGED: Mutex<bool> = Mutex::new(false);
    static ref CURRENT_COLOR_MAP: Mutex<ColorMap> = Mutex::new(ColorMap::Jet);
    static ref COLOR_CHANGED: Mutex<bool> = Mutex::new(false);
    static ref CLICK_TYPE: Mutex<ClickType> = Mutex::new(ClickType::Inactive);
    static ref CLICK_TYPE_CHANGED: Mutex<bool> = Mutex::new(true);
    static ref UNDO_CHANGED: Mutex<bool> = Mutex::new(false);
    static ref UNDO_COUNT: Mutex<usize> = Mutex::new(0);
    static ref STEP_MODE: Mutex<bool> = Mutex::new(false);
    static ref STEP_TAKEN: Mutex<bool> = Mutex::new(false);
    static ref FLUID_PRESET_CHANGE: Mutex<bool> = Mutex::new(false);
    static ref FLUID_PRESET: Mutex<FluidPreset> = Mutex::new(FluidPreset::Equilibrium);
    static ref FLUID_SPEED: Mutex<f32> = Mutex::new(0.1);
    static ref BARRIER_PRESET_CHANGE: Mutex<bool> = Mutex::new(false);
    static ref BARRIER_PRESET: Mutex<BarrierPreset> = Mutex::new(BarrierPreset::Tunnel);
}

pub mod driver;
pub mod barrier_shapes;
pub mod lbm;

const OMEGA:f32 = 1.0/(0.5 + 0.3);


pub async fn run_wasm(event_loop: EventLoop<()>, window:Window, x:u32, y:u32, pixel_ratio: f32) {

    let driver = Driver::new(&window).await;

    let mut lbm = lbm::LBM::new(&driver, OMEGA, x, y);
    let mut pressed = false; 
    let mut click_handler = ClickHandler::new(x, y);
    let mut current_position: (isize, isize) = (0,0);
 
    let swapchain_capabilities = driver.surface.get_capabilities(&driver.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];
 
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: driver.size.width,
        height: driver.size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };
 
    driver.surface.configure(&driver.device, &config);
    lbm.render(&driver);

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                config.width = size.width;
                config.height = size.height;
                driver.surface.configure(&driver.device, &config);
                window.request_redraw();
            }

            Event::WindowEvent { 
                event: WindowEvent::CursorMoved{position, ..}, ..  
            } => {
                let temp:(i32, i32) = position.to_logical::<i32>(pixel_ratio.into()).into();
                current_position = click_handler.validate_click((temp.0 as isize, temp.1 as isize));
                if pressed{
                    click_handler.handle_movement(current_position);
                }
            }

            Event::WindowEvent{
                event: WindowEvent::MouseInput {state, ..}, ..
            } => {
                let mut click_type_changed = CLICK_TYPE_CHANGED.lock().unwrap();
                if *click_type_changed{
                    let t = CLICK_TYPE.lock().unwrap();
                    click_handler.switch_click_type(*t);
                    *click_type_changed = false;
                }
                if state == ElementState::Released{
                    pressed = false;
                    click_handler.handle_release();
                } else{
                    pressed = true;
                    click_handler.handle_click(current_position);
                }
            }

            Event::RedrawRequested(_) => {

                let paused = *PAUSE.lock().unwrap();
                let mut barrier_redraw = !click_handler.current_blob.is_empty() || !click_handler.current_curve.is_empty();
                let mut output_changed = OUTPUT_CHANGED.lock().unwrap();
                let mut color_changed = COLOR_CHANGED.lock().unwrap();
                let mut fluid_preset_changed = FLUID_PRESET_CHANGE.lock().unwrap();
                let mut undo_changed = UNDO_CHANGED.lock().unwrap();
                let mut barrier_preset_changed = BARRIER_PRESET_CHANGE.lock().unwrap();
                let step_mode = *STEP_MODE.lock().unwrap();
                let mut step = STEP_TAKEN.lock().unwrap();

                if *barrier_preset_changed{

                    lbm.reset_barrier(&driver);
                    match *BARRIER_PRESET.lock().unwrap(){
                        BarrierPreset::Welcome => lbm.welcome_barrier(&driver),
                        BarrierPreset::Tunnel => (),
                        BarrierPreset::Curl => lbm.curl_barrier(&driver),
                        BarrierPreset::Chaos => lbm.chaos_barrier(&driver),
                    }

                    click_handler.clear_barrier();
                    barrier_redraw = false;
                    let mut undo_count = UNDO_COUNT.lock().unwrap();
                    *undo_count = 0;
                    *undo_changed = false;
                }

                if *undo_changed{
                    let mut undo_count = UNDO_COUNT.lock().unwrap();
                    let mut undo_blob = Blob::new_empty();
                    for _ in 0..*undo_count{
                        // click_handler.test_undo();
                        match click_handler.undo() {
                            Some(u) => undo_blob.join(&*u),
                            None => {},
                        }
                    }
                    if !undo_blob.is_empty(){
                        lbm.draw_shape(&driver, &undo_blob);
                    }
                    click_handler.empty_all();
                    barrier_redraw = false;
                    *undo_count = 0;
                }

                if *fluid_preset_changed{
                    match *FLUID_PRESET.lock().unwrap() {
                        FluidPreset::Equilibrium => lbm.reset_to_equilibrium(&driver),
                        FluidPreset::SingleNorth => lbm.single_cell(&driver, 1),
                        FluidPreset::SingleOrigin => lbm.single_cell(&driver, 4),
                        FluidPreset::SingleEast => lbm.single_cell(&driver, 5),
                        FluidPreset::SingleNorthEast => lbm.single_cell(&driver, 2),
                        FluidPreset::CustomSpeed => lbm.custom_speed(&driver, *FLUID_SPEED.lock().unwrap()),
                    }
                }

                if *output_changed{
                    let current:SummaryStat =  *CURRENT_OUTPUT.lock().unwrap();
                    lbm.set_summary(current);
                }

                if *color_changed{
                    lbm.color_map = *CURRENT_COLOR_MAP.lock().unwrap();
                }

                if barrier_redraw{
                    click_handler.current_blob.join(&click_handler.current_curve);
                    lbm.draw_shape(&driver, &click_handler.current_blob);
                    click_handler.update(pressed, current_position);
                }

                let mut viscosity_changed = VISCOSITY_CHANGED.lock().unwrap();
                if *viscosity_changed{
                    let omega = 1.0/(3.0 * *VISCOSITY.lock().unwrap() + 0.5);
                    lbm.update_omega_buffer(&driver, omega);
                    *viscosity_changed = false;
                }

                if !paused && !step_mode{
                    let current:u32 =  *COMPUTE_PER_RENDER.lock().unwrap();
                    lbm.iterate(&driver, current as usize);
                }else if step_mode && *step{
                    if *step{
                        lbm.iterate(&driver, 1);
                    }
                    *step = false;
                }else if *output_changed || barrier_redraw || *color_changed || *fluid_preset_changed || *undo_changed || *barrier_preset_changed{
                    lbm.rerender(&driver);
                }
                *undo_changed = false;
                *output_changed = false;
                *color_changed = false;
                *fluid_preset_changed = false;
                *barrier_preset_changed = false;
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ClickType{
    Line,
    Erase, 
    Draw, 
    Inactive
}

struct ClickHandler{
    current_type: ClickType,
    line_points: Vec<(isize, isize)>,
    current_blob: Blob,
    current_curve: Curve,
    contiguous_curve: CurveCollection,
    undo_stack: Vec<Box<dyn Shape>>,
    history: HashMap<(isize, isize), Vec<bool>>,
    x: u32, 
    y: u32,
}

impl ClickHandler{
    
    pub fn new(x: u32, y:u32) -> ClickHandler{
        ClickHandler{
            current_type: ClickType::Draw,
            current_blob: Blob { points: HashSet::<(isize, isize, bool)>::new() },
            line_points: Vec::<(isize, isize)>::new(),
            undo_stack: Vec::<Box<dyn Shape>>::new(),
            current_curve: Curve::new(),
            contiguous_curve: CurveCollection::new(),
            history: HashMap::<(isize, isize), Vec<bool>>::new(),
            x,
            y,
        }
    }

    pub fn clear_barrier(&mut self){
        self.current_curve = Curve::new();
        self.contiguous_curve = CurveCollection::new();
        self.undo_stack.clear();
        self.history.clear();
        self.current_blob.empty();
        self.line_points.clear();
    }

    pub fn handle_movement(&mut self, location: (isize, isize)){
        match self.current_type {
            ClickType::Erase => self.current_curve.erase_segment(location, self.x as isize, self.y as isize),
            ClickType::Draw => self.current_curve.add_segment(location, self.x as isize, self.y as isize),
            _ => (),
        }  
    }

    pub fn handle_release(&mut self){
        match self.current_type {
            ClickType::Erase => self.release(),
            ClickType::Draw => self.release(),
            _ => (),
        }
    }

    fn release(&mut self){
        //Join current curve to blob so it will be rendered
        self.current_blob.join(&self.current_curve);

        //Add current to contiguous curve
        let mut temp = Curve::new();
        mem::swap(&mut self.current_curve, &mut temp);
        self.contiguous_curve.add_curve(temp);

        //Add contiguous curve to history
        let mut temp = CurveCollection::new();
        mem::swap(&mut self.contiguous_curve, &mut temp);
        self.add_to_history(Box::new(temp));
    }

    pub fn empty_all(&mut self){
        self.current_blob.empty();
        self.current_curve.empty();
    }

    pub fn handle_click(&mut self, click_location: (isize, isize)){
        match self.current_type {
            ClickType::Line => self.line_click(click_location),
            ClickType::Erase => self.current_curve.erase_segment(click_location, self.x as isize, self.y as isize),
            ClickType::Draw => self.current_curve.add_segment(click_location, self.x as isize, self.y as isize),
            _ => (),
        }
    }

    pub fn switch_click_type(&mut self, click_type: ClickType){
        self.current_type = click_type;
        self.line_points.clear();
    }

    pub fn update(&mut self, pressed: bool, location: (isize, isize)){
        self.current_blob.empty();
        if pressed && *CLICK_TYPE.lock().unwrap() == ClickType::Draw{
            self.draw_update(location);
        }
        if pressed && *CLICK_TYPE.lock().unwrap() == ClickType::Erase{
            self.erase_update(location);
        }
    }

    pub fn undo(&mut self) -> Option<Box<dyn Shape>>{
        if self.current_blob.is_empty() && self.current_curve.is_empty(){
            match self.undo_stack.pop() {
                Some(s) => 
                return Some(self.remove_shape(&*s)),
                None => return None,
            };
        } else {
            self.current_blob.empty();
            self.current_curve.empty();
        }
        None
    }

    fn line_click(&mut self, click_location: (isize, isize)){
        self.line_points.push(click_location);
        if self.line_points.len() >= 2{
            let shape = line::Line::new(self.line_points[0], 
                                              self.line_points[1], 
                                                            self.x as isize, 
                                                            self.y as isize)
                                                            .unwrap();
            self.current_blob.join(&shape);
            self.add_to_history(Box::new(shape));
            self.line_points.clear();
        }
    }

    fn draw_update(&mut self, click_location: (isize, isize)){
        let mut temp = Curve::new();
        mem::swap(&mut self.current_curve, &mut temp);
        self.contiguous_curve.add_curve(temp);
        self.current_curve.add_segment(click_location, self.x as isize, self.y as isize);
    }

    fn erase_update(&mut self, click_location: (isize, isize)){
        let mut temp = Curve::new();
        mem::swap(&mut self.current_curve, &mut temp);
        self.contiguous_curve.add_curve(temp);
        self.current_curve.erase_segment(click_location, self.x as isize, self.x as isize);
    }

    pub fn validate_click(&self, click_location: (isize, isize)) -> (isize, isize){
        let mut new_click = (0,0);
        if click_location.0 < 0{
            new_click.0 = self.x as isize + click_location.0;
        } else if click_location.0 >= self.x as isize {
            new_click.0 = self.x as isize - 1;
        } else {
            new_click.0 = click_location.0;
        }
        if click_location.1 < 0{
            new_click.1 = self.y as isize + click_location.1;
        } else if click_location.1 >= self.y as isize {
            new_click.1 = self.y as isize - 1;
        } else{
            new_click.1 = click_location.1;
        }
        new_click
    }

    fn add_to_history(&mut self, shape: Box<dyn Shape>){
        for i in shape.get_points(){
            match self.history.get_mut(&(i.0, i.1)){
                Some(vec) => {vec.push(i.2)},
                None => {
                    self.history.insert((i.0, i.1), vec![i.2]);
                },
            };
        }
        self.undo_stack.push(shape);
    }

    fn remove_shape(&mut self, shape: &dyn Shape) -> Box<dyn Shape>{
        let mut points = Vec::<(isize, isize, bool)>::new();
        for i in shape.get_points(){
            match self.history.get_mut(&(i.0, i.1)){
                Some(vec) => {
                    vec.pop(); 
                    points.push((i.0, i.1, Self::add_point(vec)));
                    if vec.is_empty(){
                        self.history.remove(&(i.0, i.1));
                    }
                },
                None => {
                    points.push((i.0, i.1, false));
                },
            };
        }
        points.sort();
        let mut blob = Blob::new(HashSet::new());
        blob.add(&points, self.x, self.y);
        Box::new(blob)
    }

    fn add_point(vec: &Vec<bool>) -> bool{
       if vec.is_empty(){
        return false;
       }
       vec.last().unwrap().to_owned()
    }
}

#[wasm_bindgen]
pub enum FluidPreset{
    Equilibrium,
    CustomSpeed,
    SingleNorth,
    SingleOrigin,
    SingleEast,
    SingleNorthEast
}

#[wasm_bindgen]
pub enum BarrierPreset{
    Welcome, 
    Tunnel,
    Curl, 
    Chaos
}


#[wasm_bindgen]
struct WASMInteraction{
}

#[wasm_bindgen]
impl WASMInteraction {

    pub fn set_output(summary_stat:SummaryStat){
        let mut mutex_changer = CURRENT_OUTPUT.lock().unwrap();
        *mutex_changer = summary_stat;
        let mut mutex_changer = OUTPUT_CHANGED.lock().unwrap();
        *mutex_changer = true;
    }

    pub fn set_draw_type(draw_type: ClickType){
        let mut mutex_changer = CLICK_TYPE.lock().unwrap();
        *mutex_changer = draw_type;
        let mut mutex_changer = CLICK_TYPE_CHANGED.lock().unwrap();
        *mutex_changer = true;
    }

    pub fn set_color_map(color_map:ColorMap){
        let mut mutex_changer = CURRENT_COLOR_MAP.lock().unwrap();
        *mutex_changer = color_map;
        let mut mutex_changer = COLOR_CHANGED.lock().unwrap();
        *mutex_changer = true;
    }

    pub fn toggle_pause(){
        let mut mutex_changer = PAUSE.lock().unwrap();
        *mutex_changer = !*mutex_changer;
    }

    pub fn update_compute_rate(rate: u32){
        let mut mutex_changer = COMPUTE_PER_RENDER.lock().unwrap();
        *mutex_changer = rate;
    }

    pub fn update_viscosity(viscosity: f32){
        let mut mutex_changer = VISCOSITY.lock().unwrap();
        *mutex_changer = viscosity;
        let mut mutex_changer = VISCOSITY_CHANGED.lock().unwrap();
        *mutex_changer = true;
    }

    pub fn update_flow_speed(speed: f32){
        let mut fluid_changer = FLUID_PRESET_CHANGE.lock().unwrap();
        *fluid_changer = true;
        let mut fluid_preset = FLUID_PRESET.lock().unwrap();
        *fluid_preset = FluidPreset::CustomSpeed;
        let mut fluid_speed = FLUID_SPEED.lock().unwrap();
        *fluid_speed = speed;
    }

    pub fn undo(){
        let mut mutex_changer = UNDO_COUNT.lock().unwrap();
        *mutex_changer += 1;
        let mut mutex_changer = UNDO_CHANGED.lock().unwrap();
        *mutex_changer = true;
    }

    pub fn set_step_mode(){
        let mut step_mode = STEP_MODE.lock().unwrap();
        *step_mode = true;
    }

    pub fn release_step_mode(){
        let mut step_mode = STEP_MODE.lock().unwrap();
        *step_mode = false;
    }

    pub fn take_step(){
        let mut take_step = STEP_TAKEN.lock().unwrap();
        *take_step = true;
    }

    pub fn change_fluid_preset(f: FluidPreset){
        let mut fluid_changer = FLUID_PRESET_CHANGE.lock().unwrap();
        *fluid_changer = true;
        let mut fluid_preset = FLUID_PRESET.lock().unwrap();
        *fluid_preset = f;
    }

    pub fn change_barrier_preset(b: BarrierPreset){
        let mut barrier_changer = BARRIER_PRESET_CHANGE.lock().unwrap();
        *barrier_changer = true;
        let mut barrier_preset = BARRIER_PRESET.lock().unwrap();
        *barrier_preset = b;
    }
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Resolution{
    TEST = 1000,
    NHD = 230400, 
    HD =  921600,
    FHD = 2073600,
    UHD = 3686400,
}

fn calculate_dimensions(res: Resolution, width: u32, height: u32) -> (u32, u32, f32){
    let aspect_ratio = height as f64/ width as f64;
    let x_pixels = (res as isize as f64/aspect_ratio).sqrt().floor() as u32;
    let pixel_size = width as f64/ x_pixels as f64;
    let y_pixels = (height as f64/pixel_size).floor() as u32;
    (x_pixels, y_pixels, pixel_size as f32)
}


#[wasm_bindgen]
pub fn run(pixel_ratio: f32, res: Resolution, width: u32, height: u32) {
    let event_loop = EventLoop::new();
    let dimensions = calculate_dimensions(res, width, height);
    let window = winit::window::WindowBuilder::new()
                        .with_inner_size(LogicalSize{width: dimensions.0 as f32 * dimensions.2, height: dimensions.1 as f32 * dimensions.2})
                        .build(&event_loop).unwrap();
    use winit::platform::web::WindowExtWebSys;
    use wasm_bindgen_futures;
    use web_sys;
    console_log::init().expect("could not initialize logger");
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");
    console::log_1(&"Hello from before run".into());
    wasm_bindgen_futures::spawn_local(run_wasm(event_loop, window, dimensions.0, dimensions.1, pixel_ratio * dimensions.2));
}

#[wasm_bindgen]
pub async fn test_compatibility() -> bool{
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
                        .with_inner_size(LogicalSize{width: 100.0 as f32, height: 100.0})
                        .build(&event_loop).unwrap();
    Driver::test_compatibility(&window).await
}
