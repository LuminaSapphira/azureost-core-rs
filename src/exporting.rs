use std::path::PathBuf;
use std::fs::{OpenOptions, File};
use ::errors::AzureError;

#[cfg(feature = "lamemp3")]
extern crate lame;

extern crate vorbis;
extern crate lewton;

use self::vorbis::{Decoder, Encoder, VorbisQuality, VorbisError};
use self::lewton::inside_ogg::OggStreamReader;

pub enum ExportMode {
    #[cfg(feature="lamemp3")]
    MP3(PathBuf),
    OGG(PathBuf),
}

impl ExportMode {
    pub fn get_path(&self) -> &PathBuf {
        match self {
            #[cfg(feature="lamemp3")]
            ExportMode::MP3(pb) => pb,
            ExportMode::OGG(pb) => pb,
        }
    }

    #[cfg(feature="lamemp3")]
    fn export_mp3(&self, file_name: &str, data: Vec<u8>) -> Result<(), AzureError> {
        use self::lame::{Lame, EncodeError};
        unimplemented!();
    }

    pub fn export_file(&self, file_name: &str, data: Vec<u8>) -> Result<(), AzureError> {
        decode_ogg(data)
            .and_then(|decoded| {
                /*
                TODO process file
                1. Interleave
                2. Loop
                3. Fade
                */
            })
        unimplemented!();
    }
}

struct LoopInfo {
    pub start: usize,
    pub end: usize,
}

struct DecodedOgg {
    pub samples: Vec<Vec<i16>>,
    pub rate: u64,
    pub channels: usize,
    pub loop_info: Option<LoopInfo>,
}

fn extract_loop_info(comments: Vec<(String, String)>) -> Option<LoopInfo> {
    use std::str::FromStr;
    let loop_start = comments
        .iter()
        .filter(|comment| {
            comment.0 == "LoopStart"
        })
        .map(|f| {
            usize::from_str(f.1.as_str())
        })
        .next();
    let loop_end = comments.iter()
        .filter(|comment| {
            comment.0 == "LoopEnd"
        })
        .map(|f| {
            usize::from_str(f.1.as_str())
        })
        .next();

    loop_start
        .and_then(|res| res.ok())
        .and_then(|ls_sam| {
            loop_end.and_then(|res| res.ok())
                .map(|le_sam| {
                    LoopInfo {start: ls_sam, end: le_sam}
                })
        })
}

fn decode_ogg(scd_ogg: Vec<u8>) -> Result<DecodedOgg, AzureError> {
    use std::io::Cursor;
    OggStreamReader::new(Cursor::new(scd_ogg))
        .map_err(|o| AzureError::ErrorDecoding)
        .and_then(|mut osr| {
            let mut samples = vec![Vec::<i16>::new(); osr.ident_hdr.audio_channels as usize];
            let mut has_samples: bool = true;
             while has_samples {
                 let packet = osr.read_dec_packet().map_err(|o|AzureError::ErrorDecoding)?;
                 if packet.is_some() {
                     let vecs = packet.unwrap();
                     for (i, pkt_samples) in vecs.into_iter().enumerate() {
                         samples[i].extend(pkt_samples);
                     }
                 } else { has_samples = false }
             }
            Ok((osr.comment_hdr.comment_list, samples, osr.ident_hdr.audio_sample_rate))
        })
        .and_then(|(comment_list, samples, rate)| {
            Ok(DecodedOgg {
                channels: samples.len(),
                samples,
                rate: rate as u64,
                loop_info: extract_loop_info(comment_list)
            })
        })
}

/// sample_index: interleaved index
/// fade_start interleaved index
/// fade_length raw index
fn fade_val(sample_index: usize, fade_start: usize, fade_length: usize, channels: usize) -> f32 {
    if sample_index >= fade_start {
        let ch_sample_index = sample_index / channels;
        let ch_fade_start = fade_start / channels;
        (fade_length + ch_fade_start - ch_sample_index) as f32 / fade_length as f32
    } else { 1f32 }
}

fn fade(samp: &mut Vec<i16>, rate: u64, channels: usize) {
    let len = samp.len();
    // fade for the last 5% of a song or 30 seconds, whichever is less
    let fade_length = (rate as usize * 30).min((len as f32 / channels as f32 * 0.05) as usize);
    let fade_start = len - fade_length * channels;
    samp.iter_mut().enumerate().for_each(|sample| {
        (*sample.1) = ((*sample.1 as f32) *
            fade_val(sample.0, fade_start, fade_length, channels)

        ) as i16;
    });
}


fn interleave<T>(input: Vec<Vec<T>>) -> Vec<T> {
    let capacity = input.len()*input[0].len();
    let mut t = input.into_iter().map(|a| {
        a.into_iter()
    }).collect::<Vec<_>>();
    let mut end = Vec::with_capacity(capacity);
    'outer: loop {
        for mut f in &mut t {
            match f.next() {
                Some(v) => end.push(v),
                None => {
                    break 'outer;
                }
            }
        }
    }
    end
}