use std::collections::{BTreeMap, HashMap};
use std::fs::DirBuilder;

use ::{BGMOptions, AzureOptions};
use ::errors::AzureError;
use ::async_data_processor::{ThreadStatus, async_processor};
use ::sqpack_blue::{ExFileIdentifier, FFXIVError};
use ::sqpack_blue::sheet::ex::SheetLanguage;
use ::sha1::Sha1;
use ::manifest::*;
use ::callbacks::*;

fn is_known_skip(skip: &str) -> bool {
    match skip {
        "music/ffxiv/BGM_Season_China01.scd" => true,
        "music/ffxiv/BGM_Event_OP01.scd" => true,
        "music/ffxiv/BGM_Leves_Lim_01.scd" => true,
        "" => true,
        "music/ffxiv/BGM_Null.scd" => true,
        _ => false,
    }
}

//fn get_sheet_index(ffxiv: FFXIV) ->
pub fn process(azure_opts: AzureOptions,
               bgm_opts: BGMOptions,
               process_indicies: Vec<usize>,
               callbacks: &AzureCallbacks) -> Result<(), AzureError> {

    callbacks.pre_phase(AzureProcessPhase::Begin);
    callbacks.post_phase(AzureProcessPhase::Begin);

    Ok(azure_opts.ffxiv.clone())
        // get the Sheet index
        .and_then(|ffxiv| {
            callbacks.pre_phase(AzureProcessPhase::ReadingBGMSheet);
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
                        .map(|index| { (index, &sheet.rows[&index]) })
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
                                Err(_) => true
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

                callbacks.post_phase(AzureProcessPhase::ReadingBGMSheet);

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
                    callbacks.pre_phase(AzureProcessPhase::Hashing);
                    callbacks.process_begin(AzureProcessBegin {
                        total_operations_count: exfiles.len()
                    });
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
                    let mut files_errored = 0usize;
                    'thread_recv: for received in recv {
                        match received {
                            ThreadStatus::Continue((index, digest)) => {
                                hashes.insert(index, digest);
                                files_completed += 1;
                                callbacks.process_progress(AzureProcessProgress {
                                    total_operations_count: exfiles.len(),
                                    is_skip: false,
                                    current_operation: index,
                                    operations_progress: files_completed
                                })
                            },
                            ThreadStatus::Complete => {
                                threads_completed += 1;
                                if threads_completed == azure_opts.thread_count {
                                    break 'thread_recv;
                                }
                            },
//                            ThreadStatus::Skip => {
//                                files_completed += 1;
//                            },
                            ThreadStatus::Error(reason, current_operation) => {
                                files_completed += 1;
                                files_errored += 1;
                                callbacks.process_nonfatal_error(AzureProcessNonfatalError {
                                    reason, current_operation
                                });
                            },
                        }
                    }
                    callbacks.process_complete(AzureProcessComplete {
                        operations_completed: files_completed,
                        operations_errored: files_errored
                    });
                    callbacks.post_phase(AzureProcessPhase::Hashing);
                    Some(hashes)
                } else {
                    None
                };

            Ok((ffxiv, exfiles, hashes))
        })
        // partition exfiles into collected and uncollected
        .and_then(|(ffxiv, exfiles, hashes)| {
            callbacks.pre_phase(AzureProcessPhase::Collecting);
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
            callbacks.post_phase(AzureProcessPhase::Collecting);
            Ok((ffxiv, collects))
        })
        // save manifest file
        .and_then(|(ffxiv,
                       (collects, uncollects))| {

            let next = bgm_opts.save_file.as_ref()
                .and_then(|save_file| {
                    callbacks.pre_phase(AzureProcessPhase::SavingManifest);
                    let write_output = Some(::serde_json::to_writer_pretty(save_file,
                                                        &ManifestFile {
                                                            files: collects.iter().cloned().chain(uncollects.iter().cloned())
                                                                .map(|t_mf| (t_mf.index, t_mf))
                                                                .collect::<BTreeMap<usize, TrackManifest>>()
                                                        }));
                    callbacks.post_phase(AzureProcessPhase::SavingManifest);
                    write_output
                })
                .map(|save_res| {
                    save_res.map_err(|_| AzureError::ErrorWritingSaveFile)
                })
                .unwrap_or(Ok(()))
                .map(|_| (ffxiv, collects, uncollects));
            next
        })
