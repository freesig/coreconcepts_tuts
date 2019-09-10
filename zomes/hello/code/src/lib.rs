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

use hdk::holochain_core_types::{dna::entry_types::Sharing, entry::Entry, link::LinkMatch};
use hdk::{entry_definition::ValidatingEntryType, error::ZomeApiResult};

use hdk::holochain_json_api::{error::JsonError, json::JsonString};

use hdk::holochain_persistence_api::cas::content::Address;

use hdk_proc_macros::zome;

// see https://developer.holochain.org/api/0.0.18-alpha1/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Post {
    message: String,
    timestamp: u64,
    author_id: Address,
}

#[zome]
mod my_zome {

    #[init]
    fn init() {
        Ok(())
    }

    // Turn this function into an entry definition.
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

    #[zome_fn("hc_public")]
    pub fn create_post(message: String, timestamp: u64) -> ZomeApiResult<Address> {
        let post = Post {
            message,
            timestamp,
            author_id: hdk::AGENT_ADDRESS.clone(),
        };
        let entry = Entry::App("post".into(), post.into());
        let address = hdk::commit_entry(&entry)?;
        hdk::link_entries(&hdk::AGENT_ADDRESS, &address, "author_post", "")?;
        Ok(address)
    }

    #[zome_fn("hc_public")]
    fn retrieve_posts(address: Address) -> ZomeApiResult<Vec<(Address, Post)>> {
        let posts = hdk::get_links(
            &address,
            LinkMatch::Exactly("author_post"),
            LinkMatch::Any,
            )?;
        let addresses = posts.addresses();
        let posts = addresses.iter()
            .filter_map(|address| hdk::utils::get_as_type(address.clone()).ok().map(|post|(address.clone(), post)))
            .collect();
        Ok(posts)

    }

    #[zome_fn("hc_public")]
    fn hello_holo() -> ZomeApiResult<String> {
        Ok("Hello Holo".into())
    }

    #[zome_fn("hc_public")]
    fn get_agent_id() -> ZomeApiResult<Address> {
        Ok(hdk::AGENT_ADDRESS.clone())
    }

    #[zome_fn("hc_public")]
    fn delete_post(post_address: Address) -> ZomeApiResult<()> {
        hdk::remove_entry(&post_address)?;
        Ok(())
    }

    #[zome_fn("hc_public")]
    fn update_post(
        post_address: Address,
        message: String,
        timestamp: u64,
    ) -> ZomeApiResult<Address> {
        let post = Post {
            message,
            timestamp,
            author_id: hdk::AGENT_ADDRESS.clone(),
        };
        let entry = Entry::App("post".into(), post.into());
        hdk::update_entry(entry, &post_address)
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
        Ok(())
    }

}
