extern crate sqpack_blue;
extern crate threadpool;
extern crate fallible_iterator;
extern crate sha1;
extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

use std::path::PathBuf;
use std::fs::{OpenOptions, File};

mod process_all;
mod general_processor;
mod async_data_processor;
mod exporting;

pub mod errors;
pub mod manifest;

pub use process_all::process_all;

use errors::AzureError;
use sqpack_blue::FFXIV;

pub use exporting::ExportMode;

pub struct BGMOptions {
    save_file: Option<File>,
    compare_file: Option<manifest::ManifestFile>,
    export_mode: Option<ExportMode>,
}

pub struct AzureOptions {
    ffxiv: FFXIV,
    thread_count: usize
}

impl BGMOptions {
    pub fn new(save_file: Option<PathBuf>,
               compare_file: Option<PathBuf>,
               export_mode: Option<ExportMode>) -> Result<BGMOptions, AzureError> {
        save_file.map_or(Ok(None), |f_str| {
            OpenOptions::new().write(true).create_new(true).open(f_str).map_err(|_| {
                AzureError::UnableToCreateSaveFile
            }).map(|f| Some(f))
        }).and_then(|save_file| {
            compare_file.map_or(Ok(None), |f_str| {
                OpenOptions::new().read(true).open(f_str).map_err(|_| {
                    AzureError::UnableToReadCompareFile
                }).and_then(|compare_file| {
                    ::serde_json::from_reader::<File, manifest::ManifestFile>(compare_file).map_err(|_| {
                        AzureError::UnableToReadCompareFile
                    })
                }).map(|mf| Some(mf))
            }).map(|compare_file| (save_file, compare_file))
        }).and_then(|(save_file, compare_file)| {
            Ok(BGMOptions {
                save_file,
                compare_file,
                export_mode
            })
        })
    }
}

impl AzureOptions {
    pub fn new(ffxiv_path: PathBuf, thread_count: usize) -> Result<AzureOptions, AzureError> {
        Ok(ffxiv_path.as_path())
            .and_then(|ff| FFXIV::new(ff).ok_or(AzureError::NoFFXIV))
            .and_then(|ffxiv| {
                Ok(AzureOptions{ ffxiv, thread_count })
            })
    }
}

pub fn export_one() {

}

pub fn bgm_csv() {

}

#[derive(Debug)]
pub enum AzureProcessPhase {
    Begin,
    ReadingBGMSheet,
    Hashing,
    Collecting,
    SavingManifest,
    Exporting,
}

//pub enum ProcessResult
//
pub enum AzureProcessStatus {
    Start,
    Continue,
    Completed,
}

pub struct AzureProcessBegin {
    total_operations_count: usize,
}

pub struct AzureProcessProgress {
    total_operations_count: usize,
    operations_progress: usize,
    current_operation: usize,
    is_skip: bool,
}

pub struct AzureProcessNonfatalError {
    current_operation: usize,
    reason: String,
}

pub struct AzureProcessComplete {
    operations_completed: usize,
    operations_errored: usize,
}

pub trait AzureCallbacks {
    fn pre_phase(&self, phase: AzureProcessPhase);
    fn post_phase(&self, phase: AzureProcessPhase);

    fn process_begin(&self, info: AzureProcessBegin);
    fn process_progress(&self, info: AzureProcessProgress);
    fn process_nonfatal_error(&self, info: AzureProcessNonfatalError);
    fn process_complete(&self, info: AzureProcessComplete);
}

#[cfg(test)]
mod tests {


    struct MyCB;
    use ::*;
    impl ::AzureCallbacks for MyCB {
        fn pre_phase(&self, phase: AzureProcessPhase) {
            println!("{:?}", phase);
        }
        fn post_phase(&self, phase: AzureProcessPhase) {
            println!("{:?}", phase);
        }

        fn process_begin(&self, info: AzureProcessBegin) {

        }
        fn process_progress(&self, info: AzureProcessProgress) {

        }
        fn process_nonfatal_error(&self, info: AzureProcessNonfatalError) {

        }
        fn process_complete(&self, info: AzureProcessComplete) {

        }
    }

    #[test]
    fn it_works() {

        use std::path::PathBuf;
        let ffpath = PathBuf::from("C:\\Program Files (x86)\\SquareEnix\\FINAL FANTASY XIV - A Realm Reborn\\game\\sqpack");
        let save_file = PathBuf::from("save.json");
        let export_path = PathBuf::from("output");
        let bgmopts = ::BGMOptions::new(None, None, Some(::ExportMode::OGG(export_path)));



        super::process_all(::AzureOptions::new(ffpath, 4usize).unwrap(), bgmopts.unwrap(), &MyCB{}).unwrap();


    }
}
