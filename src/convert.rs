use threadpool::ThreadPool;

use std::{fs, io::Write, process::Command, thread::available_parallelism};

pub fn change_gltf_to_use_ktx2() {
    for path in [
        "./assets/bistro_exterior/BistroExterior.gltf",
        "./assets/bistro_interior_wine/BistroInterior_Wine.gltf",
    ] {
        let contents = fs::read_to_string(path).unwrap();
        let new = contents
            .replace("\"mimeType\":\"image/png\",", "")
            .replace(".png", ".ktx2");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        let _ = file.write(new.as_bytes()).unwrap();
    }
}

pub fn convert_images_to_ktx2() {
    for path in ["./assets/bistro_exterior", "./assets/bistro_interior_wine"] {
        let pool = ThreadPool::new(available_parallelism().unwrap().get());
        for path in fs::read_dir(path).unwrap() {
            pool.execute(move || {
                if let Ok(path) = path {
                    let path = path.path();
                    if path.is_file() && path.extension().unwrap() == "png" {
                        let path_string = path.to_string_lossy().to_string();
                        let new_path_string =
                            path.with_extension("ktx2").to_string_lossy().to_string();
                        let name = path.file_stem().unwrap().to_string_lossy().to_lowercase();
                        let nor = name.contains("Normal");

                        let mut cmd = Command::new("kram");
                        cmd.arg("encode").arg("-f");
                        // should be able to use bc5 for nor and rough+metal, but they looked bad
                        cmd.arg("bc7");
                        if nor {
                            cmd.arg("-normal");
                        }
                        cmd.arg("-type")
                            .arg("2d")
                            .arg("-srgb")
                            .arg("-zstd")
                            .arg("0")
                            .arg("-i")
                            .arg(path_string)
                            .arg("-o")
                            .arg(new_path_string);
                        dbg!(&cmd);
                        cmd.output().expect("ls command failed to start");
                    }
                }
            });
        }
        pool.join();
    }
}
