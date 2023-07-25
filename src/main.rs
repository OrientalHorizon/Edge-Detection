mod rt_weekend;
mod vec3;
use console::style;
use indicatif::ProgressBar;
use std::{fs::File, process::exit};

use image::{GenericImageView, ImageBuffer, RgbImage};
use vec3::{Color3, Vec3};

pub fn gray(v: &Vec3) -> f64 {
    v.x().max(v.y()).max(v.z())
}
pub fn write_color(pixel_color: &Color3, samples_per_pixel: u32) -> [u8; 3] {
    let mut r: f64 = pixel_color.x();
    let mut g: f64 = pixel_color.y();
    let mut b: f64 = pixel_color.z();

    // Divide the color by the number of samples.
    let scale: f64 = 1.0 / samples_per_pixel as f64;
    r = (scale * r).sqrt();
    g = (scale * g).sqrt();
    b = (scale * b).sqrt();

    // Write the translated [0,255] value of each color component.
    [
        (256.0 * rt_weekend::clamp(r, 0.0, 0.999)) as u8,
        (256.0 * rt_weekend::clamp(g, 0.0, 0.999)) as u8,
        (256.0 * rt_weekend::clamp(b, 0.0, 0.999)) as u8,
    ]
}

fn main() {
    let path = std::path::Path::new("output/gen-sobel.jpg");
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");

    let img = image::open("image21.jpg").expect("Failed to open image");
    let width: usize = img.width() as usize;
    let height: usize = img.height() as usize;
    // let mut data: Vec<Vec<Vec3>> = Vec::new();
    let mut original: Vec<Vec<f64>> = Vec::new();
    let mut gen: Vec<Vec<f64>> = Vec::new();
    for _i in 0..width {
        let mut row: Vec<f64> = Vec::new();
        for _j in 0..height {
            row.push(0.0);
        }
        original.push(row.clone());
        gen.push(row);
    }
    for i in 0..width {
        for j in 0..height {
            let pixel = img.get_pixel(i as u32, j as u32);
            let (r, g, b) = (pixel[0] as f64, pixel[1] as f64, pixel[2] as f64);
            original[i][j] = gray(&Vec3::construct(&[r, g, b]));
            // if i == width / 2 && j == width / 2 {
            //     println!("{} {} {}", r, g, b);
            // }
        }
    }
    // Laplace
    // for i in 0..width {
    //     for j in 0..height {
    //         if i == 0 || i == width - 1 || j == 0 || j == height - 1 {
    //             gen[i][j] = 0f64;
    //             continue;
    //         }

    //         let upleft = original[i - 1][j - 1];
    //         let up = original[i - 1][j];
    //         let upright = original[i - 1][j + 1];
    //         let left = original[i][j - 1];
    //         let center = original[i][j];
    //         let right = original[i][j + 1];
    //         let downleft = original[i + 1][j - 1];
    //         let down = original[i + 1][j];
    //         let downright = original[i + 1][j + 1];
    //         let sum =
    //             upleft + up + upright + left + right + downleft + down + downright - 8.0 * center;
    //         gen[i][j] = sum / 256.0;
    //     }
    // }

    // Sobel
    let sobel_x: [[f64; 3]; 3] = { [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]] };
    let sobel_y: [[f64; 3]; 3] = { [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]] };
    for i in 0..width {
        for j in 0..height {
            if i == 0 || i == width - 1 || j == 0 || j == height - 1 {
                gen[i][j] = 0f64;
                continue;
            }
            let mut gx = 0.0;
            for k in 0..3 {
                for l in 0..3 {
                    gx += sobel_x[k][l] * original[i + k - 1][j + l - 1];
                }
            }
            let mut gy = 0.0;
            for k in 0..3 {
                for l in 0..3 {
                    gy += sobel_y[k][l] * original[i + k - 1][j + l - 1];
                }
            }
            let ans = (gx * gx + gy * gy).sqrt();
            // println!("{ans}");
            if ans > 58.0 {
                gen[i][j] = 0.0;
            } else {
                gen[i][j] = original[i][j];
                // println!("fuck");
            }
        }
    }

    let quality = 100;
    let mut img: RgbImage = ImageBuffer::new(width as u32, height as u32);

    let progress = if option_env!("CI").unwrap_or_default() == "true" {
        ProgressBar::hidden()
    } else {
        ProgressBar::new((height * width) as u64)
    };

    for i in 0..width {
        for j in 0..height {
            let pixel = img.get_pixel_mut(i as u32, j as u32);
            let mut gray = gen[i][j];
            let rgb = write_color(&Color3::construct(&[gray, gray, gray]), 1);
            *pixel = image::Rgb(rgb);
            progress.inc(1);
        }
    }
    progress.finish();

    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    let output_image = image::DynamicImage::ImageRgb8(img);
    let mut output_file = File::create(path).unwrap();
    match output_image.write_to(&mut output_file, image::ImageOutputFormat::Jpeg(quality)) {
        Ok(_) => {}
        Err(_) => println!("{}", style("Outputing image fails.").red()),
    }

    exit(0);
}
