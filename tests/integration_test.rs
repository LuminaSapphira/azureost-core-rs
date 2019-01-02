extern crate azure_ost_core;
struct MyCB;
use azure_ost_core::callbacks::*;
use azure_ost_core::*;

impl AzureCallbacks for MyCB {
    fn pre_phase(&self, phase: AzureProcessPhase) {
        println!("PRE: {:?}", phase);
    }
    fn post_phase(&self, phase: AzureProcessPhase) {
        println!("POST: {:?}", phase);
    }

    fn process_begin(&self, info: AzureProcessBegin) {
        println!("Process Begin: {:?}", info);
    }
    fn process_progress(&self, info: AzureProcessProgress) {
        println!("Process Progress: {:?}", info);
    }
    fn process_nonfatal_error(&self, info: AzureProcessNonfatalError) {
        println!("Process Error: {:?}", info);
    }
    fn process_complete(&self, info: AzureProcessComplete) {
        println!("Process Complete: {:?}", info);
    }
}

#[test]
fn it_works() {

}