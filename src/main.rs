use std::io::Cursor;

use clap::Parser;
use gltf::json::image::MimeType;
use image::imageops::FilterType;

/// The most powerful tool for optimizing game assets in the Tri-State Area!
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to the file
    #[arg(short, long)]
    input: String,

    // limits how big textures can be
    #[arg(short, long)]
    max_texture_dimensions: u32,

    // limits how big textures can be
    #[arg(short, long)]
    texture_quality: u32,
}

fn gltf_image_to_image_image(data: &gltf::image::Data) -> image::DynamicImage {
    match data.format {
        gltf::image::Format::R8 => image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(data.width, data.height, data.pixels.clone()).unwrap(),
        ),
        gltf::image::Format::R8G8 => image::DynamicImage::ImageLumaA8(
            image::GrayAlphaImage::from_raw(data.width, data.height, data.pixels.clone()).unwrap(),
        ),
        gltf::image::Format::R8G8B8 => image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(data.width, data.height, data.pixels.clone()).unwrap(),
        ),
        gltf::image::Format::R8G8B8A8 => image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(data.width, data.height, data.pixels.clone()).unwrap(),
        ),
        _ => panic!("input image format not implemented!"),
    }
}

fn main() {
    let args = Args::parse();
    let (document, buffers, image_data) = gltf::import(args.input).unwrap();

    let images = document
        .images()
        .zip(&image_data)
        .map(|(image_view, image_data)| {
            let image = gltf_image_to_image_image(image_data);
            image.resize(
                args.max_texture_dimensions,
                args.max_texture_dimensions,
                FilterType::Gaussian,
            );

            let mut buf = Vec::new();
            let mut cursor = Cursor::new(&mut buf);
            image
                .write_to(
                    &mut cursor,
                    image::ImageOutputFormat::Jpeg(args.texture_quality as u8),
                )
                .unwrap();

            gltf::json::Image {
                buffer_view: None,
                uri: Some(format!(
                    "data:application/octet-stream;base64,{}",
                    base64::encode(&buf)
                )),
                mime_type: Some(MimeType("image/jpeg".into())),
                name: image_view.name().map(|s| s.into()),
                extensions: None,
                extras: image_view.extras().clone(),
            }
        });

    let mut root = document.clone().into_json();
    root.buffers = buffers
        .iter()
        .map(|b| -> gltf::json::Buffer {
            gltf::json::Buffer {
                byte_length: b.0.len() as u32,
                name: None,
                uri: Some(format!(
                    "data:application/octet-stream;base64,{}",
                    base64::encode(&b.0)
                )),
                extensions: None,
                extras: gltf::json::extras::Void::default(),
            }
        })
        .collect();

    root.images = images.collect();

    let mut out = std::fs::File::create("smolinator_output.gltf").unwrap();
    root.to_writer_pretty(&mut out).unwrap();
}
