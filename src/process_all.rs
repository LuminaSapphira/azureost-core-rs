use ::{BGMOptions, AzureOptions, AzureError, AzureCallbacks};
use ::sqpack_blue::sheet::ex::SheetLanguage;
use std::iter::FromIterator;

pub fn process_all<AC>(azure_opts: AzureOptions, bgm_opts: BGMOptions, ac: &AC) -> Result<(), AzureError>
    where AC: AzureCallbacks + Sized
    {

    let ffxiv = azure_opts.ffxiv.clone();

    Ok(ffxiv)
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
        .and_then(|(_, sheet)| {
            ::general_processor::process(azure_opts, bgm_opts, Vec::from_iter(2..sheet.rows.len()), ac)
        })

}