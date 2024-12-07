extern crate image;
extern crate num;

use glob::glob;
use imageproc::filter::median_filter;
use clap::Parser;
use image::imageops::{resize, FilterType};
use image::{DynamicImage, ImageBuffer, Luma, ColorType, LumaA};
use num::bigint::BigUint;

use std::path;
use std::{
    fs::File,
    io::{Write, stdout},
    collections::HashMap,
    path::PathBuf
};

#[derive(Debug, Parser)]
#[clap(name = "Yakei", version = "0.3.3", author = "ReYeS")]
pub struct CliCommands {
    #[clap(long, short = 'd')]
    directory: String,
    #[clap(long = "output", short = 'o', default_value_t = String::from("result.txt"))]
    result_path: String,
    #[clap(long = "median", short = 'm', default_value_t = false)]
    use_median_filter: bool,
    #[clap(long = "median-radius", short = 'l', default_value_t = 2)]
    radius: u32,
    #[clap(long = "hash-size", short = 's', default_value_t = 8)]
    size: u32,
    #[clap(long = "min-percentage", short = 'p', default_value_t = 80.0)]
    minimal_percentage: f64,
    #[clap(long, short = 'r', default_value_t = false)]
    recursive: bool
}

fn arith(img: &mut ImageBuffer<Luma<u8>, Vec<u8>>, n: u32) -> u8 {
    let mut arithm: u32 = 0;
    for y in 0..n {
        for x in 0..n {
            arithm += img.get_pixel(x, y).0[0] as u32;
        }
    }
    return (arithm / (n * n)) as u8;
}

fn arith_alpha(img: &mut ImageBuffer<LumaA<u8>, Vec<u8>>, n: u32) -> u8 {
    let mut arithm: u32 = 0;
    for y in 0..n {
        for x in 0..n {
            arithm += img.get_pixel(x, y).0[0] as u32;
        }
    }
    return (arithm / (n * n)) as u8;
}

fn to_hash(array: &Vec<u8>) -> String {
    let mut bin_string = String::new();
    for c in array {
        bin_string.push_str(&c.to_string());
    }
    
    match bin_string.len() {
        0..=8 => format!("{:016X}", u8::from_str_radix(bin_string.as_str(), 2).unwrap()).to_owned(),
        9..=16 => format!("{:016X}", u16::from_str_radix(bin_string.as_str(), 2).unwrap()).to_owned(),
        17..=32 => format!("{:016X}", u32::from_str_radix(bin_string.as_str(), 2).unwrap()).to_owned(), 
        33..=64 => format!("{:016X}", u64::from_str_radix(bin_string.as_str(), 2).unwrap()).to_owned(),
        65..=128 => format!("{:016X}", u128::from_str_radix(bin_string.as_str(), 2).unwrap()).to_owned(), 
        _ => format!("{:016X}", BigUint::parse_bytes(bin_string.as_bytes(), 2).unwrap()).to_owned()
    }
}

fn imghash(filename: &PathBuf, radius: u32, size: u32, do_median: bool) -> Vec<u8> {
    // let binding = image::open(&filename).unwrap().grayscale();
    let binding = image::io::Reader::open(filename);
    match binding {
        Ok(binding) => {
            match binding.with_guessed_format() {
                Ok(binding) => {
                    match binding.decode() {
                        Ok(binding) => {
                            let binding = binding.grayscale();
                            if binding.color() == ColorType::La8 {
                                let binding = binding.as_luma_alpha8();

                                match binding {
                                    Some(binding) => {
                                        let binding = match do_median {
                                            true => median_filter(&binding, radius, radius),
                                            false => binding.to_owned(),
                                        };
                                    
                                        let mut binding: DynamicImage = DynamicImage::ImageLumaA8(resize(&binding, size, size, FilterType::Lanczos3));
                                        let mut binding: &mut ImageBuffer<LumaA<u8>, Vec<u8>> = binding.as_mut_luma_alpha8().unwrap();
                                    
                                        let arithm = arith_alpha(&mut binding, size);
                                        
                                        let mut array: Vec<u8> = Vec::new();
                                    
                                        for y in 0..size {
                                            for x in 0..size {
                                                if binding.get_pixel(x, y).0[0] > arithm {
                                                    array.push(1);
                                                }
                                                else {
                                                    array.push(0);
                                                }
                                            }
                                        }
                                    
                                        return array;
                                    },
                                    None => { println!("{:?} - Something wrong.", filename); return vec![0; size as usize]; }
                                }
                            } else {
                                let binding = binding.as_luma8();
                                match binding {
                                    Some(binding) => {
                                        let binding = match do_median {
                                            true => median_filter(&binding, radius, radius),
                                            false => binding.to_owned(),
                                        };
                                    
                                        let mut binding: DynamicImage = DynamicImage::ImageLuma8(resize(&binding, size, size, FilterType::Lanczos3));
                                        let mut binding: &mut ImageBuffer<Luma<u8>, Vec<u8>> = binding.as_mut_luma8().unwrap();
                                    
                                        let arithm = arith(&mut binding, size);
                                        
                                        let mut array: Vec<u8> = Vec::new();
                                    
                                        for y in 0..size {
                                            for x in 0..size {
                                                if binding.get_pixel(x, y).0[0] > arithm {
                                                    array.push(1);
                                                }
                                                else {
                                                    array.push(0);
                                                }
                                            }
                                        }
                                    
                                        return array;
                                    },
                                    None => { println!("{:?} - Something wrong.", filename); return vec![0; size as usize]; },
                                }
                            };
                        },
                        Err(error) => { println!("{:?} - {:#?}", filename, error); return vec![0; size as usize]; }
                    }
                },
                Err(error) => { println!("{:?} - {:#?}", filename, error); return vec![0; size as usize]; }
            }
        },
        Err(error) => { println!("{:?} - {:#?}", filename, error); return vec![0; size as usize]; }
    };
}

