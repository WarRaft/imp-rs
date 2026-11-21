use super::*;

pub(crate) fn list() -> Imports {
    Imports {
        version: 1,
        files: vec![
            Import { flags: 13, path: "war3mapImported\\Icon.blp".into() },
            Import { flags: 10, path: "Sound.mp3".into() },
            Import { flags: 0, path: "a, \"b\".mdx".into() },
        ],
    }
}

#[test]
fn reads_and_writes_back() {
    for version in [0, 1] {
        let mut l = list();
        l.version = version;
        if version == 0 {
            l.files.iter_mut().for_each(|f| f.flags = 0);
        }
        let bytes = l.to_bytes().unwrap();
        assert_eq!(parse(&bytes).unwrap(), l);
    }
    let l = list();
    assert_eq!(l.files[0].path_in_map(), "war3mapImported\\Icon.blp");
    assert_eq!(l.files[1].path_in_map(), "war3mapImported\\Sound.mp3");
    let custom = Import { flags: CUSTOM_PATH, path: "Units\\X.mdx".into() };
    assert_eq!(custom.path_in_map(), "Units\\X.mdx");
    let plain = Import { flags: 0, path: "Units\\X.mdx".into() };
    assert_eq!(plain.path_in_map(), "war3mapImported\\X.mdx");
}

#[test]
fn bad_files() {
    let err = |b: &[u8]| parse(b).unwrap_err().to_string();
    assert!(err(&[2, 0, 0, 0, 0, 0, 0, 0]).contains("unknown imp version 2"));
    assert!(err(&[1, 0, 0, 0, 9, 0, 0, 0, 1]).contains("doesn't fit"));
    let mut bytes = list().to_bytes().unwrap();
    bytes.push(0);
    assert!(err(&bytes).contains("extra byte"));
}

/// Every map's import list reads and writes back, and through the text
/// form: `IMP_CORPUS` is a folder of `<map>/war3map.imp`.
#[test]
#[ignore]
fn corpus() {
    let dir = std::env::var("IMP_CORPUS").expect("IMP_CORPUS");
    let (mut n, mut files, mut bad) = (0, 0, Vec::new());
    let mut flags = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path().join("war3map.imp");
        let Ok(bytes) = std::fs::read(&path) else { continue };
        n += 1;
        match parse(&bytes) {
            Ok(l) if l.to_bytes().unwrap() != bytes => bad.push(format!("{}: differs", path.display())),
            Ok(l) => {
                files += l.files.len();
                l.files.iter().for_each(|f| *flags.entry(f.flags).or_insert(0) += 1);
                let back = text::from_text(&text::to_text(&l));
                if back.list.map(|l| l.to_bytes().unwrap()) != Some(bytes.clone()) {
                    bad.push(format!("{}: differs through text: {:?}", path.display(), back.errors));
                }
            }
            Err(e) => bad.push(format!("{}: {e}", path.display())),
        }
    }
    println!("{n} files, {files} imports, flags {flags:?}");
    assert!(bad.is_empty(), "{} of {n}:\n{}", bad.len(), bad.join("\n"));
}
