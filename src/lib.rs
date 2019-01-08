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
pub mod selector;
pub mod callbacks;

pub use process_all::{process_one, process_all};
pub use callbacks::AzureCallbacks;
pub use exporting::ExportMode;

use errors::AzureError;
use sqpack_blue::FFXIV;

/// Holds options for controlling the processing of the BGM. This includes the manifest save file,
/// the manifest file to compare against, and the export mode and path to use. Should be
/// instantiated using its ::new() function, which validates its arguments and returns a result.
pub struct BGMOptions {
    save_file: Option<File>,
    compare_file: Option<manifest::ManifestFile>,
    export_mode: Option<ExportMode>,
}

/// Holds data pertaining to the operation of the process, including the sqpack_blue FFXIV structure
/// and the number of threads to use during expensive procedures. Should be instantiated using its
/// ::new() function, which validates its arguments and returns a result.

#[derive(Clone)]
pub struct AzureOptions {
    ffxiv: FFXIV,
    thread_count: usize
}

impl BGMOptions {
    /// Creates an instance of the struct BGMOptions or else returns an error.
    ///
    /// # Parameters:
    /// * `save_file` - An Option referencing the location of the to be generated manifest file. This
    /// should not already exist - for safety purposes this program will not truncate manifest files.
    /// If you would like to skip the generation of a manifest, use the `None` variant here.
    /// * `compare_file` - An Option referencing the location of an existing manifest file to compare
    /// against. If you would like to skip comparisons and operate on all possible values, without
    /// regard for changes, use the `None` variant here.
    /// * `export_mode` - An Option referencing the **directory** to output decoded/encoded files to.
    /// No checking is done here. If the directory does not exist it will be made during the export
    /// process. If the directory does exist, the outputted files will be placed inside. If the path
    /// conflicts with a file, errors will be thrown during the export process (but will not panic!.)
    /// If you would like to skip exporting, use the `None` variant here.
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
    /// Creates an instance of AzureOptions by validating the parameters and returning a result.
    /// # Arguments
    /// * `ffxiv_path` - a `PathBuf` referencing the sqpack directory inside an FFXIV installation.
    /// **Note: internally, no checking is done to make sure the FFXIV directory is properly formatted
    /// other than that it exists. If files are missing, there may be errors.**
    /// * `thread_count` - the number of threads to use for expensive operations, such as hashing,
    /// exporting, etc.
    pub fn new(ffxiv_path: PathBuf, thread_count: usize) -> Result<AzureOptions, AzureError> {
        Ok(ffxiv_path.as_path())
            .and_then(|ff| FFXIV::new(ff).ok_or(AzureError::NoFFXIV))
            .and_then(|ffxiv| {
                Ok(AzureOptions{ ffxiv, thread_count })
            })
    }
}

/// Writes FFXIV's BGM datasheet to a specified file
/// # Arguments
/// * `azure_opts` - `AzureOptions` structure created earlier
/// * `output` - A `PathBuf` pointing to the file to output the CSV to. May or may not exist. If the
/// file exists, it will be truncated.
/// # Returns
/// * **Ok Variant** - An empty tuple indicating success.
/// * `Err(AzureError::UnableToCreateSaveFile)` - Indicates that the requested output file was
/// unable to be opened for writing or created.
/// * `Err(AzureError::FFXIVError)` - Indicates there was an error when parsing the BGM sheet internally.
pub fn bgm_csv(azure_opts: AzureOptions, output: PathBuf) -> Result<(), AzureError> {
    use sqpack_blue::sheet::write_csv;
    OpenOptions::new().create(true).truncate(true).write(true).open(output)
        .map_err(|_| AzureError::UnableToCreateSaveFile)
        .and_then(|save_file| {
            azure_opts.ffxiv.get_sheet_index().map_err(|o| AzureError::FFXIVError(o))
                .map(|sheet_index| (save_file, sheet_index))
        })
        .and_then(|(save_file, sheet_index)| {
            use ::sqpack_blue::sheet::ex::SheetLanguage;
            azure_opts.ffxiv.get_sheet(&String::from("bgm"), SheetLanguage::None, &sheet_index)
                .map_err(|o| AzureError::FFXIVError(o))
                .map(|sheet| (save_file, sheet))
        })
        .and_then(|(mut save_file, sheet)| {
            write_csv(&sheet, &mut save_file).map_err(|o| AzureError::FFXIVError(o))
        })

}

#[cfg(test)]
mod tests {

    use super::callbacks::*;

    impl AzureCallbacks for MyCB {
        fn pre_phase(&self, phase: AzureProcessPhase) {
            println!("PRE: {:?}", phase);
        }
        fn post_phase(&self, phase: AzureProcessPhase) {
            println!("POST: {:?}", phase);
        }

        fn process_begin(&self, info: AzureProcessBegin) {
            println!("Process Begin: {:?}", info);
        }
        fn process_progress(&self, info: AzureProcessProgress) {
            println!("Process Progress: {:?}", info);
        }
        fn process_nonfatal_error(&self, info: AzureProcessNonfatalError) {
            println!("Process Error: {:?}", info);
        }
        fn process_complete(&self, info: AzureProcessComplete) {
            println!("Process Complete: {:?}", info);
        }
    }
    struct MyCB;

    #[test]
    fn it_works() {
        use super::*;
        use std::path::Path;
        use std::fs;
        fs::remove_file("output.json").ok();
        let azopt = AzureOptions::new(Path::new(&std::env::var("FFXIV_SQPACK_PATH").unwrap()).to_path_buf(),
                                      4usize
        ).unwrap();
        let bgmopt = BGMOptions::new(Some(Path::new("output.json").to_path_buf()), None, None).unwrap();
        process_all(azopt, bgmopt, &MyCB{}).unwrap();
//        process_one(&639usize, azopt, bgmopt, &MyCB{}).unwrap();
    }

}
