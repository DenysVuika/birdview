use std::{
    env::var,
    fs::{copy, create_dir_all, read_dir},
    io,
    path::{Path, PathBuf},
};

fn main() {
    let cargo_dir = var("CARGO_MANIFEST_DIR").unwrap();
    let profile = var("PROFILE").unwrap();
    let asset_dir = PathBuf::from(cargo_dir).join("static");
    let output_path = get_output_path();
    // println!(
    //     "cargo:warning=Calculated build path: {}",
    //     output_path.to_str().unwrap()
    // );

    if profile == "release" {
        copy_dir_all(asset_dir, output_path.join("static")).unwrap();
    }
}

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = var("PROFILE").unwrap();

    Path::new(&manifest_dir_string)
        .join("target")
        .join(build_type)
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    create_dir_all(&dst)?;

    for entry in read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    Ok(())
}
