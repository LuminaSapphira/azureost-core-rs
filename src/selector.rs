use ::AzureError;
use ::sqpack_blue::FFXIV;
use ::sqpack_blue::sheet::ex::SheetLanguage;

pub trait Selector {
    fn select_azure_ost(&self, ffxiv: &FFXIV) -> Result<usize, AzureError>;
}

impl Selector for usize {
    #[inline]
    fn select_azure_ost(&self, _: &FFXIV) -> Result<usize, AzureError> {
        Ok(*self)
    }
}

impl Selector for String {
    fn select_azure_ost(&self, ffxiv: &FFXIV) -> Result<usize, AzureError> {
        ffxiv.get_sheet_index()
            .and_then(|sheet_index| ffxiv.get_sheet(&String::from("bgm"), SheetLanguage::None, &sheet_index))
            .map_err(|_| AzureError::UnableToSelect)
            .and_then(|sheet| {
                sheet.rows.iter().enumerate().find(|row| {
                    (row.1).read_cell_data::<String>(0)
                        .map(|title| self.eq_ignore_ascii_case(title.as_str()))
                        .unwrap_or(false)
                })
                    .and_then(|row| {
                        Some(row.0)
                    })
                    .ok_or(AzureError::UnableToSelect)
            })

    }
}