fn calculate_percents(first: &[u8], second: &[u8]) -> f64 {
    let mut equals: f64 = 0.0;
    for i in 0..(first.len()-1) {
        if i <= second.len() {
            if first[i] == second[i] {
                equals += 1.0;
            }
        }
    }

    return (equals / first.len() as f64) * 100.0;
}

fn main() {
    let cmd_args: CliCommands = CliCommands::parse();

    let mut original_files: Vec<PathBuf> = Vec::new();
    
    let mut path: String = cmd_args.directory.clone();
    let separator: &str = if path.contains("\\") {"\\"} else {"/"};
    if path.as_str().ends_with(separator) {
        path.push_str("*");
    }
    else {
        path.push_str(separator);
        path.push_str("*");
    }
    
    println!("Checking folder.");
    let mut file_counter: usize = 0;
    for i in glob(path.as_str()).unwrap() {
        match i {
            Ok(path) => {
                let mimetype: Result<Option<infer::Type>, std::io::Error> = infer::get_from_path(path.to_str().unwrap());
                match &mimetype {
                    Ok(Some(_)) => {},
                    Ok(None) => { continue; },
                    Err(_) => { continue; }
                }
                let mimetype: String = mimetype.unwrap().unwrap().to_string();
                let mimetype: Vec<&str> = mimetype.split("/").collect::<Vec<&str>>();

                if mimetype[1] != "gif" {
                    if mimetype[0] == "image" {
                        original_files.push(path.clone());
                    }
                }
                file_counter += 1;
                print!("{}\r", file_counter);
                stdout().flush().unwrap();
            },
            Err(_) => (),
        }
    }
    
    file_counter = 0;
    if cmd_args.recursive == true {
        println!("Get files from subdirectories");
        path.push_str("*/*");
        
        for i in glob(path.as_str()).unwrap() {
            match i {
                Ok(path) => {
                    if !original_files.contains(&path) {
                        let mimetype: Result<Option<infer::Type>, std::io::Error> = infer::get_from_path(path.to_str().unwrap());
                        match &mimetype {
                            Ok(Some(_)) => {},
                            Ok(None) => { continue; },
                            Err(_) => { continue; }
                        }
                        let mimetype: String = mimetype.unwrap().unwrap().to_string();
                        let mimetype: Vec<&str> = mimetype.split("/").collect::<Vec<&str>>();
                        
                        if mimetype[1] != "gif" {
                            if mimetype[0] == "image" {
                                original_files.push(path.clone());
                            }
                        }
                        file_counter += 1;
                        print!("{}\r", file_counter);
                        stdout().flush().unwrap();
                    }
                },
                Err(_) => (),
            }
        }
    }
    
    file_counter = 0;
    println!("Calculating hashes for images.");
    let mut hashes_file: File = File::create("image_hashes.txt").expect("Unable to create file");
    let mut images_dict = HashMap::new();
    for image_file in &original_files {
        let image_hash = imghash(&image_file, cmd_args.radius, cmd_args.size, cmd_args.use_median_filter);
        images_dict.insert(image_file.to_str().unwrap(), image_hash.clone());
        hashes_file.write(format!("{} | {}\n", to_hash(&image_hash), path::absolute(image_file).unwrap().display()).as_bytes()).unwrap();

        file_counter += 1;
        print!("{}/{}\r", file_counter, original_files.len());
        stdout().flush().unwrap();
    }
    drop(hashes_file);
    
    println!("Checking for duplicates.");
    let mut result_file: File = File::create(cmd_args.result_path.clone()).expect("Unable to create file");
    let mut second_position: u32 = 1;
    for image_path in &original_files {
        let image_path: &str = image_path.to_str().unwrap();
        let first_hash: Vec<u8> = images_dict.get(image_path).unwrap().to_vec();
        println!("\n------------------------------------------------------+");
        images_dict.remove(image_path);

        for second_path in images_dict.keys() {
            let second_hash = images_dict.get(second_path).unwrap();
            let hash_percents: f64 = calculate_percents(&first_hash, &second_hash);

            println!("{:4}/{:4} - {:4} - {}/{} | {} {} | {} {}", second_position, images_dict.len(), original_files.len() - images_dict.len(), (first_hash == *second_hash), hash_percents, to_hash(&first_hash), to_hash(&second_hash), image_path, second_path);

            if (first_hash == *second_hash) || hash_percents > cmd_args.minimal_percentage {
                result_file.write(format!("{:16} - {:5} | {} {} | {} {}\n", hash_percents, (first_hash == *second_hash), to_hash(&first_hash), to_hash(&second_hash), image_path, second_path).as_bytes()).unwrap();
            }
            second_position += 1;
            stdout().flush().unwrap();
        }
        second_position = 1;
    }

    if cmd_args.size < 8 {
        println!("You serious? Ok, have fun.");
    }
}
