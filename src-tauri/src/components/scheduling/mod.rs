pub mod adapters;
pub mod application;
pub mod availability;
pub mod domain;

#[cfg(test)]
mod application_tests;
#[cfg(test)]
mod availability_query_tests;
#[cfg(test)]
mod availability_tests;
#[cfg(test)]
mod repository_tests;
#[cfg(test)]
#[cfg(test)]
mod block_tests;
#[cfg(test)]
mod exception_tests;
#[cfg(test)]
mod online_tests;
#[cfg(test)]
mod rule_tests;
#[cfg(test)]
pub(crate) mod test_support;
