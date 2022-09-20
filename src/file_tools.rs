use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

fn containes_all_file(root_path: &PathBuf, files: &mut HashSet<&str>) -> bool {
    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    files.remove(file_name.as_str());
                }
            }
        }
    }

    files.is_empty()
}

pub fn check_is_bds_root(root: &str) -> bool {
    let mut check_files = HashSet::from([
        "bedrock_server.exe",
        "bedrock_server.pdb",
        "behavior_packs",
        "definitions",
        "resource_packs",
        "server.properties",
        "allowlist.json",
    ]);

    containes_all_file(&PathBuf::from(root), &mut check_files)
}

pub fn check_installed_liteloader(root: &str) -> bool {
    let mut check_files = HashSet::from([
        "plugins",
        "LLPeEditor.exe",
        "LiteLoader.dll",
        "bedrock_server_mod.exe",
    ]);

    containes_all_file(&PathBuf::from(root), &mut check_files)
}

pub fn check_installed_trapdoor(path: &str) -> bool {
    let bds_root = PathBuf::from(path);
    let plugins_root = bds_root.join("plugins");
    let config_path = plugins_root.clone().join("trapdoor");

    let mut exists_tr_dll = false;
    if let Ok(entries) = fs::read_dir(plugins_root) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with("trapdoor-") && file_name.ends_with(".dll") {
                        exists_tr_dll = true;
                    }
                }
            }
        }
    }

    let mut check_files = HashSet::from(["config.json"]);

    exists_tr_dll && containes_all_file(&config_path, &mut check_files)
}

pub fn extract_file_to(file_name: &str, out_dir: &str) -> bool {
    let fname = std::path::Path::new(file_name);
    let file = fs::File::open(&fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    let out_prefix = PathBuf::from(out_dir);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        //修正
        let mut r = out_prefix.clone();
        for d in outpath.iter() {
            if let Some(c) = d.to_str() {
                if c != "out" {
                    r.push(c);
                }
            }
        }

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&r).unwrap();
        } else {
            if let Some(p) = r.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&r).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    true
}
