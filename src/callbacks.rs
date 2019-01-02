/// An enum used in the callback system to specify which phase of processing the processor is on.
/// It is guaranteed to appear in the order, however certain phases may be omitted, if the options
/// specify so.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum AzureProcessPhase {
    Begin,
    ReadingBGMSheet,
    Hashing,
    Collecting,
    SavingManifest,
    Exporting,
}

/// A structure used in the callback system during threaded processing. This is passed as an
/// argument to a callback to indicate that a process has begun and the number of operations that
/// should be expected.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct AzureProcessBegin {
    /// The total number of operations in this threaded process.
    pub total_operations_count: usize,
}

/// A structure used in the callback system during threaded processing. This is passed as an
/// argument to a callback to indicate that a process is continuing and contains certain data
/// accordingly.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct AzureProcessProgress {
    /// The total number of operations in this threaded process.
    pub total_operations_count: usize,
    /// The number of operations completed across all threads (the total progress)
    pub operations_progress: usize,

    /// Which operation was just completed. These may appear in any order due to the asynchronous
    /// nature of the operations.
    pub current_operation: usize,

    /// Whether or not this progress was a skip (as in, nothing occurred internally)
    pub is_skip: bool,
}

/// A structure used in the callback system during threaded processing. This is passed as an
/// argument to a callback to indicate that a process has experienced a nonfatal error and contains
/// certain data accordingly.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct AzureProcessNonfatalError {
    /// Which operation failed
    pub current_operation: usize,

    /// The reason passed to the callback system
    pub reason: String,
}

/// A structure used in the callback system during threaded processing. This is passed as an
/// argument to a callback to indicate that a process has completed and contains certain data
/// accordingly.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct AzureProcessComplete {
    /// The number of operations that were completed (successful or otherwise)
    pub operations_completed: usize,

    /// The number of operations that experienced an error
    pub operations_errored: usize,
}

/// A trait that may be implemented on a type to provide for callback functionality. A set of
/// functions is provided that will be called by the processor at certain points.
pub trait AzureCallbacks {
    /// This will be called before a phase begins. The phase is passed to the function
    fn pre_phase(&self, phase: AzureProcessPhase);

    /// This will be called after a phase has completed. The phase is passed to the function
    fn post_phase(&self, phase: AzureProcessPhase);


    /// This will be called when a threaded operation has begun
    fn process_begin(&self, info: AzureProcessBegin);

    /// This will be called when a threaded operation makes progress
    fn process_progress(&self, info: AzureProcessProgress);

    /// This will be called when a threaded operation experiences a nonfatal error
    fn process_nonfatal_error(&self, info: AzureProcessNonfatalError);

    /// This will be called when the threaded operation has completed
    fn process_complete(&self, info: AzureProcessComplete);
}

pub struct NoOpCallback;
impl AzureCallbacks for NoOpCallback {
    fn pre_phase(&self, _: AzureProcessPhase) {}
    fn post_phase(&self, _: AzureProcessPhase) {}
    fn process_begin(&self, _: AzureProcessBegin) {}
    fn process_progress(&self, _: AzureProcessProgress) {}
    fn process_nonfatal_error(&self, _: AzureProcessNonfatalError) {}
    fn process_complete(&self, _: AzureProcessComplete) {}
}