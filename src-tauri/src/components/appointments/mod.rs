pub mod adapters;
pub mod application;
pub mod domain;

#[cfg(test)]
mod application_tests;
#[cfg(test)]
mod cancel_tests;
#[cfg(test)]
mod complete_tests;
#[cfg(test)]
mod confirm_tests;
#[cfg(test)]
mod create_tests;
#[cfg(test)]
mod repository_tests;
#[cfg(test)]
mod reschedule_tests;
#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod no_show_tests;
#[cfg(test)]
mod occupancy_tests;
