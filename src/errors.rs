use std::error::Error;
use ::sqpack_blue::FFXIVError;

#[derive(Debug)]
pub enum AzureError {
    NoFFXIV,
    FFXIVError(FFXIVError),
    FFXIVErrorVec(Vec<FFXIVError>),
    InvalidBGMIndex(Vec<usize>),
    UnableToCreateSaveFile,
    UnableToReadCompareFile,
    ErrorWritingSaveFile,
    ErrorExporting,
    ErrorDecoding
}

impl Error for AzureError {}

impl std::fmt::Display for AzureError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::AzureError::*;
        match self {
            NoFFXIV => write!(f, "No FFXIV sqpack at this location!"),
            FFXIVError(e) => write!(f, "An error occurred while interfacing with FFXIV! {:?}", e),
            FFXIVErrorVec(e) => write!(f, "Several errors occurred while interfacing with FFXIV! {:?}", e),
            InvalidBGMIndex(index) => write!(f, "The requested index was invalid {:?}", index),
            UnableToCreateSaveFile => write!(f, "The save file was unable to be created anew."),
            UnableToReadCompareFile => write!(f, "The compare file was unable to be read or parsed."),
            ErrorWritingSaveFile => write!(f, "There was an error writing to the save file."),
            ErrorExporting => write!(f, "An error occurred during the export process"),
            ErrorDecoding => write!(f, "An error occurred while attempting to decode the SCD/OggVorbis Samples"),
        }
    }
}

impl From<FFXIVError> for AzureError {
    fn from(e: FFXIVError) -> AzureError {
        AzureError::FFXIVError(e)
    }
}