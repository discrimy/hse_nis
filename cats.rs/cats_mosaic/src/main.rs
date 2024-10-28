use std::{
    io::{Cursor, Write},
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use base64ct::{Base64, Encoding};
use clap::Parser;
use image::{imageops, DynamicImage, ImageBuffer, ImageReader, Pixel, Rgb, Rgba};
use indexmap::IndexMap;
use rand::seq::SliceRandom;
use reqwest::blocking::multipart;
use sha1::{Digest, Sha1};
use url::Url;
use zip::{write::SimpleFileOptions, ZipWriter};

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum OutputFormat {
    #[default]
    ZIP,
    PNG_COLLAGE,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long, default_value = "http://127.0.0.1:8080")]
    server: Url,

    #[arg(short, long, default_value = "4")]
    fetchers_threads: u32,

    #[arg(short, long, default_value = "4")]
    collage_builders_threads: u32,

    #[arg(short, long, default_value = "zip")]
    output_format: OutputFormat,
}

const COLLAGE_IMAGES_COUNT: usize = 12;

fn bytes_to_sha1(bytes: &[u8]) -> String {
    let digest = Sha1::digest(bytes);
    Base64::encode_string(&digest)
}

fn pack_images_to_zip(buffer: &mut Vec<u8>, images: &[&DynamicImage]) {
    let mut cursor = Cursor::new(buffer);
    let mut zip = ZipWriter::new(&mut cursor);

    for (index, image) in images.iter().enumerate() {
        let options = SimpleFileOptions::default();
        zip.start_file(format!("cat{}.jpeg", index), options)
            .expect("Cannot start writing to file in ZIP archive");
        let mut image_bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut image_bytes), image::ImageFormat::Jpeg)
            .expect("Cannot convert image to JPEG");
        zip.write_all(&image_bytes)
            .expect("Cannot write to file in ZIP archive");
    }

    zip.finish().expect("Cannot write to file");
}

