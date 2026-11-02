//! Account rules. No storage, HTTP, or remote transport models.
use crate::shared_kernel::{NightingaleError, Result, id::AccountId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountStatus {
    Active,
    Inactive,
}

impl AccountStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

impl TryFrom<&str> for AccountStatus {
    type Error = NightingaleError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            _ => Err(NightingaleError::validation("estado de cuenta inválido")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountPolicy {
    no_show_grace_minutes: i64,
}

impl AccountPolicy {
    pub fn new(minutes: i64) -> Result<Self> {
        if !(0..=1440).contains(&minutes) {
            return Err(NightingaleError::validation(
                "la tolerancia debe estar entre 0 y 1440 minutos",
            ));
        }

        Ok(Self {
            no_show_grace_minutes: minutes,
        })
    }

    pub fn no_show_grace_minutes(self) -> i64 {
        self.no_show_grace_minutes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountView {
    pub account_id: AccountId,
    pub display_name: String,
    pub time_zone: String,
    pub location_name: String,
    pub location_address: String,
    pub location_city: String,
    pub location_state: String,
    pub location_country: String,
    pub policy: AccountPolicy,
    pub status: AccountStatus,
    pub version: i64,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountDetailsPatch(BTreeMap<String, String>);

impl AccountDetailsPatch {
    pub fn new(fields: BTreeMap<String, String>) -> Result<Self> {
        if fields.is_empty() {
            return Err(NightingaleError::validation(
                "el cambio de cuenta está vacío",
            ));
        }

        for (key, value) in &fields {
            let valid = match key.as_str() {
                "displayName" | "locationName" | "locationAddress" | "locationCity"
                | "locationState" => !value.trim().is_empty(),
                // Pinned IANA membership data; no date arithmetic or OS dependency.
                "timeZone" => include_str!("time_zones.txt")
                    .lines()
                    .any(|zone| zone == value),
                "locationCountry" => include_str!("countries.txt")
                    .lines()
                    .any(|country| country == value),
                _ => false,
            };

            if !valid {
                return Err(NightingaleError::validation("campo de cuenta inválido"));
            }
        }

        Ok(Self(fields))
    }

    pub fn fields(&self) -> &BTreeMap<String, String> {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

impl TryFrom<&str> for Theme {
    type Error = NightingaleError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "system" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(NightingaleError::validation("tema inválido")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenceKey {
    Theme,
    ServicesShowInactive,
}

impl TryFrom<&str> for PreferenceKey {
    type Error = NightingaleError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "theme" => Ok(Self::Theme),
            "servicesShowInactive" => Ok(Self::ServicesShowInactive),
            _ => Err(NightingaleError::validation(
                "preferencia local no permitida",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preference {
    Theme(Theme),
    ServicesShowInactive(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preferences {
    pub theme: Theme,
    pub services_show_inactive: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            services_show_inactive: true,
        }
    }
}

impl Preferences {
    pub fn apply(&mut self, preference: Preference) {
        match preference {
            Preference::Theme(value) => self.theme = value,
            Preference::ServicesShowInactive(value) => self.services_show_inactive = value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_has_inclusive_bounds() {
        for minutes in [0, 1, 1440] {
            assert_eq!(
                AccountPolicy::new(minutes).unwrap().no_show_grace_minutes(),
                minutes
            );
        }
        assert!(AccountPolicy::new(-1).is_err());
        assert!(AccountPolicy::new(1441).is_err());
    }

    #[test]
    fn patch_checks_membership_and_preserves_only_chosen_fields() {
        for (key, value) in [
            ("timeZone", "America/Caracas"),
            ("timeZone", "UTC"),
            ("timeZone", "US/Eastern"),
            ("locationCountry", "VE"),
            ("displayName", "Synthetic"),
        ] {
            let fields = BTreeMap::from([(key.to_owned(), value.to_owned())]);
            assert_eq!(
                AccountDetailsPatch::new(fields.clone()).unwrap().fields(),
                &fields
            );
        }
        for (key, value) in [
            ("timeZone", "America/Invented"),
            ("locationCountry", "ZZ"),
            ("locationCountry", "ve"),
            ("displayName", "  "),
            ("noShowGraceMinutes", "5"),
        ] {
            assert!(
                AccountDetailsPatch::new(BTreeMap::from([(key.to_owned(), value.to_owned())]))
                    .is_err()
            );
        }
        assert!(AccountDetailsPatch::new(BTreeMap::new()).is_err());
    }

    #[test]
    fn preferences_cannot_replace_policy_or_access() {
        for key in [
            "timeZone",
            "noShowGraceMinutes",
            "allowedActions",
            "token",
            "status",
        ] {
            assert!(PreferenceKey::try_from(key).is_err());
        }
        let mut preferences = Preferences::default();
        preferences.apply(Preference::Theme(Theme::Dark));
        preferences.apply(Preference::ServicesShowInactive(false));
        assert_eq!(
            preferences,
            Preferences {
                theme: Theme::Dark,
                services_show_inactive: false
            }
        );
    }
}
