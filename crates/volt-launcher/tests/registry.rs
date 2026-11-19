#![cfg(windows)]
#[path = "../src/registry.rs"]
mod registry;
use horizon_mods::index::{self, Index, Kind};
use std::{
    fs,
    io::{Cursor, Write},
};
#[test]
fn fresh_verified_install_is_profile_scoped_and_fails_closed() {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("pack.mcmeta", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(br#"{"pack":{"pack_format":1,"description":"test"}}"#)
        .unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    let sha = index::sha256(&bytes);
    let json = serde_json::json!({"schema":1,"projects":[{"id":"example","kind":"resourcepack","name":"Test","summary":"Fixture","license":"MIT","source":"https://github.com/example/fixture","versions":[{"version":"1","game":horizon_core::VANILLA_VERSION,"api":"1","file":{"name":"fixture.zip","url":"https://github.com/example/fixture/releases/download/1/pack.zip","size":bytes.len(),"sha256":sha}}]}]});
    let idx = Index::parse(&json.to_string()).unwrap();
    let e = registry::rows(idx.clone(), &Kind::ResourcePack)
        .pop()
        .unwrap();
    let mut nonce = [0; 16];
    getrandom::fill(&mut nonce).unwrap();
    let root = std::env::temp_dir().join(format!("Volt registry {}", u128::from_le_bytes(nonce)));
    let profile = root.join("profiles/profile-1");
    let put = |ix, data: Vec<u8>| {
        registry::install_from(ix, &profile, &Kind::ResourcePack, e.clone(), |_| {
            Ok(Cursor::new(data))
        })
    };
    assert!(put(&idx, vec![0; bytes.len()]).is_err());
    assert_eq!(
        fs::read_dir(profile.join("resourcepacks")).unwrap().count(),
        0
    );
    let mut revoked = json.clone();
    revoked["revoked"] = serde_json::json!([{"id":"example","reason":"test","sha256":sha}]);
    let revoked = Index::parse(&revoked.to_string()).unwrap();
    assert!(put(&revoked, bytes.clone()).is_err());
    put(&idx, bytes.clone()).unwrap();
    assert!(put(&idx, bytes.clone()).is_err());
    assert_eq!(
        fs::read(profile.join("resourcepacks/volt-example.zip")).unwrap(),
        bytes
    );
    assert!(!root.join("resourcepacks").exists());
    fs::remove_dir_all(root).unwrap();
}
