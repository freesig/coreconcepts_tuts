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

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Post {
    message: String,
    timestamp: u64,
    author_id: Address,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct InvalidPost {
    post: Post,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Agent {
    id: String,
}

#[zome]
mod my_zome {

    #[init]
    fn init() {
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
            validation: | validation_data: hdk::EntryValidationData<Post>| {
                let too_long = |message: String| if message.len() > 140 {
                            Err("Post is too long".into())
                        } else {
                            Ok(())
                        };
                match validation_data {
                    hdk::EntryValidationData::Create{entry, validation_data}=> {
                        let chain_header = validation_data.package.chain_header;
                        let entries = validation_data.package.source_chain_entries;
                        let headers = validation_data.package.source_chain_headers;
                        hdk::debug(format!("cheese chain_header {:?}", chain_header))?;
                        for entry in entries {
                            hdk::debug(format!("cheese entry_ {:?}", entry))?;
                        }
                        for header in headers {
                            hdk::debug(format!("cheese header_ {:?}", header))?;
                        }
                        too_long(entry.message.clone())
                        .and_then(|_|banned(entry.author_id))
                    },
                    hdk::EntryValidationData::Modify{new_entry, ..} => too_long(new_entry.message.clone())
                        .and_then(|_|banned(new_entry.author_id)),
                    _ => Ok(()),
                }
            }
        )
    }

    #[entry_def]
    fn invalid_post_entry_def() -> ValidatingEntryType {
        entry!(
            name: "invalid_post",
            description: "A blog post that is too long",
            sharing: Sharing::Public,
            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },
            validation: | validation_data: hdk::EntryValidationData<InvalidPost>| {
                let too_short = |message: String| if message.len() <= 140 {
                            Err("Invalid post is actually valid".into())
                        } else {
                            Ok(())
                        };
                match validation_data {
                    hdk::EntryValidationData::Create{entry, ..} => {
                        too_short(entry.post.message)
                    },
                    hdk::EntryValidationData::Modify{..} => Err("Modifying an invalid post is not possible".into()),
                    hdk::EntryValidationData::Delete{..} => Err("Deleting an invalid post is not possible".into()),
                }
            }
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
            ),
            to!(
                "invalid_post",
                link_type: "invalid_posts",
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
        let id: String = hdk::AGENT_ADDRESS.clone().into();
        let agent_id = Agent { id };
        let entry = Entry::App("agent".into(), agent_id.into());
        let agent_address = hdk::commit_entry(&entry)?;
        let entry = Entry::App("post".into(), post.clone().into());
        let address = hdk::commit_entry(&entry);
        hdk::debug(format!("cheese {:?}", address)).ok();
        let address = match address {
            Ok(address) => address,
            Err(err) => {
                if banned(post.author_id.clone()).is_ok() {
                    let invalid_post = InvalidPost { post };
                    let invalid_post_entry = Entry::App("invalid_post".into(), invalid_post.into());
                    let invalid_post_address = hdk::commit_entry(&invalid_post_entry)?;
                    hdk::link_entries(&agent_address, &invalid_post_address, "invalid_posts", "")?;
                }
                return Err(err);
            }
        };
        hdk::link_entries(&agent_address, &address, "author_post", "")?;
        Ok(address)
    }

    #[zome_fn("hc_public")]
    fn retrieve_posts(address: Address) -> ZomeApiResult<Vec<(Address, Post)>> {
        let id: String = address.into();
        let agent_id = Agent { id };
        let entry = Entry::App("agent".into(), agent_id.into());
        let agent_address = hdk::commit_entry(&entry)?;
        let posts = hdk::get_links(
            &agent_address,
            LinkMatch::Exactly("author_post"),
            LinkMatch::Any,
        )?;
        let addresses = posts.addresses();
        let posts = addresses
            .iter()
            .filter_map(|address| {
                hdk::utils::get_as_type(address.clone())
                    .ok()
                    .map(|post| (address.clone(), post))
            })
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

fn banned(agent_address: Address) -> Result<(), String> {
    let agent_id = Agent { id: agent_address.to_string() };
    let entry = Entry::App("agent".into(), agent_id.into());
    let agent_address = hdk::commit_entry(&entry)?;
    hdk::get_links_count(
        &agent_address,
        LinkMatch::Exactly("invalid_posts"),
        LinkMatch::Any,
    )
    .map_err(|_| "Agent not found".into())
    .and_then(|count| {
        if count.count > 3 {
            Err("This agent is banned".into())
        } else {
            Ok(())
        }
    })
}
