use std::collections::HashMap;
use egui::TextureHandle;

pub fn get_all_angles_for_flight() -> Vec<i32> {
    vec![0, 45, 90, 135, 180, 225, 270, 315]
}

pub fn get_texture_for_flight(angle: f32, all_textures: HashMap<i32, Option<TextureHandle>>) -> Option<TextureHandle> {
    match angle {
        0.0..=22.5 | 337.5..=360.0 => get_some_texture(all_textures.get(&0)),
        22.5..=67.5 => get_some_texture(all_textures.get(&45)),
        67.5..=112.5 => get_some_texture(all_textures.get(&90)),
        112.5..=157.5 => get_some_texture(all_textures.get(&135)),
        157.5..=202.5 => get_some_texture(all_textures.get(&180)),
        202.5..=247.5 => get_some_texture(all_textures.get(&225)),
        247.5..=292.5 => get_some_texture(all_textures.get(&270)),
        292.5..=337.5 => get_some_texture(all_textures.get(&315)),
        _ => None,
    }
}

fn get_some_texture(value: Option<&Option<TextureHandle>>) -> Option<TextureHandle> {
    if let Some(value) = value {
        return value.clone();
    }
    return None;
}
