use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn set_readonly(path: &PathBuf, readonly: bool) -> () {
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(readonly);
    fs::set_permissions(&path, permissions).expect(&format!("setting permissions on {:?}", path))
}

pub fn force_remove_file(path: &PathBuf) -> () {
    set_readonly(&path, false);
    fs::remove_file(&path).expect(&format!("removing file {:?}", path));
}

fn force_remove_dir(path: &PathBuf) -> () {
    set_readonly(&path, false);
    fs::remove_dir(&path).expect(&format!("removing dir {:?}", path));
}

fn force_remove_dir_all(path: &PathBuf) -> () {
    // We can't remove files in this directory if it is readonly.
    set_readonly(&path, false);
    for child in fs::read_dir(&path).unwrap() {
        let child = child.unwrap();
        let metadata = child.metadata().unwrap();
        let path = child.path();
        if metadata.is_dir() {
            force_remove_dir_all(&path);
        } else if metadata.is_file() {
            force_remove_file(&path);
        };
    };
    force_remove_dir(&path);
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let gopath = format!("{}/gopath", out_dir);

    Command::new("go")
        .env("GOPATH", gopath.clone())
        .arg("get")
        .arg("./");

    Command::new("go")
        .env("GOPATH", gopath.clone())
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(format!("{}/{}", out_dir, "libesbuild.a"))
        .arg("esbuild.go")
        .status()
        .expect("compile Go library");

    // Otherwise Cargo will complain that we've modified files outside OUT_DIR.
    fs::remove_file("go.sum").expect("remove go.sum");
    // Go package manager makes dependency files read only, causing issues with rebuilding and
    // clearing.
    force_remove_dir_all(&PathBuf::from(gopath));

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", "esbuild");
}
