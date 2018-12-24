use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::fs::DirBuilder;

use ::{ExportMode, BGMOptions, AzureOptions, AzureCallbacks};
use ::errors::AzureError;
use ::async_data_processor::{ThreadStatus, async_processor};
use ::sqpack_blue::{FFXIV, ExFileIdentifier, FFXIVError};
use ::sqpack_blue::sheet::ex::SheetLanguage;
use ::sqpack_blue::sheet::SheetRow;
use ::sha1::{Sha1, Digest};
use ::manifest::*;

fn is_known_skip(skip: &str) -> bool {
    match skip {
        "music/ffxiv/BGM_Season_China01.scd" => true,
        "music/ffxiv/BGM_Event_OP01.scd" => true,
        "music/ffxiv/BGM_Leves_Lim_01.scd" => true,
        _ => false,
    }
}

struct EXFCollection {
    pub collected: Vec<TrackManifest>,
    pub uncollected: Vec<TrackManifest>,
}

//fn get_sheet_index(ffxiv: FFXIV) ->

pub fn process<AC>(azure_opts: AzureOptions,
               bgm_opts: BGMOptions,
               process_indicies: Vec<usize>,
               callbacks: &AC) -> Result<(), AzureError>
    where AC: AzureCallbacks + Sized{

//    convert(process_indicies.into_iter()).

    Ok(azure_opts.ffxiv.clone())
        // get the Sheet index
        .and_then(|ffxiv| {
            ffxiv.get_sheet_index().map_err(|e| e.into())
                .map(|a| (ffxiv, a))
        })
        // Get the BGM Sheet using the sheet index
        .and_then(|(ffxiv, sheet_index)| {
            ffxiv.get_sheet(&String::from("bgm"),
                            SheetLanguage::None, &sheet_index)
                .map_err(|e| e.into())
                .map(|a| (ffxiv, a))
        })
        // Read the BGM Sheet to transform the requested indices into ExFileIdentifiers
        .and_then(|(ffxiv, sheet)| {
            let invalid_indices = process_indicies.iter().cloned().filter(|index| {
                *index >= sheet.rows.len()
            }).collect::<Vec<_>>();
            if invalid_indices.len() > 0 {
                Err(AzureError::InvalidBGMIndex(invalid_indices))
            } else {
                // TODO refactor into functions
                let (exfiles, errors): (Vec<_>, Vec<_>) =
                    process_indicies.iter().cloned()
                        .map(|index| { (index, &sheet.rows[index]) })
                        .map(|(index, row)| {
                            row.read_cell_data::<String>(0).map_err(|e| {
                                FFXIVError::SheetError(e)
                            }).map(|f_str| {
                                (index, f_str)
                            })
                        })
                        .filter(|s| {
                            match s {
                                Ok(s) => !is_known_skip(s.1.as_str()),
                                Err(e) => true
                            }
                        })
                        .map(|s| {
                            s.and_then(|(index, f_str)| {
                                ExFileIdentifier::new(&f_str).map(|exf| (index, exf))
                            }).and_then(|e| Ok(e))
                        })
                        .partition(|e| e.is_ok());
                //.partition::<Vec<Result<(usize, ExFileIdentifier), FFXIVError>>, Vec<_>>(|e| {Result::is_ok});
                let exfiles = exfiles.into_iter().map(|k| k.unwrap()).collect::<Vec<_>>();
                let errors = errors.into_iter().map(|k| k.err().unwrap()).collect::<Vec<_>>();
                if !errors.is_empty() {
                    Err(AzureError::FFXIVErrorVec(errors))
                } else {
                    Ok((ffxiv, exfiles))
                }
            }
        })
        // hash the exfile SCDs
        .and_then(|(ffxiv, exfiles)| {
            let hashes =
                if bgm_opts.compare_file.is_some() || bgm_opts.save_file.is_some() {
                    let recv = async_processor(
                        azure_opts.thread_count,
                        ffxiv.clone(),
                        &exfiles,
                        |index, data| {
                            ThreadStatus::Continue((index, Sha1::from(data).digest()))
                        });
                    let mut hashes = HashMap::new();
                    let mut threads_completed = 0usize;
                    let mut files_completed = 0usize;
                    'thread_recv: for received in recv {
                        match received {
                            ThreadStatus::Continue((index, digest)) => {
                                hashes.insert(index, digest);
                                files_completed += 1;
                                // todo call continue func
                            }
                            ThreadStatus::Complete => {
                                threads_completed += 1;
                                if threads_completed == azure_opts.thread_count {
                                    break 'thread_recv;
                                }
                            }
                            ThreadStatus::Skip => {
                                files_completed += 1;
                                // todo call skip func
                            }
                            ThreadStatus::Error(_) => {
                                files_completed += 1;
                                // todo call error func
                            }
                        }
                    }
                    Some(hashes)
                } else {
                    None
                };
            Ok((ffxiv, exfiles, hashes))
        })
        // partition exfiles into collected and uncollected
        .and_then(|(ffxiv, exfiles, hashes)| {
            let collects: (Vec<TrackManifest>, Vec<TrackManifest>) =
                exfiles.into_iter()
                    .map(|(index, exf)| {
                        TrackManifest {
                            index,
                            name: exf.get_exfile_string().clone(),
                            sha1: hashes.as_ref().map(|h| h[&index]).unwrap_or_else(|| Sha1::new().digest()),
                        }
                    })
                    .partition(|track_mf| {
                        bgm_opts.compare_file.as_ref()
                            .map(|compare| {
                                compare.files.get(&track_mf.index)
                                    .map(|compare_track_mf| {
                                        compare_track_mf.sha1.ne(&track_mf.sha1)
                                    })
                                    .unwrap_or(true)
                            }).unwrap_or(true)
                    });
            Ok((ffxiv, collects))
        })
        // save manifest file
        .and_then(|(ffxiv,
                       (collects, uncollects))| {
            bgm_opts.save_file.as_ref()
                .and_then(|save_file| {
                    Some(::serde_json::to_writer_pretty(save_file,
                                                        &ManifestFile {
                                                            files: collects.iter().cloned().chain(uncollects.iter().cloned())
                                                                .map(|t_mf| (t_mf.index, t_mf))
                                                                .collect::<BTreeMap<usize, TrackManifest>>()
                                                        }))
                })
                .map(|save_res| {
                    save_res.map_err(|_| AzureError::ErrorWritingSaveFile)
                })
                .unwrap_or(Ok(()))
                .map(|_| (ffxiv, collects, uncollects))
        })
        .map(|_| ())
