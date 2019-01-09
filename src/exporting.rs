use std::path::{Path, PathBuf};
use std::fs::{DirBuilder, OpenOptions};
use std::io::Write;
use ::errors::AzureError;

#[cfg(feature = "lamemp3")]
extern crate lame;

extern crate vorbis;
extern crate lewton;

use self::vorbis::{Encoder, VorbisQuality};
use self::lewton::inside_ogg::OggStreamReader;

#[derive(Clone)]
pub enum ExportMode {
    #[cfg(feature="lamemp3")]
    MP3(PathBuf),
    OGG(PathBuf),
}

#[inline]
fn get_looped_len(interleaved_len: usize, loop_info: &LoopInfo, num_channels: usize) -> usize {
    interleaved_len + (loop_info.end - loop_info.start) * num_channels
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
    fn export_mp3(&self, file_name: &str, data: Vec<i16>, sample_rate: u64) -> Result<(), AzureError> {
        use self::lame::Lame;

        let mut lame = Lame::new().unwrap();
        // TODO allow changing quality
        lame.set_quality(3).unwrap();
        lame.set_channels(2).unwrap();
        lame.set_sample_rate(sample_rate as u32).unwrap();
        lame.set_kilobitrate(128).unwrap();
        lame.init_params().unwrap();
        let mut out = vec![0u8; (5 * data.len()) / 4 + 7200];
        let (left, right): (Vec<(usize, i16)>, Vec<(usize, i16)>) = data
            .into_iter()
            .enumerate()
            .partition(|s| s.0 % 2 == 0);
        let left = left.into_iter().map(|a| a.1).collect::<Vec<i16>>();
        let right = right.into_iter().map(|a| a.1).collect::<Vec<i16>>();
        let output = lame.encode(&left, &right, &mut out).unwrap();
        let out = out.into_iter().take(output).collect::<Vec<u8>>();

        let path = Path::new(self.get_path()).join(Path::new(file_name).with_extension("mp3"));
        path.parent()
            .map(|parent| {
                DirBuilder::new().recursive(true).create(parent)
                    .map_err(|_| AzureError::ErrorExporting("Creating directory for output"))
            })
            .unwrap_or(Ok(()))
            .and_then(|_| {
                OpenOptions::new().create(true).write(true).truncate(true).open(path)
                    .and_then(|mut file| {
                        file.write_all(&out)
                    })
                    .map_err(|_| AzureError::ErrorExporting("Writing File"))
            })


    }

    fn export_ogg(&self, file_name: &str, data: Vec<i16>, sample_rate: u64) -> Result<(), AzureError> {
        // TODO allow changing quality
        Encoder::new(2, sample_rate, VorbisQuality::Midium)
            .map_err(|_| AzureError::ErrorExporting("Creating Vorbis encoder"))
            .and_then(|mut encoder| {
                encoder.encode(&data)
                    .map_err(|_| AzureError::ErrorExporting("Encoding vorbis"))
                    .and_then(|out| {
                        let path = Path::new(self.get_path()).join(Path::new(file_name).with_extension("ogg"));
                        path.parent()
                            .map(|parent| {
                                DirBuilder::new().recursive(true).create(parent)
                                    .map_err(|_| AzureError::ErrorExporting("Creating directory for output"))
                            })
                            .unwrap_or(Ok(()))
                            .and_then(|_| {
                                OpenOptions::new().create(true).write(true).truncate(true).open(path)
                                    .and_then(|mut file| {
                                        file.write_all(&out)
                                    })
                                    .map_err(|_| AzureError::ErrorExporting("Writing File"))
                            })
                    })
            })
    }

    pub fn export_file(&self, base_path: &str, scd_entry_index: usize, scd_entry_count: usize, data: Vec<u8>) -> Result<(), AzureError> {
        decode_ogg(data)
            .and_then(|decoded| {

                // needs to have at least 2 channels
                if decoded.channels < 2 {
                    return Err(AzureError::ErrorExporting("Needs at least 2 channels"))
                }

                // needs to be on L/R channels
                if decoded.channels % 2 != 0 {
                    return Err(AzureError::ErrorExporting("needs to be Left/Right channels"))
                }

                let layer_count = decoded.channels / 2;

                let mut layer_index = 0usize;

                let mut in_samples = decoded.samples;
                for _ in (0..decoded.channels).step_by(2) {
                    let r_c = in_samples.pop().unwrap();
                    let l_c = in_samples.pop().unwrap();
                    let lr_channels = vec![l_c, r_c];
                    let mut interleaved = interleave(lr_channels);

                    let mut samples = if decoded.loop_info.is_some() {
                        let info = decoded.loop_info.unwrap();
                        let mut final_samples = Vec::with_capacity(get_looped_len(interleaved.len(), &info, 2));
                        final_samples.extend(interleaved.drain(0..info.start * 2));
                        final_samples.extend(interleaved.iter().cloned().take((info.end - info.start) * 2));
                        let range = 0..((info.end - info.start) * 2);
                        final_samples.extend(interleaved.drain(range));
                        final_samples.extend(interleaved);
                        final_samples
                    } else {
                        interleaved
                    };
                    fade(&mut samples, decoded.rate, 2);

                    let layer_name = decoded.channels / 2 - layer_index;

                    let mut base_path = String::from(base_path);
                    let bp_len = base_path.len();
                    base_path.truncate(bp_len - 4);
                    let format_str = if layer_count == 1 {
                        if scd_entry_count == 1 {
                            format!("{}", base_path)
                        } else {
                            format!("{}_entry{}", base_path, scd_entry_index)
                        }
                    } else {
                        if scd_entry_count == 1 {
                            format!("{}_layer{}", base_path, layer_name)
                        } else {
                            format!("{}_entry{}_layer{}", base_path, scd_entry_index, layer_name)
                        }
                    };



                    match self {
                        #[cfg(feature="lamemp3")]
                        ExportMode::MP3(_) => self.export_mp3(format_str.as_str(), samples, decoded.rate)?,
                        ExportMode::OGG(_) => self.export_ogg(format_str.as_str(), samples, decoded.rate)?,
                    };
                    layer_index += 1;
                };
                Ok(())
            })
    }
}

#[derive(Copy, Clone)]
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
        .map_err(|_| AzureError::ErrorDecoding)
        .and_then(|mut osr| {
            let mut samples = vec![Vec::<i16>::new(); osr.ident_hdr.audio_channels as usize];
            let mut has_samples: bool = true;
             while has_samples {
                 let packet = osr.read_dec_packet().map_err(|_|AzureError::ErrorDecoding)?;
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