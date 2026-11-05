pub mod adapters;
pub mod application;
pub mod domain;

#[cfg(test)]
mod application_tests;
#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod completeness_tests;
#[cfg(test)]
mod contact_tests;
#[cfg(test)]
mod development_tests;
#[cfg(test)]
mod ipc_registration_tests;
#[cfg(test)]
mod reconciliation_tests;
#[cfg(test)]
mod relationship_tests;
#[cfg(test)]
mod repository_tests;
