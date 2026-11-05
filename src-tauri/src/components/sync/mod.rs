pub mod adapters;
pub mod application;
pub mod coordinator;
pub mod dispatch;
pub mod domain;
pub mod download;
pub mod events;
pub mod remote;
pub mod snapshot;

#[cfg(test)]
mod contract_tests;

#[cfg(test)]
mod catalog_integration_tests;
#[cfg(test)]
mod patients_integration_tests;

#[cfg(test)]
mod matrix_tests;

#[cfg(test)]
mod development_tests;