fn build_collage(images: &[&DynamicImage]) -> DynamicImage {
    let columns: u32 = 4;
    let collage_width_base: u32 = 512;
    let padding_columns: u32 = 10;
    let padding_rows_min: u32 = 10;
    let collage_margin: u32 = 10;

    let width = collage_width_base + padding_columns * (columns - 1) + collage_margin * 2;
    let column_width = collage_width_base / columns;
    let images: Vec<DynamicImage> = images
        .iter()
        .map(|image| {
            image.resize(
                column_width,
                image.height(),
                image::imageops::FilterType::Lanczos3,
            )
        })
        .collect();

    let mut columns_images: [Vec<&DynamicImage>; 4] = [
        vec![&images[0]],
        vec![&images[1]],
        vec![&images[2]],
        vec![&images[3]],
    ];
    for image_to_insert in &images[4..] {
        let (min_height_index, min_height) = columns_images
            .iter()
            .map(|v| v.iter().map(|image| image.height()).sum::<u32>())
            .enumerate()
            .min_by_key(|(j, height)| *height + image_to_insert.height())
            .unwrap();
        columns_images[min_height_index].push(image_to_insert);
    }

    let max_height_column_images = columns_images
        .iter()
        .max_by_key(|v| v.iter().map(|image| image.height()).sum::<u32>())
        .unwrap();
    let max_height_base = max_height_column_images
        .iter()
        .map(|image| image.height())
        .sum::<u32>()
        + (padding_rows_min * (max_height_column_images.len() as u32 - 1));
    let max_height = max_height_base 
    + collage_margin * 2;

    let mut collage_image = ImageBuffer::from_pixel(width, max_height, Rgba([255, 255, 255, 255]));

    let mut current_x = collage_margin;
    for column_images in columns_images {
        let images_height: u32 = column_images.iter().map(|im| im.height()).sum();
        let padding_height: u32 = (max_height_base - images_height) / (column_images.len() as u32 - 1);

        let mut current_y: u32 = collage_margin;
        for image in column_images {
            imageops::overlay(
                &mut collage_image,
                image,
                current_x as i64,
                current_y as i64,
            );
            current_y += image.height() + padding_height;
        }

        current_x += column_width + padding_columns;
    }

    DynamicImage::ImageRgba8(collage_image)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    let images_cache: Arc<RwLock<IndexMap<String, DynamicImage>>> =
        Arc::new(RwLock::new(IndexMap::new()));
    let client = reqwest::blocking::Client::new();

    let threads_fetchers: Vec<JoinHandle<_>> = (0..args.fetchers_threads)
        .map(|_| {
            let images = images_cache.clone();
            let args = args.clone();
            let client = client.clone();
            let thread = thread::spawn(move || loop {
                let mut resp = client
                    .get(
                        args.server
                            .join("/cat")
                            .expect("Cannot build destination URL"),
                    )
                    .send()
                    .expect("Error while sending request")
                    .error_for_status()
                    .expect("Unexpected response status")
                    .bytes()
                    .expect("Cannot parse body to bytes");
                let image_hash = bytes_to_sha1(&resp);

                let cursor = Cursor::new(&mut resp);
                let mut image_reader = ImageReader::new(cursor);
                image_reader.set_format(image::ImageFormat::Jpeg);
                let image = image_reader.decode().expect("Cannot decode image");

                let image_exists = {
                    let images = images.read().unwrap();
                    images.contains_key(&image_hash)
                };
                if !image_exists {
                    let mut images = images.write().unwrap();
                    images.insert(image_hash, image);

                    println!("Found new image! Current size: {}", images.len());
                }
            });
            thread
        })
        .collect();

    let threads_collage_builders: Vec<JoinHandle<_>> = (0..args.collage_builders_threads)
        .map(|_| {
            let images_cache = images_cache.clone();
            let args = args.clone();
            let client = client.clone();
            let thread = thread::spawn(move || loop {
                let images = images_cache.read().unwrap();
                if images.len() < COLLAGE_IMAGES_COUNT {
                    continue;
                }
                println!("Enough images have been collected");
                let mut rng = rand::thread_rng();
                let mut indexes: Vec<usize> = (0..images.len()).collect();
                indexes.shuffle(&mut rng);
                let collage_indexes = &indexes[..COLLAGE_IMAGES_COUNT];
                let collage_images: Vec<&DynamicImage> = collage_indexes
                    .iter()
                    .map(|i| images.get_index(*i).unwrap().1)
                    .collect();

                match args.output_format {
                    OutputFormat::ZIP => {
                        let mut zip_buffer: Vec<u8> = Vec::new();
                        pack_images_to_zip(&mut zip_buffer, &collage_images);
                        client
                            .post(
                                args.server
                                    .join("/cat")
                                    .expect("Cannot build destination URL"),
                            )
                            .multipart(
                                multipart::Form::new().part(
                                    "file",
                                    multipart::Part::bytes(zip_buffer)
                                        .file_name("cats.zip")
                                        .mime_str("application/zip")
                                        .expect("Cannot format POST file"),
                                ),
                            )
                            .send()
                            .expect("Error while sending request")
                            .error_for_status()
                            .expect("Unexpected response status");
                    }
                    OutputFormat::PNG_COLLAGE => {
                        let collage_image = build_collage(&collage_images[..]);
                        let mut collage_image_png: Vec<u8> = Vec::new();
                        collage_image
                            .write_to(
                                &mut Cursor::new(&mut collage_image_png),
                                image::ImageFormat::Png,
                            )
                            .expect("Cannot encode image as PNG");
                        client
                            .post(
                                args.server
                                    .join("/cat")
                                    .expect("Cannot build destination URL"),
                            )
                            .multipart(
                                multipart::Form::new().part(
                                    "file",
                                    multipart::Part::bytes(collage_image_png)
                                        .file_name("collage.png")
                                        .mime_str("image/png")
                                        .expect("Cannot format POST file"),
                                ),
                            )
                            .send()
                            .expect("Error while sending request")
                            .error_for_status()
                            .expect("Unexpected response status");
                    }
                }
                println!("Archive has been created");
            });
            thread
        })
        .collect();

    for thread in threads_fetchers {
        thread.join().expect("Cannot join thread");
    }
    for thread in threads_collage_builders {
        thread.join().expect("Cannot join thread");
    }
    // let resp = reqwest::blocking::get(args.server)?.error_for_status()?.bytes()?;
    // File::create("image.jpeg")?.write(&resp)?;
    Ok(())
}
