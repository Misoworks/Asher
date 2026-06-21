use asher_config::{AsherConfig, ConfigPaths};
use image::{DynamicImage, GenericImageView, ImageReader, imageops::FilterType};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const GLASS_BLUR_MAX_EDGE: u32 = 960;
const GLASS_BLUR_CACHE_VERSION: u8 = 5;

#[derive(Debug, Clone, Copy)]
pub struct MaterialColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl MaterialColor {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            a: ((self.a as f32 * alpha).round() as i32).clamp(0, 255) as u8,
            ..self
        }
    }

    pub fn css_rgba(self) -> String {
        format!(
            "rgba({}, {}, {}, {:.3})",
            self.r,
            self.g,
            self.b,
            self.a as f32 / 255.0
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialPalette {
    pub panel: MaterialColor,
    pub panel_control: MaterialColor,
    pub panel_text: MaterialColor,
    pub dock: MaterialColor,
    pub accent: MaterialColor,
    pub text_soft: MaterialColor,
    pub text_muted: MaterialColor,
}

impl Default for MaterialPalette {
    fn default() -> Self {
        Self {
            panel: MaterialColor::rgba(22, 22, 20, 158),
            panel_control: MaterialColor::rgba(255, 255, 255, 20),
            panel_text: MaterialColor::rgba(248, 248, 246, 245),
            dock: MaterialColor::rgba(24, 23, 20, 86),
            accent: MaterialColor::rgba(210, 192, 130, 255),
            text_soft: MaterialColor::rgba(218, 216, 205, 232),
            text_muted: MaterialColor::rgba(164, 162, 154, 222),
        }
    }
}

pub fn shell_material_palette(config: &AsherConfig) -> MaterialPalette {
    config
        .compositor
        .background_image
        .as_deref()
        .and_then(load_wallpaper_average)
        .map(glass_palette_from_wallpaper)
        .unwrap_or_default()
}

pub fn glass_blur_wallpaper_path(config: &AsherConfig) -> Option<PathBuf> {
    let wallpaper = config.compositor.background_image.as_deref()?;
    if !wallpaper.is_file() {
        return None;
    }

    let cache_path = cache_path_for(wallpaper)?;
    if cache_path.is_file() {
        return Some(cache_path);
    }

    let image = ImageReader::open(wallpaper)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let (width, height) = image.dimensions();
    let scale = (GLASS_BLUR_MAX_EDGE as f32 / width.max(height).max(1) as f32).min(1.0);
    let resized = image.resize(
        (width as f32 * scale).round().max(1.0) as u32,
        (height as f32 * scale).round().max(1.0) as u32,
        FilterType::Triangle,
    );
    let glass_blur = glass_blur_material_image(resized.blur(24.0));

    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).ok()?;
    }
    glass_blur.save(&cache_path).ok()?;
    Some(cache_path)
}

fn glass_blur_material_image(image: DynamicImage) -> DynamicImage {
    let mut image = image.to_rgba8();
    for pixel in image.pixels_mut() {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        let luma = luminance(r, g, b);
        let target_luma = compressed_glass_blur_luma(luma);
        let scale = if luma <= 0.001 {
            0.0
        } else {
            target_luma / luma
        };
        let channels = [
            glass_blur_channel(r, luma, scale, 0.095),
            glass_blur_channel(g, luma, scale, 0.082),
            glass_blur_channel(b, luma, scale, 0.064),
        ];
        pixel[0] = normalized_channel(channels[0]);
        pixel[1] = normalized_channel(channels[1]);
        pixel[2] = normalized_channel(channels[2]);
    }
    DynamicImage::ImageRgba8(image)
}

fn luminance(r: f32, g: f32, b: f32) -> f32 {
    r * 0.2126 + g * 0.7152 + b * 0.0722
}

fn compressed_glass_blur_luma(luma: f32) -> f32 {
    if luma > 0.5 {
        0.5 + (luma - 0.5) * 0.18
    } else {
        luma * 0.82
    }
    .clamp(0.045, 0.58)
}

fn glass_blur_channel(channel: f32, luma: f32, scale: f32, ambient: f32) -> f32 {
    let toned = (luma + (channel - luma) * 1.16) * scale;
    toned * 0.92 + ambient * 0.08
}

fn normalized_channel(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn load_wallpaper_average(path: &Path) -> Option<[u8; 3]> {
    let image = ImageReader::open(path).ok()?.decode().ok()?.to_rgb8();
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return None;
    }

    let step_x = (width / 96).max(1);
    let step_y = (height / 96).max(1);
    let mut total = [0u64; 3];
    let mut count = 0u64;

    for y in (0..height).step_by(step_y as usize) {
        for x in (0..width).step_by(step_x as usize) {
            let pixel = image.get_pixel(x, y);
            total[0] += pixel[0] as u64;
            total[1] += pixel[1] as u64;
            total[2] += pixel[2] as u64;
            count += 1;
        }
    }

    (count > 0).then_some([
        (total[0] / count) as u8,
        (total[1] / count) as u8,
        (total[2] / count) as u8,
    ])
}

fn glass_palette_from_wallpaper(average: [u8; 3]) -> MaterialPalette {
    let dark = MaterialColor::rgba(10, 11, 11, 255);
    let soft = MaterialColor::rgba(36, 37, 34, 255);
    let bright = MaterialColor::rgba(246, 244, 232, 255);
    let source = MaterialColor::rgba(average[0], average[1], average[2], 255);
    let accent = readable_accent(source);

    MaterialPalette {
        panel: mix_color(source, dark, 0.68).with_alpha(0.34),
        panel_control: mix_color(source, soft, 0.56).with_alpha(0.22),
        panel_text: mix_color(accent, bright, 0.74).with_alpha(0.97),
        dock: mix_color(source, dark, 0.64).with_alpha(0.24),
        accent,
        text_soft: mix_color(accent, bright, 0.62).with_alpha(0.9),
        text_muted: mix_color(accent, MaterialColor::rgba(150, 150, 142, 255), 0.54)
            .with_alpha(0.84),
    }
}

fn readable_accent(source: MaterialColor) -> MaterialColor {
    let luma = luminance(
        source.r as f32 / 255.0,
        source.g as f32 / 255.0,
        source.b as f32 / 255.0,
    );
    let lift = if luma < 0.32 { 0.48 } else { 0.32 };
    mix_color(source, MaterialColor::rgba(255, 245, 210, 255), lift)
}

fn mix_color(left: MaterialColor, right: MaterialColor, amount: f32) -> MaterialColor {
    let amount = amount.clamp(0.0, 1.0);
    MaterialColor::rgba(
        mix_channel(left.r, right.r, amount),
        mix_channel(left.g, right.g, amount),
        mix_channel(left.b, right.b, amount),
        mix_channel(left.a, right.a, amount),
    )
}

fn mix_channel(left: u8, right: u8, amount: f32) -> u8 {
    (left as f32 + (right as f32 - left as f32) * amount).round() as u8
}

fn cache_path_for(wallpaper: &Path) -> Option<PathBuf> {
    let paths = ConfigPaths::discover().ok()?;
    let metadata = fs::metadata(wallpaper).ok()?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let canonical = wallpaper
        .canonicalize()
        .unwrap_or_else(|_| wallpaper.to_path_buf());
    let mut hasher = DefaultHasher::new();
    GLASS_BLUR_CACHE_VERSION.hash(&mut hasher);
    canonical.hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    modified.hash(&mut hasher);
    Some(
        paths
            .cache_home
            .join("materials")
            .join(format!("glass-blur-{:016x}.png", hasher.finish())),
    )
}
