use super::*;
use crate::tests::list;

#[test]
fn text_and_back() {
    let l = list();
    let text = to_text(&l);
    assert_eq!(text, "[imp]\nversion=1\nfile=13,\"war3mapImported\\Icon.blp\"\nfile=10,\"Sound.mp3\"\nfile=0,\"a, \"\\u{22}\"b\"\\u{22}\".mdx\"\n");
    assert_eq!(from_text(&text).list.unwrap(), l);
    let v0 = Imports { version: 0, files: vec![Import { flags: 0, path: "x.blp".into() }] };
    assert_eq!(to_text(&v0), "[imp]\nversion=0\nfile=\"x.blp\"\n");
    assert_eq!(from_text(&to_text(&v0)).list.unwrap(), v0);
}

#[test]
fn errors() {
    let e: Vec<(usize, String)> = from_text("[imp]\nversion=0\nfile=1,\"x\"\nfoo=1\n[more]\nx=1\n")
        .errors
        .into_iter()
        .map(|e| (e.line, e.message))
        .collect();
    assert_eq!(e.len(), 3, "{e:?}");
    assert!(e.iter().any(|(l, m)| *l == 3 && m.contains("no flags")), "{e:?}");
    assert!(from_text("[imp]\nfile=300,\"x\"\n").errors[0].message.contains("0 to 255"));
    assert!(from_text("[x]\n").errors[0].message.contains("starts with [imp]"));
}
