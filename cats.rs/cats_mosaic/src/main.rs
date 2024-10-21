use std::{
    io::{Cursor, Write},
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use base64ct::{Base64, Encoding};
use clap::Parser;
use image::{DynamicImage, ImageReader};
use indexmap::IndexMap;
use rand::seq::SliceRandom;
use reqwest::blocking::multipart;
use sha1::{Digest, Sha1};
use url::Url;
use zip::{write::SimpleFileOptions, ZipWriter};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long, default_value = "http://127.0.0.1:8080")]
    server: Url,

    #[arg(short, long, default_value = "4")]
    fetchers_threads: u32,

    #[arg(short, long, default_value = "4")]
    collage_builders_threads: u32,
}

const COLLAGE_IMAGES_COUNT: usize = 12;

fn bytes_to_sha1(bytes: &[u8]) -> String {
    let digest = Sha1::digest(bytes);
    Base64::encode_string(&digest)
}

fn resize_image_keep_aspect_ratio(image: &DynamicImage, target_width: u32) -> DynamicImage {
    image.resize(target_width, 1024, image::imageops::FilterType::Lanczos3)
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
// fn build_collage(images: [&Image<'_>; COLLAGE_IMAGES_COUNT]) -> RgbImage {
//     let width = 512;
//     let column_width = width / 4;
//     let images = images.map(|image| resize_image_keep_aspect_ratio(image, column_width));

//     let mut columns_images: [Vec<&Image<'_>>; 4] = [
//         vec![&images[0]],
//         vec![&images[1]],
//         vec![&images[2]],
//         vec![&images[3]],
//     ];
//     for image_to_insert in &images[4..] {
//         let (min_height_index, min_height) = columns_images
//             .iter()
//             .map(|v| v.iter().map(|image| image.height()).sum::<u32>())
//             .enumerate()
//             .min_by_key(|(j, height)| *height + image_to_insert.height())
//             .unwrap();
//         columns_images[min_height_index].push(image_to_insert);
//     }

//     let max_height = columns_images
//         .iter()
//         .map(|v| v.iter().map(|image| image.height()).sum::<u32>())
//         .max()
//         .unwrap();
//     let collage_image = Image::new(width, max_height, columns_images[0][0].pixel_type());

//     todo!()
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    let images_cache: Arc<RwLock<IndexMap<String, DynamicImage>>> =
        Arc::new(RwLock::new(IndexMap::new()));
    let client = reqwest::blocking::Client::new();

    let threads_fetchers: Vec<JoinHandle<_>> = (0..args.fetchers_threads).map(|_| {
        let images = images_cache.clone();
        let args = args.clone();
        let client = client.clone();
        let thread = thread::spawn(move || loop {
            let mut resp = client
                .get(args.server.join("/cat").expect("Cannot build destination URL"))
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
    }).collect();

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

                let mut zip_buffer: Vec<u8> = Vec::new();
                pack_images_to_zip(&mut zip_buffer, &collage_images);

                client
                    .post(args.server.join("/cat").expect("Cannot build destination URL"))
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
