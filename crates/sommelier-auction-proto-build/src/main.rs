//! Protobuf files in the Sommelier repo, copying the result to the Sommelier_proto crate for import
//! and use. While this builder generates about a dozen files only one contains all the Sommelier
//! proto info and the rest are discarded in favor of upstream cosmos-sdk-proto

// Building new Sommelier rust proto definitions
// run 'cargo run'

use regex::Regex;
use std::{
    ffi::OsStr,
    fs::{self, create_dir_all, remove_dir_all},
    path::PathBuf,
};
use std::{io, path::Path};
use walkdir::WalkDir;

/// Protos belonging to these Protobuf packages will be excluded
/// (i.e. because they are sourced from `tendermint-proto` or `cosmos-sdk-proto`)
const EXCLUDED_PROTO_PACKAGES: &[&str] = &["gogoproto", "google", "tendermint", "cosmos.base"];
/// Regex for locating instances of `cosmos-sdk-proto` in prost/tonic build output
const COSMOS_SDK_PROTO_REGEX: &str = "(super::)+cosmos";

/// the output directory
const TMP_PATH: &str = "/tmp/sommelier-auction-proto-build/";
const OUT_PATH: &str = "../sommelier-auction-proto/src/gen/";
const SOMMELIER_RELEASE_VERSION: &str = "v7.0.1";

// All paths must end with a / and either be absolute or include a ./ to reference the current
// working directory.

fn main() {
    let out_path = Path::new(&OUT_PATH);
    let tmp_path = Path::new(&TMP_PATH);

    // create directories for temporary build dirs
    fs::create_dir_all(tmp_path)
        .unwrap_or_else(|_| panic!("Failed to create {:?}", tmp_path.to_str()));

    let sommelier_repo_endpoint = format!("https://github.com/PeggyJV/sommelier/archive/refs/tags/{}.zip", SOMMELIER_RELEASE_VERSION);
    get_proto_files(sommelier_repo_endpoint, tmp_path);
    compile_protos(out_path, tmp_path);
}

// Download the sommelier repo and copy the files to the tmp directory
fn get_proto_files(sommelier_repo_endpoint: String, tmp_dir: &Path) {
    eprintln!("[info] Downloading sommelier repo...");
    let mut sommelier_zip = reqwest::blocking::get(&sommelier_repo_endpoint).unwrap();
    let sommelier_zip_name = format!("{}/sommelier.zip", tmp_dir.display());
    let mut sommelier_zip_file = std::fs::File::create(&sommelier_zip_name).unwrap();
    io::copy(&mut sommelier_zip, &mut sommelier_zip_file).unwrap();
    let sommelier_zip_file = std::fs::File::open(sommelier_zip_name).unwrap();
    let mut archive = zip::ZipArchive::new(sommelier_zip_file).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        let outpath = tmp_dir.join(outpath);
        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }
    eprintln!("[info] => Done!");
}

fn compile_protos(out_dir: &Path, tmp_dir: &Path) {
    eprintln!(
        "[info] Compiling .proto files to Rust into '{}'...",
        out_dir.display()
    );

    let root = env!("CARGO_MANIFEST_DIR");
    let root: PathBuf = root.parse().unwrap();
    // this gives us the repo root by going up two levels from the module root
    let root = root.parent().unwrap().to_path_buf();

    let proto_root = format!("/tmp/sommelier-auction-proto-build/sommelier-{}", SOMMELIER_RELEASE_VERSION[1..].to_string());

    let mut cellarfees_proto_dir = root.clone();
    cellarfees_proto_dir.push(format!("{}/proto/cellarfees/v1", proto_root));
    let mut auction_proto_dir = root.clone();
    auction_proto_dir.push(format!("{}/proto/auction/v1", proto_root));

    let mut proto_include_dir = root.clone();
    proto_include_dir.push(format!("{}/proto", proto_root));
    let mut third_party_proto_include_dir = root;
    third_party_proto_include_dir.push(format!("{}/third_party/proto", proto_root));

    // Paths
    let proto_paths = [
        cellarfees_proto_dir,
        auction_proto_dir,
    ];
    // we need to have an include which is just the folder of our protos to satisfy protoc
    // which insists that any passed file be included in a directory passed as an include
    let proto_include_paths = [proto_include_dir, third_party_proto_include_dir];

    // List available proto files
    let mut protos: Vec<PathBuf> = vec![];
    for proto_path in &proto_paths {
        protos.append(
            &mut WalkDir::new(proto_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().is_file()
                        && e.path().extension().is_some()
                        && e.path().extension().unwrap() == "proto"
                })
                .map(|e| e.into_path())
                .collect(),
        );
    }

    // Compile all proto files
    let mut config = prost_build::Config::default();
    config.out_dir(tmp_dir);
    config
        .type_attribute("ScheduledCorkProposal", "#[derive(serde::Deserialize, serde::Serialize)]")
        .type_attribute("AxelarScheduledCorkProposal", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile_protos(&protos, &proto_include_paths)
        .unwrap();

    // Compile all proto client for GRPC services
    println!("[info ] Compiling proto clients for GRPC services!");
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .format(true)
        .out_dir(tmp_dir)
        .compile_with_config(config, &protos, &proto_include_paths)
        .unwrap();

    copy_generated_files(tmp_dir, out_dir);

    println!("[info ] => Done!");
}

fn copy_generated_files(from_dir: &Path, to_dir: &Path) {
    eprintln!("Copying generated files into '{}'...", to_dir.display());

    // Remove old compiled files
    remove_dir_all(&to_dir).unwrap_or_default();
    create_dir_all(&to_dir).unwrap();

    let mut filenames = Vec::new();

    // Copy new compiled files (prost does not use folder structures)
    let errors = WalkDir::new(from_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name().to_str().unwrap().contains(".rs"))
        .map(|e| {
            let filename = e.file_name().to_os_string().to_str().unwrap().to_string();
            filenames.push(filename.clone());
            copy_and_patch(e.path(), format!("{}/{}", to_dir.display(), &filename))
        })
        .filter_map(|e| e.err())
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        for e in errors {
            eprintln!("[error] Error while copying compiled file: {}", e);
        }

        panic!("[error] Aborted.");
    }
}

fn copy_and_patch(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<()> {
    // Skip proto files belonging to `EXCLUDED_PROTO_PACKAGES`
    for package in EXCLUDED_PROTO_PACKAGES {
        if let Some(filename) = src.as_ref().file_name().and_then(OsStr::to_str) {
            if filename.starts_with(&format!("{}.", package)) {
                return Ok(());
            }
        }
    }

    let contents = fs::read_to_string(src)?;

    // `prost-build` output references types from `tendermint-proto` crate
    // relative paths, which we need to munge into `tendermint_proto` in
    // order to leverage types from the upstream crate.
    let contents = Regex::new(COSMOS_SDK_PROTO_REGEX)
        .unwrap()
        .replace_all(&contents, "cosmos_sdk_proto::cosmos");

    fs::write(dest, contents.as_bytes())
}