//        .map(|_| ())
        .and_then(|(ffxiv, collects, _uncollects)| {
            let export_result = bgm_opts.export_mode.clone()
                .and_then(|export_mode| {
                    callbacks.pre_phase(AzureProcessPhase::Exporting);
                    let out_option = Some({
                        DirBuilder::new()
                            .recursive(true)
                            .create(export_mode.get_path())
                            .map_err(|_| AzureError::ErrorExporting("Creating directory"))
                            .and_then(|_| {
                                collects.iter().map(|t_mf| {
                                    ffxiv.get_exfile(&t_mf.name)
                                        .map(|exf| (t_mf.index, exf))
                                }).collect::<Result<Vec<_>, _>>()
                                    .and_then(|work| {
                                        let index_name_map = work
                                            .iter().map(|(index, exf)| (*index, exf.get_exfile_string().clone()))
                                            .collect::<HashMap<usize, String>>();
                                        callbacks.process_begin(AzureProcessBegin{total_operations_count: work.len()});
                                        let recv = async_processor(azure_opts.thread_count, ffxiv.clone(), &work, move |index, data| {
                                            index_name_map.get(&index).map_or(ThreadStatus::Error(format!("Invalid index passed to exporter! Index: {}", index), index), |f_name| {
                                                let a: Vec<&str> = f_name.split("/").skip(1).collect();
                                                ffxiv.decode_sound(data)
                                                    .map_err(|_| AzureError::ErrorDecoding)
                                                    .and_then(|scd| {
                                                        let entry_count = scd.header.entry_count as usize;
                                                        let base_path = a.join("/");
                                                        scd.entries.into_iter()
                                                            .rev()
                                                            .enumerate()
                                                            .map(|(index, entry)| {
                                                                let mut decoded_ogg = Vec::new();
                                                                decoded_ogg.clone_from(entry.decoded());
                                                                export_mode.export_file(base_path.as_str(), index, entry_count, decoded_ogg)
                                                            })
                                                            .collect::<Result<(), AzureError>>()
                                                            .map(|_| ThreadStatus::Continue(index))
                                                    })
                                                    .unwrap_or_else(|err| ThreadStatus::Error(format!("Failed to decode SCD: {}, reason: {:?}", f_name, err), index))
                                            })
                                        });
                                        let mut threads_completed = 0usize;
                                        let mut files_completed = 0usize;
                                        let mut files_errored = 0usize;
                                        'thread_recv: for received in recv {
                                            match received {
                                                ThreadStatus::Continue(index) => {
                                                    files_completed += 1;
                                                    callbacks.process_progress(AzureProcessProgress {
                                                        total_operations_count: work.len(),
                                                        is_skip: false,
                                                        current_operation: index,
                                                        operations_progress: files_completed
                                                    })
                                                },
                                                ThreadStatus::Complete => {
                                                    threads_completed += 1;
                                                    if threads_completed == azure_opts.thread_count {
                                                        break 'thread_recv;
                                                    }

                                                },
//                                                ThreadStatus::Skip => {
//                                                    files_completed += 1;
//                                                    // todo call skip func
//                                                },
                                                ThreadStatus::Error(reason, current_operation) => {
                                                    files_completed += 1;
                                                    files_errored += 1;
                                                    callbacks.process_nonfatal_error(AzureProcessNonfatalError {
                                                        current_operation,
                                                        reason
                                                    });
                                                },
                                            } }
                                        callbacks.process_complete(AzureProcessComplete {
                                            operations_completed: files_completed,
                                            operations_errored: files_errored
                                        });
                                        Ok(())
                                    })
                                    .map_err(|o| AzureError::FFXIVError(o))

                            })

                    });
                    callbacks.post_phase(AzureProcessPhase::Exporting);
                    out_option
                })
                .unwrap_or(Ok(()));

            export_result
//                .map(|export_result| {
//
//                })
        })
//    let sheet = ffxiv.get_sheet(
//        &String::from("bgm"),
//        SheetLanguage::English,
//        &ffxiv.get_sheet_index().unwrap()).unwrap();
//
//    let a = async_processor(1, ffxiv, Vec::new(), |data| {
//        ThreadStatus::Continue(3usize)
//    });
}