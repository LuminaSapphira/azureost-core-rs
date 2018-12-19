use ::sha1::{Sha1, Digest};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestFile {
    pub files: BTreeMap<usize, TrackManifest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackManifest {
    pub index: usize,
    pub name: String,
    pub sha1: Digest,

}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    #[test]
    fn basic_manifest() {

        let manifest = ManifestFile {
            files: vec![
                TrackManifest {
                    index: 3,
                    sha1: Sha1::from("asd").digest(),
                    name: "bgm_lol_no.scd".into()
                },
                TrackManifest {
                    index: 4,
                    sha1: Sha1::from("asasdasd").digest(),
                    name: "bgm_ayy_lmao.scd".into()
                },
                TrackManifest {
                    index: 5,
                    sha1: Sha1::from("13234234").digest(),
                    name: "bgm_neko_nyaaa.scd".into()
                }
            ].into_iter().map(|mf| (mf.index.clone(), mf)).collect()
        };
        let sha_str = ::serde_json::to_string(&manifest).unwrap();
        println!("{}", ::serde_json::to_string(&manifest).unwrap());
        let sha_bytes: [u8; 20] = [0xD7,0x82,0x4B,0xF2,0xA4,0x63,0xF7,0x34,0x0D,0xB7,0xF1,0x30,0x50,0x73,0x74,0xB2,0x3D,0xF9,0xE7,0x2E];

        assert_eq!(Sha1::from(sha_str).digest().bytes(), sha_bytes);
    }

}