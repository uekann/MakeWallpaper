use clap::Parser;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use imageproc::filter::gaussian_blur_f32;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
struct Style {
    width: u32,
    height: u32,
    menubar_height: u32,
    blur: Option<f32>,
    round_corners: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct StylesConfig {
    #[serde(flatten)]
    styles: HashMap<String, Style>,
}

#[derive(Parser, Debug)]
#[command(name = "make_wallpaper")]
#[command(about = "Convert images to Mac wallpapers with menu bar blackout and optional effects")]
struct Args {
    /// Input image path (JPEG or PNG)
    input_image: PathBuf,

    /// Style template name
    #[arg(short = 't', long)]
    style: String,

    /// Path to styles config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Override output resolution (e.g., 1920x1080)
    #[arg(short, long, value_parser = parse_size)]
    size: Option<(u32, u32)>,

    /// Override menubar height
    #[arg(short, long)]
    menubar_height: Option<u32>,

    /// Override blur radius
    #[arg(short, long, num_args = 0..=1, default_missing_value = "10")]
    blur: Option<f32>,

    /// Override corner radius
    #[arg(short, long, num_args = 0..=1, default_missing_value = "20")]
    round_corners: Option<u32>,

    /// Output directory (default: current directory)
    #[arg(short, long)]
    out_dir: Option<PathBuf>,
}

fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err("Size must be in format WIDTHxHEIGHT (e.g., 1920x1080)".to_string());
    }
    let width = parts[0]
        .parse::<u32>()
        .map_err(|_| "Invalid width".to_string())?;
    let height = parts[1]
        .parse::<u32>()
        .map_err(|_| "Invalid height".to_string())?;
    Ok((width, height))
}

fn find_config_path() -> Option<PathBuf> {
    let candidates = [
        env::var("XDG_CONFIG_HOME")
            .ok()
            .map(|p| PathBuf::from(p).join("make_wallpaper").join("styles.toml")),
        dirs::home_dir().map(|p| p.join(".config").join("make_wallpaper").join("styles.toml")),
        dirs::home_dir().map(|p| p.join(".make_wallpaper").join("styles.toml")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn load_styles(config_path: Option<PathBuf>) -> Result<StylesConfig, Box<dyn std::error::Error>> {
    let path = if let Some(p) = config_path {
        if !p.exists() {
            return Err(format!("Specified config file not found: {}", p.display()).into());
        }
        p
    } else {
        find_config_path().ok_or_else(|| {
            "Config file not found. Searched in:\n  \
             - $XDG_CONFIG_HOME/make_wallpaper/styles.toml\n  \
             - ~/.config/make_wallpaper/styles.toml\n  \
             - ~/.make_wallpaper/styles.toml\n\
             Please create a styles.toml file in one of these locations."
        })?
    };

    let content = fs::read_to_string(&path)?;
    let config: StylesConfig = toml::from_str(&content)?;
    Ok(config)
}

fn resize_and_crop(img: &DynamicImage, target_width: u32, target_height: u32) -> RgbaImage {
    let (src_width, src_height) = img.dimensions();
    let target_aspect = target_width as f64 / target_height as f64;
    let src_aspect = src_width as f64 / src_height as f64;

    let (crop_width, crop_height) = if src_aspect > target_aspect {
        ((src_height as f64 * target_aspect) as u32, src_height)
    } else {
        (src_width, (src_width as f64 / target_aspect) as u32)
    };

    let x = (src_width - crop_width) / 2;
    let y = (src_height - crop_height) / 2;

    let cropped = img.crop_imm(x, y, crop_width, crop_height);
    cropped
        .resize_exact(
            target_width,
            target_height,
            image::imageops::FilterType::Lanczos3,
        )
        .to_rgba8()
}

fn blackout_menubar(img: &mut RgbaImage, menubar_height: u32) {
    let width = img.width();
    for y in 0..menubar_height.min(img.height()) {
        for x in 0..width {
            img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
    }
}

fn apply_blur(img: &RgbaImage, radius: f32) -> RgbaImage {
    gaussian_blur_f32(img, radius)
}

fn apply_round_corners(img: &mut RgbaImage, radius: u32, menubar_height: u32) {
    let width = img.width();
    let height = img.height();
    let radius = radius.min(width / 2).min(height / 2);

    // Top-left corner (starts below menubar)
    let top_y_start = menubar_height;
    for y in top_y_start..(top_y_start + radius) {
        for x in 0..radius {
            let dx = radius - x;
            let dy = radius - (y - top_y_start);
            if dx * dx + dy * dy > radius * radius {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
    }

    // Top-right corner (starts below menubar)
    for y in top_y_start..(top_y_start + radius) {
        for x in (width - radius)..width {
            let dx = x - (width - radius - 1);
            let dy = radius - (y - top_y_start);
            if dx * dx + dy * dy > radius * radius {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
    }

    // Bottom-left corner
    for y in (height - radius)..height {
        for x in 0..radius {
            let dx = radius - x;
            let dy = y - (height - radius - 1);
            if dx * dx + dy * dy > radius * radius {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
    }

    // Bottom-right corner
    for y in (height - radius)..height {
        for x in (width - radius)..width {
            let dx = x - (width - radius - 1);
            let dy = y - (height - radius - 1);
            if dx * dx + dy * dy > radius * radius {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let styles_config = load_styles(args.config.clone())?;
    let base_style = styles_config
        .styles
        .get(&args.style)
        .ok_or_else(|| {
            let available: Vec<_> = styles_config.styles.keys().collect();
            format!(
                "Style '{}' not found. Available styles: {:?}",
                args.style, available
            )
        })?
        .clone();

    let (target_width, target_height) = args.size.unwrap_or((base_style.width, base_style.height));
    let menubar_height = args.menubar_height.unwrap_or(base_style.menubar_height);
    let blur = args.blur.or(base_style.blur);
    let round_corners = args.round_corners.or(base_style.round_corners);

    let img = image::open(&args.input_image)?;

    let mut result = resize_and_crop(&img, target_width, target_height);

    if let Some(radius) = blur {
        result = apply_blur(&result, radius);
    }

    blackout_menubar(&mut result, menubar_height);

    if let Some(radius) = round_corners {
        apply_round_corners(&mut result, radius, menubar_height);
    }

    let output_dir = args.out_dir.unwrap_or_else(|| PathBuf::from("."));
    let input_stem = args
        .input_image
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let mut overrides = Vec::new();
    if args.size.is_some() {
        overrides.push(format!("{}x{}", target_width, target_height));
    }
    if args.menubar_height.is_some() {
        overrides.push(format!("m{}", menubar_height));
    }
    if args.blur.is_some() {
        if let Some(b) = blur {
            overrides.push(format!("b{}", b));
        }
    }
    if args.round_corners.is_some() {
        if let Some(r) = round_corners {
            overrides.push(format!("r{}", r));
        }
    }

    let filename = if overrides.is_empty() {
        format!("{}_{}.png", input_stem, args.style)
    } else {
        format!("{}_{}_{}.png", input_stem, args.style, overrides.join("_"))
    };
    let output_path = output_dir.join(filename);

    result.save(&output_path)?;
    println!("Saved to: {}", output_path.display());

    Ok(())
}