//        .and_then(|(ffxiv, collects, uncollects)| {
//            bgm_opts.export_mode.as_ref()
//                .and_then(|export_mode| {
//                    Some({
//                        DirBuilder::new()
//                            .recursive(true)
//                            .create(export_mode.get_path())
//                            .and_then(|_| {
//                                collects.iter().map(|t_mf| {
//                                    ffxiv.get_exfile(&t_mf.name)
//                                        .map(|exf| (t_mf.index, exf))
//                                }).collect::<Result<Vec<_>, _>>()
//                                    .and_then(|work| {
//                                        let index_name_map = work
//                                            .iter().map(|(index, exf)| (index, exf.get_exfile_string().clone()))
//                                            .collect::<HashMap<usize, String>>();
//                                        async_processor(azure_opts.thread_count, ffxiv.clone(), &work, |index, data| {
//                                            index_name_map.get(&index).map_or(ThreadStatus::Error(format!("Invalid index passed to exporter! Index: {}", index)), |f_name| {
//                                                let a: Vec<&str> = f_name.split("/").skip(1).collect();
//                                                export_mode.export_file(a.join("/"), collect);
//                                            })
//                                        })
//                                    })
//
//                            })
//                    })
//                })
//                .map(|export_result| {
//
//                })
//        })
//    let sheet = ffxiv.get_sheet(
//        &String::from("bgm"),
//        SheetLanguage::English,
//        &ffxiv.get_sheet_index().unwrap()).unwrap();
//
//    let a = async_processor(1, ffxiv, Vec::new(), |data| {
//        ThreadStatus::Continue(3usize)
//    });
}