use bevy::color::Color;
use bevy::prelude::*;

use bevy_pixel_buffer::prelude::*;

use bevy_egui::{egui, EguiContexts, EguiPlugin};

use noise::{utils::*, MultiFractal};
use noise::{BasicMulti, Perlin};

const MAP_SIZE: usize = 1000;
const TILE_SIZE: usize = 2;
#[derive(Component)]
struct NoiseMapComponent(NoiseMap);

fn main() {
    let size = PixelBufferSize {
        size: UVec2::splat(MAP_SIZE as u32),
        pixel_size: UVec2::splat(TILE_SIZE as u32),
    };

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Noise Explorer".into(),
                    name: Some("Noise.Explorer".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            EguiPlugin,
            PixelBufferPlugins,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Startup,
            PixelBufferBuilder::new()
                .with_render(RenderConfig::sprite())
                .with_size(size)
                .setup(),
        )
        .add_systems(Update, (update_sprite, update_gui))
        .run();
}

fn get_color(val: f64) -> Color {
    match val.abs() {
        v if v < 0.1 => Color::srgb_u8(0x0u8, 0x0u8, 0xFFu8),
        v if v < 0.2 => Color::srgb_u8(0x0du8, 0xa5u8, 0x0du8),
        v if v < 0.3 => Color::srgb_u8(0x10u8, 0xcbu8, 0x10u8),
        v if v < 0.4 => Color::srgb_u8(0x18u8, 0xedu8, 0x18u8),
        v if v < 0.5 => Color::srgb_u8(0x3fu8, 0xf0u8, 0x3fu8),
        v if v < 0.6 => Color::srgb_u8(0x65u8, 0xf3u8, 0x65u8),
        v if v < 0.7 => Color::srgb_u8(0x8cu8, 0xf6u8, 0x8cu8),
        v if v < 0.8 => Color::srgb_u8(0xb2u8, 0xf9u8, 0xb2u8),
        v if v < 0.9 => Color::srgb_u8(0xd9u8, 0xfcu8, 0xd9u8),
        v if v <= 1.0 => Color::srgb_u8(0xffu8, 0xffu8, 0xffu8),
        _ => panic!("unexpected value"),
    }
}

#[derive(Component)]
struct Parameters {
    seed: u32,
    frequency: f64,
    octaves: usize,
    lacunarity: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            seed: 0,
            frequency: 2.0,
            octaves: 4,
            lacunarity: 2.0,
        }
    }
}

fn generate_noise_map(map_size: usize, params: &Parameters) -> NoiseMap {
    let basicmulti = BasicMulti::<Perlin>::new(params.seed)
        .set_octaves(params.octaves)
        .set_frequency(params.frequency)
        .set_lacunarity(params.lacunarity);

    PlaneMapBuilder::new(&basicmulti)
        .set_size(map_size, map_size)
        .build()
}

fn setup(mut commands: Commands, images: ResMut<Assets<Image>>) {
    let params = Parameters::default();

    commands.spawn(Camera2dBundle::default());

    let map = generate_noise_map(MAP_SIZE, &params);
    let (grid_width, grid_height) = map.size();
    debug!("Map size: {}x{}", grid_width, grid_height);
    let mut noise_map = NoiseMapComponent(map);
    regen_map(&mut noise_map, &params);
    commands.spawn(noise_map);
    commands.spawn(params);
}

fn update_sprite(
    image: Query<&Handle<Image>>,
    map: Query<&NoiseMapComponent>,
    mut images: ResMut<Assets<Image>>,
) {
    let map = &map.single().0;
    if let Ok(image) = image.get_single() {
        Frame::extract(&mut images, image).per_pixel(|coords, _| update_pixel(coords, map));
    }
}

fn regen_map(map_component: &mut NoiseMapComponent, params: &Parameters) {
    let map = generate_noise_map(MAP_SIZE, params);
    let (grid_width, grid_height) = map.size();
    debug!("Map size: {}x{}", grid_width, grid_height);

    map_component.0 = map;
}

fn update_pixel(coords: UVec2, map: &NoiseMap) -> Pixel {
    let val = map.get_value(coords.x as usize, coords.y as usize);
    Pixel::from(get_color(val))
}

fn update_gui(
    mut egui_context: EguiContexts,
    mut map_component: Query<&mut NoiseMapComponent>,
    mut parameters: Query<&mut Parameters>,
) {
    let mut parameters = parameters.single_mut();
    // show ui
    let ctx = egui_context.ctx_mut();
    let mut binding = map_component.single_mut();
    let map_component = binding.as_mut();
    egui::SidePanel::left("left_panel").show(ctx, |ui| {
        ui.heading("Controls");
        if ui.add(egui::Button::new("Regenerate")).clicked() {
            regen_map(map_component, &parameters);
        }
        ui.label("Parameters");
        if ui
            .add(egui::Slider::new(&mut parameters.seed, 0..=65534).text("Seed"))
            .changed()
        {
            regen_map(map_component, &parameters);
        };
        if ui
            .add(egui::Slider::new(&mut parameters.frequency, 0.0..=10.0).text("frequency"))
            .changed()
        {
            regen_map(map_component, &parameters);
        };
        if ui
            .add(egui::Slider::new(&mut parameters.octaves, 0..=10).text("Octaves"))
            .changed()
        {
            regen_map(map_component, &parameters);
        };

        if ui
            .add(egui::Slider::new(&mut parameters.lacunarity, 0.0..=10.0).text("lacunarity"))
            .changed()
        {
            regen_map(map_component, &parameters);
        };
    });
}
