use super::{
    account::adapters::outbound::AccountProjector,
    appointments::adapters::projector::AppointmentProjector,
    patients::adapters::projector::PatientsProjector,
    scheduling::adapters::projector::SchedulingProjector,
    services::adapters::projector::ServicesProjector,
    sync::remote::{RemoteChange, RemoteProjector},
};
use crate::shared_kernel::{Result, id::OperationContext};
use rusqlite::Transaction;

/// Compose accepted views at the desktop boundary, without changing sync rules.
pub(crate) struct DesktopProjector;

impl RemoteProjector for DesktopProjector {
    fn reset(&self, tx: &Transaction, scope: &OperationContext) -> Result<()> {
        AccountProjector.reset(tx, scope)?;
        ServicesProjector.reset(tx, scope)?;
        PatientsProjector.reset(tx, scope)?;
        SchedulingProjector.reset(tx, scope)?;
        AppointmentProjector.reset(tx, scope)
    }

    fn project(
        &self,
        tx: &Transaction,
        scope: &OperationContext,
        change: &RemoteChange,
    ) -> Result<()> {
        AccountProjector.project(tx, scope, change)?;
        ServicesProjector.project(tx, scope, change)?;
        PatientsProjector.project(tx, scope, change)?;
        SchedulingProjector.project(tx, scope, change)?;
        AppointmentProjector.project(tx, scope, change)
    }
}
