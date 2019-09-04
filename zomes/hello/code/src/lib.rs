#![feature(proc_macro_hygiene)]
#[macro_use]
extern crate hdk;
extern crate hdk_proc_macros;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_json_derive;

use hdk::{
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
};
use hdk::holochain_core_types::{
    entry::Entry,
    dna::entry_types::Sharing,
};

use hdk::holochain_json_api::{
    json::JsonString,
    error::JsonError
};

use hdk::holochain_persistence_api::{
    cas::content::Address
};

use hdk_proc_macros::zome;

// see https://developer.holochain.org/api/0.0.18-alpha1/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

// Allow this struct to be easily converted to and from JSON
#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
// Represent a person as a struct that holds
// their name as a String.
pub struct Person{
    name: String,
}

#[zome]
mod my_zome {

    #[init]
    fn init() {
        Ok(())
    }

    // Turn this function into an entry definition.
    #[entry_def]
    fn person_entry_def() -> ValidatingEntryType {
        // A macro that lets you easily create a `ValidatingEntryType`.
        entry!(
            // The name of the entry.
            // This should be the lowercase version of
            // the struct name `Person`.
            name: "person",
            description: "Person to say hello to",
            // This is a private entry in your source chain.
            sharing: Sharing::Public,
            // Says what is needed to validate this entry.
            // In this case just the Entry.
            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },
            // Validates this entry.
            // Returns that this entry is always Ok as long as it type checks.
            validation: | _validation_data: hdk::EntryValidationData<Person>| {
                Ok(())
            }
        )
    }

    #[zome_fn("hc_public")]
    pub fn create_person(person: Person) -> ZomeApiResult<Address> {
        let entry = Entry::App("person".into(), person.into());
        let address = hdk::commit_entry(&entry)?;
        Ok(address)
    }

    #[zome_fn("hc_public")]
    fn retrieve_person(address: Address) -> ZomeApiResult<Option<Entry>> {
        hdk::get_entry(&address)
    }

    #[zome_fn("hc_public")]
    fn hello_holo() -> ZomeApiResult<String> {
        Ok("Hello Holo".into())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
        Ok(())
    }

}
