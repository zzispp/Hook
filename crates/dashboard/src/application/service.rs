use async_trait::async_trait;
use constants::pagination::{MAX_PAGE_SIZE, MIN_PAGE_NUMBER, MIN_PAGE_SIZE};
use types::{
    dashboard::{DashboardActivityRequest, DashboardFilterOptionsRequest, DashboardOverviewRequest, DashboardPreset, DashboardScopeParam},
    pagination::PageRequest,
};

use crate::application::{
    DashboardActivityQuery, DashboardActor, DashboardBucket, DashboardError, DashboardFilterOptionsQuery, DashboardOverviewQuery, DashboardRepository,
    DashboardResult, DashboardScope, DashboardUseCase, DashboardWindowBounds,
};

#[cfg(test)]
#[path = "service_tests.rs"]
mod service_tests;

const ADMIN_ROLE: &str = "admin";
const ACTIVITY_DAYS: i64 = 365;
const ONE_DAY: i64 = 1;

pub struct DashboardService<R> {
    repository: R,
}

impl<R> DashboardService<R>
where
    R: DashboardRepository,
{
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> DashboardUseCase for DashboardService<R>
where
    R: DashboardRepository,
{
    async fn overview(&self, actor: DashboardActor, request: DashboardOverviewRequest) -> DashboardResult<types::dashboard::DashboardOverviewResponse> {
        let scope = request_scope(&actor, request.scope, request.user_id.as_deref(), request.token_id.as_deref())?;
        let window = overview_windows(request.preset, request.tz_offset_minutes)?;
        let bucket = overview_bucket(request.preset);
        let daily_page = validated_page(request.page, request.page_size)?;
        let query = DashboardOverviewQuery {
            preset: request.preset,
            scope,
            window: window.selected,
            today_window: window.today,
            monthly_window: window.monthly,
            bucket,
            admin: is_admin(&actor),
            tz_offset_minutes: request.tz_offset_minutes,
            daily_page,
        };
        self.repository.overview(query).await
    }

    async fn activity(&self, actor: DashboardActor, request: DashboardActivityRequest) -> DashboardResult<types::dashboard::DashboardActivityResponse> {
        let scope = request_scope(&actor, request.scope, request.user_id.as_deref(), request.token_id.as_deref())?;
        let window = activity_window(request.tz_offset_minutes)?;
        self.repository
            .activity(DashboardActivityQuery {
                scope,
                start_date: window.start_date,
                end_date: window.end_date,
                started_at: window.started_at,
                ended_at: window.ended_at,
                admin: is_admin(&actor),
                tz_offset_minutes: request.tz_offset_minutes,
            })
            .await
    }

    async fn filter_options(
        &self,
        actor: DashboardActor,
        _request: DashboardFilterOptionsRequest,
    ) -> DashboardResult<types::dashboard::DashboardFilterOptionsResponse> {
        let scope = if is_admin(&actor) {
            DashboardScope::Global
        } else {
            DashboardScope::Me {
                user_id: actor.user_id.clone(),
            }
        };
        self.repository.filter_options(DashboardFilterOptionsQuery { scope }).await
    }
}

fn default_scope(actor: &DashboardActor) -> DashboardScope {
    if is_admin(actor) {
        return DashboardScope::Global;
    }
    DashboardScope::Me {
        user_id: actor.user_id.clone(),
    }
}

fn request_scope(actor: &DashboardActor, scope: Option<DashboardScopeParam>, user_id: Option<&str>, token_id: Option<&str>) -> DashboardResult<DashboardScope> {
    let Some(requested) = scope else {
        return Ok(default_scope(actor));
    };
    if !is_admin(actor) && requested != DashboardScopeParam::Me {
        return Err(DashboardError::Forbidden("only administrators can request dashboard scope filters".into()));
    }
    match requested {
        DashboardScopeParam::Me => me_scope(actor, user_id),
        DashboardScopeParam::Global => Ok(DashboardScope::Global),
        DashboardScopeParam::User => required_user_scope(user_id),
        DashboardScopeParam::Token => required_token_scope(token_id),
    }
}

fn me_scope(actor: &DashboardActor, user_id: Option<&str>) -> DashboardResult<DashboardScope> {
    if !is_admin(actor) && user_id.is_some_and(|id| id != actor.user_id) {
        return Err(DashboardError::Forbidden("users can only request their own dashboard activity".into()));
    }
    Ok(DashboardScope::Me {
        user_id: actor.user_id.clone(),
    })
}

fn required_user_scope(user_id: Option<&str>) -> DashboardResult<DashboardScope> {
    let user_id = clean_required(user_id, "user_id")?;
    Ok(DashboardScope::User { user_id })
}

fn required_token_scope(token_id: Option<&str>) -> DashboardResult<DashboardScope> {
    let token_id = clean_required(token_id, "token_id")?;
    Ok(DashboardScope::Token { token_id })
}

fn overview_windows(preset: DashboardPreset, offset_minutes: i32) -> DashboardResult<OverviewWindows> {
    let now = time::OffsetDateTime::now_utc();
    let today_start = local_day_start_utc(now, offset_minutes)?;
    let days = preset_days(preset);
    Ok(OverviewWindows {
        selected: DashboardWindowBounds {
            started_at: today_start - time::Duration::days(days - ONE_DAY),
            ended_at: today_start + time::Duration::days(ONE_DAY),
        },
        today: DashboardWindowBounds {
            started_at: today_start,
            ended_at: today_start + time::Duration::days(ONE_DAY),
        },
        monthly: DashboardWindowBounds {
            started_at: local_month_start_utc(now, offset_minutes)?,
            ended_at: today_start + time::Duration::days(ONE_DAY),
        },
    })
}

fn local_month_start_utc(now_utc: time::OffsetDateTime, offset_minutes: i32) -> DashboardResult<time::OffsetDateTime> {
    let offset = utc_offset(offset_minutes)?;
    let local = now_utc.to_offset(offset);
    local
        .date()
        .replace_day(1)
        .map_err(|error| DashboardError::InvalidInput(format!("invalid local month boundary: {error}")))?
        .with_hms(0, 0, 0)
        .map(|value| value.assume_offset(offset).to_offset(time::UtcOffset::UTC))
        .map_err(|error| DashboardError::InvalidInput(format!("invalid local month boundary: {error}")))
}

fn activity_window(offset_minutes: i32) -> DashboardResult<ActivityWindow> {
    let now = time::OffsetDateTime::now_utc();
    let end_day_start = local_day_start_utc(now, offset_minutes)?;
    let started_at = end_day_start - time::Duration::days(ACTIVITY_DAYS - ONE_DAY);
    let ended_at = end_day_start + time::Duration::days(ONE_DAY);
    Ok(ActivityWindow {
        start_date: local_date(started_at, offset_minutes)?,
        end_date: local_date(end_day_start, offset_minutes)?,
        started_at,
        ended_at,
    })
}

fn local_day_start_utc(now_utc: time::OffsetDateTime, offset_minutes: i32) -> DashboardResult<time::OffsetDateTime> {
    let offset = utc_offset(offset_minutes)?;
    let local = now_utc.to_offset(offset);
    local
        .date()
        .with_hms(0, 0, 0)
        .map(|value| value.assume_offset(offset).to_offset(time::UtcOffset::UTC))
        .map_err(|error| DashboardError::InvalidInput(format!("invalid local day boundary: {error}")))
}

fn local_date(value: time::OffsetDateTime, offset_minutes: i32) -> DashboardResult<time::Date> {
    Ok(value.to_offset(utc_offset(offset_minutes)?).date())
}

fn utc_offset(offset_minutes: i32) -> DashboardResult<time::UtcOffset> {
    let seconds = offset_minutes
        .checked_mul(60)
        .ok_or_else(|| DashboardError::InvalidInput("tz_offset_minutes exceeds supported range".into()))?;
    time::UtcOffset::from_whole_seconds(seconds).map_err(|_| DashboardError::InvalidInput("tz_offset_minutes must be between -1439 and 1439".into()))
}

fn preset_days(preset: DashboardPreset) -> i64 {
    match preset {
        DashboardPreset::Today => 1,
        DashboardPreset::SevenDays => 7,
        DashboardPreset::ThirtyDays => 30,
        DashboardPreset::NinetyDays => 90,
    }
}

fn overview_bucket(preset: DashboardPreset) -> DashboardBucket {
    match preset {
        DashboardPreset::Today => DashboardBucket::Hour,
        DashboardPreset::SevenDays | DashboardPreset::ThirtyDays | DashboardPreset::NinetyDays => DashboardBucket::Day,
    }
}

fn validated_page(page: u64, page_size: u64) -> DashboardResult<PageRequest> {
    if page < MIN_PAGE_NUMBER {
        return Err(DashboardError::InvalidInput("page must be greater than 0".into()));
    }
    if page_size < MIN_PAGE_SIZE || page_size > MAX_PAGE_SIZE {
        return Err(DashboardError::InvalidInput(format!(
            "page_size must be between {MIN_PAGE_SIZE} and {MAX_PAGE_SIZE}"
        )));
    }
    Ok(PageRequest { page, page_size })
}

fn clean_required(value: Option<&str>, field: &str) -> DashboardResult<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| DashboardError::InvalidInput(format!("{field} is required")))
}

fn is_admin(actor: &DashboardActor) -> bool {
    actor.role == ADMIN_ROLE
}

struct ActivityWindow {
    start_date: time::Date,
    end_date: time::Date,
    started_at: time::OffsetDateTime,
    ended_at: time::OffsetDateTime,
}

struct OverviewWindows {
    selected: DashboardWindowBounds,
    today: DashboardWindowBounds,
    monthly: DashboardWindowBounds,
}
