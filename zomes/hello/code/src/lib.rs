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
    link::LinkMatch,
};

use hdk::holochain_json_api::{
    json::JsonString,
    error::JsonError,
};

use hdk::holochain_persistence_api::{
    cas::content::Address
};

use hdk_proc_macros::zome;

// see https://developer.holochain.org/api/0.0.18-alpha1/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct Post {
    message: String,
    timestamp: u64,
    author_id: Address,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Agent {
    id: String,
}

#[zome]
mod hello_zome {

    #[init]
    fn init() {
        Ok(())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
        Ok(())
    }

    #[entry_def]
    fn post_entry_def() -> ValidatingEntryType {
        entry!(
            name: "post",
            description: "A blog post",
            sharing: Sharing::Public,
            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },
            validation: | _validation_data: hdk::EntryValidationData<Post>| {
                Ok(())
            },
            links: [
            from!(
                "%agent_id",
                link_type: "author_post",
               validation_package: || {
                   hdk::ValidationPackageDefinition::Entry
               },
               validation: |_validation_data: hdk::LinkValidationData| {
                   Ok(())
               }
            )
            ]
        )
    }

    #[entry_def]
    fn agent_entry_def() -> ValidatingEntryType {
        entry!(
            name: "agent",
            description: "Hash of agent",
            sharing: Sharing::Public,
            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },
            validation: | _validation_data: hdk::EntryValidationData<Agent>| {
                Ok(())
            },
            links: [
            to!(
                "post",
                link_type: "author_post",
               validation_package: || {
                   hdk::ValidationPackageDefinition::Entry
               },
               validation: |_validation_data: hdk::LinkValidationData| {
                   Ok(())
               }
            )
            ]
        )
    }

    #[zome_fn("hc_public")]
    pub fn create_post(message: String, timestamp: u64) -> ZomeApiResult<Address> {
        let post = Post {
            message,
            timestamp,
            author_id: hdk::AGENT_ADDRESS.clone(),
        };
        let agent_id = Agent { id: hdk::AGENT_ADDRESS.clone().into() };
        let entry = Entry::App("agent".into(), agent_id.into());
        let agent_address = hdk::commit_entry(&entry)?;
        let entry = Entry::App("post".into(), post.into());
        let address = hdk::commit_entry(&entry)?;
        hdk::link_entries(&agent_address, &address, "author_post", "")?;
        Ok(address)
    }

    #[zome_fn("hc_public")]
    fn retrieve_posts(address: Address) -> ZomeApiResult<Vec<Post>> {
        let agent_id = Agent { id: address.into() };
        let entry = Entry::App("agent".into(), agent_id.into());
        let agent_address = hdk::commit_entry(&entry)?;
        hdk::utils::get_links_and_load_type(
            &agent_address,
            LinkMatch::Exactly("author_post"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn hello_holo() -> ZomeApiResult<String> {
        Ok("Hello Holo".into())
    }

    #[zome_fn("hc_public")]
    fn get_agent_id() -> ZomeApiResult<Address> {
        Ok(hdk::AGENT_ADDRESS.clone())
    }

}
