
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

use ::sqpack_blue::{FFXIV, ExFileIdentifier, Index};
use ::threadpool::ThreadPool;
use ::sqpack_blue::FFXIVError;

pub enum ThreadStatus<T> {
    Continue(T),
    Complete,
    Error(String, usize),
}

fn get_index_from_map_or_insert<'a>(map: &'a mut HashMap<String, Index>, exf: &ExFileIdentifier, ffxiv: &FFXIV) -> Result<&'a Index, FFXIVError> {
    Ok(map.entry(exf.get_sqpack_base_file_name())
        .or_insert(ffxiv.get_index(exf)?))
}

pub fn async_processor<O: 'static, F: 'static>(thread_count: usize,
                                      ffxiv: FFXIV,
                                      work: &Vec<(usize, ExFileIdentifier)>,
                                      handler: F)
    -> Receiver<ThreadStatus<O>>
    where O: Send,
F: Fn(usize, Vec<u8>) -> ThreadStatus<O> + Send + Sync
{

    let data_handler = Arc::new(handler);

    let (tx, rx) = mpsc::channel();

    let distributed_work = split_work(work, thread_count);

    let pool = ThreadPool::new(thread_count);

    distributed_work.into_iter().for_each(|each_work| {
        let tx_n = tx.clone();
        let ffxiv = ffxiv.clone();
        let data_handler = data_handler.clone();
        pool.execute(move || {
            let mut ff_index_files = HashMap::new();
            each_work.into_iter().for_each(|(index, exf)| {
                get_index_from_map_or_insert(&mut ff_index_files, &exf, &ffxiv)
                    .and_then(|ff_index| {
                        ffxiv.get_raw_data_with_index(&exf, ff_index)
                    })
                    .and_then(|data| {
                        Ok(tx_n.send(data_handler(index, data)).ok())
                    })
                    .unwrap_or_else(|e| {
                        tx_n.send(ThreadStatus::Error(e.to_string(), index)).ok()
                    });
            });
            tx_n.send(ThreadStatus::Complete).ok();
        });
    });

    rx
}

fn split_work<T>(indices: &Vec<T>, split_count: usize) -> Vec<Vec<T>> where T: Clone {
    assert_ne!(split_count, 0, "Cannot split work between 0 loads!");
    let mut output = Vec::with_capacity(split_count as usize);
    let each = indices.len() / split_count as usize;
    let extra = indices.len() % split_count as usize;
    for i in 0..split_count {
        let start = (i as usize) * each + if (i as usize) < extra {
            i as usize
        } else {
            extra
        };
        let end = start + each + if (i as usize) < extra {
            1
        } else {
            0
        };
        let bounds = start..end;
        use std::iter::FromIterator;
        let inner = Vec::from_iter(indices[bounds].iter().cloned());
        output.push(inner);
    };

    output

}
