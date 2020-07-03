// https://docs.rs/image/0.23.6/image/fn.open.html
// http://gudok.xyz/thumbnail/#:~:text=When%20you%20scale%20images%2C%20you,single%20pixel%20of%20the%20thumbnail.

use image::{
    imageops::{
        unsharpen,
        FilterType::{CatmullRom, Triangle},
    },
    GenericImageView,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

fn read_directory(directory: &str) -> Vec<PathBuf> {
    let mut jpegs = Vec::<PathBuf>::new();

    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                // entry is `DirEntry` now
                let entry_path = entry.path();
                if entry_path.extension().unwrap() == "jpg" {
                    jpegs.push(entry_path);
                }
            } else {
                eprintln!("[Error] Error parsing entry.");
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("[Error] Could not read the directory.");
        std::process::exit(1);
    }

    jpegs
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("[Error] Invalid number of arguments.");
        std::process::exit(1);
    }

    let input_ratio: f32 = args[1].parse().unwrap();
    let images_dir = &args[2];
    let output_dir = &args[3];

    let jpegs_entries = read_directory(images_dir);

    let mut thread_handlers = vec![];

    for i in 0..jpegs_entries.len() {
        thread_handlers.push(thread::spawn({
            let jpegs_clone = jpegs_entries.clone();
            let output_d = output_dir.clone();
            let mut ratio = input_ratio;

            move || {
                let mut img = image::open(&jpegs_clone[i]).unwrap();

                while ratio < 0.5 {
                    let (w, h) = img.dimensions();
                    img = img.resize(w / 2, h / 2, Triangle);
                    ratio *= 2.0;
                }

                let (w, h) = img.dimensions();

                if (w as f32) * ratio != w as f32 {
                    img = img.resize(
                        ((w as f32) * ratio) as u32,
                        ((h as f32) * ratio) as u32,
                        CatmullRom,
                    );
                }

                let unsharped = unsharpen(&img, 1.5, 5);

                let out = Path::new(&output_d).join(&jpegs_clone[i].file_name().unwrap());

                match unsharped.save(out) {
                    Ok(t) => t,
                    Err(_) => eprintln!("[Error] The system cannot find the output directory."),
                }
            }
        }));
    }

    for handle in thread_handlers {
        handle.join().unwrap();
    }
}
