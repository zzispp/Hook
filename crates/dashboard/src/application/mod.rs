mod error;
mod ports;
mod service;

pub use error::{DashboardError, DashboardResult};
pub use ports::{
    DashboardActivityQuery, DashboardActor, DashboardBucket, DashboardFilterOptionsQuery, DashboardOverviewQuery, DashboardRepository, DashboardScope,
    DashboardUseCase, DashboardWindowBounds,
};
pub use service::DashboardService;
