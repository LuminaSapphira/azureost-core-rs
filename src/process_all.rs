use ::{BGMOptions, AzureOptions, AzureError, AzureCallbacks};
use ::sqpack_blue::sheet::ex::SheetLanguage;
use ::selector::Selector;

/// Process all files in the BGM sheet using the provided AzureOptions and BGMOptions. Callbacks are
/// made synchronously to the provided AzureCallbacks reference.
/// # Arguments
/// * `azure_opts` - The general options to use
/// * `bgm_opts` - The BGM options to use
/// * `callbacks` - A reference to an AzureCallbacks implementation. If no specific callback
/// functionality is desired `azure_ost_core::callbacks::NoOpCallback` may be used.
pub fn process_all(azure_opts: AzureOptions, bgm_opts: BGMOptions, callbacks: &AzureCallbacks) -> Result<(), AzureError>
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
        .map(|(_, sheet)| {
            sheet.rows.keys().cloned().collect::<Vec<usize>>()
        })
        .and_then(|process_indices| {
            ::general_processor::process(azure_opts, bgm_opts, process_indices, callbacks)
        })
}

/// Process all files in the BGM sheet using the provided AzureOptions and BGMOptions. Callbacks are
/// made synchronously to the provided AzureCallbacks reference.
/// # Arguments
/// * `selected` - A reference to a type that implements `azure_ost_core::selector::Selector`. This
/// specifies which row from the BGM sheet should be operated upon.
/// * `azure_opts` - The general options to use
/// * `bgm_opts` - The BGM options to use
/// * `callbacks` - A reference to an AzureCallbacks implementation. If no specific callback
/// functionality is desired `azure_ost_core::callbacks::NoOpCallback` may be used.
pub fn process_one(selected: &Selector, azure_opts: AzureOptions,
                       bgm_opts: BGMOptions, ac: &AzureCallbacks) -> Result<(), AzureError> {
    let ffxiv = azure_opts.ffxiv.clone();

    Ok(ffxiv)
        // get the Sheet index
        .and_then(|ffxiv| {
            selected.select_azure_ost(&ffxiv)
        })
        .and_then(|index| {
            ::general_processor::process(azure_opts, bgm_opts, vec![index], ac)
        })
}