use async_trait::async_trait;
use constants::pagination::{MAX_PAGE_SIZE, MIN_PAGE_NUMBER, MIN_PAGE_SIZE};
use types::{
    dashboard::{
        DashboardActivityRequest, DashboardApiKeyLeaderboardRequest, DashboardCostAnalysisPreset, DashboardCostForecastRequest, DashboardCostSavingsRequest,
        DashboardFilterOptionsRequest, DashboardOverviewRequest, DashboardPreset, DashboardProviderAggregationRequest, DashboardScopeParam,
        DashboardUserStatsGranularity, DashboardUserStatsLeaderboardRequest, DashboardUserStatsTimeSeriesRequest, DashboardUserUsageStatsRequest,
    },
    pagination::PageRequest,
};

use crate::application::{
    DashboardActivityQuery, DashboardActor, DashboardApiKeyLeaderboardQuery, DashboardBucket, DashboardCostAnalysisWindow, DashboardCostForecastQuery,
    DashboardCostSavingsQuery, DashboardError, DashboardFilterOptionsQuery, DashboardOverviewQuery, DashboardProviderAggregationQuery, DashboardRepository,
    DashboardResult, DashboardScope, DashboardUseCase, DashboardUserStatsBucket, DashboardUserStatsLeaderboardQuery, DashboardUserStatsTimeSeriesQuery,
    DashboardUserStatsWindow, DashboardUserUsageStatsQuery, DashboardWindowBounds,
};

#[cfg(test)]
#[path = "service_tests.rs"]
mod service_tests;

const ADMIN_ROLE: &str = "admin";
const ACTIVITY_DAYS: i64 = 365;
const ONE_DAY: i64 = 1;
const DEFAULT_USER_STATS_PRESET: DashboardPreset = DashboardPreset::SevenDays;
const MAX_FORECAST_DAYS: u32 = 90;

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

    async fn user_stats_leaderboard(
        &self,
        actor: DashboardActor,
        request: DashboardUserStatsLeaderboardRequest,
    ) -> DashboardResult<types::dashboard::DashboardUserStatsLeaderboardResponse> {
        ensure_admin(&actor)?;
        let window = user_stats_window(
            request.start_date.as_deref(),
            request.end_date.as_deref(),
            request.preset,
            request.tz_offset_minutes,
        )?;
        self.repository
            .user_stats_leaderboard(DashboardUserStatsLeaderboardQuery {
                window,
                metric: request.metric,
                limit: validated_limit(request.limit)?,
                offset: request.offset,
            })
            .await
    }

    async fn user_usage_stats(
        &self,
        actor: DashboardActor,
        request: DashboardUserUsageStatsRequest,
    ) -> DashboardResult<types::dashboard::DashboardUserUsageStatsResponse> {
        ensure_admin(&actor)?;
        let window = user_stats_window(
            request.start_date.as_deref(),
            request.end_date.as_deref(),
            request.preset,
            request.tz_offset_minutes,
        )?;
        self.repository
            .user_usage_stats(DashboardUserUsageStatsQuery {
                window,
                user_id: clean_optional(request.user_id),
            })
            .await
    }

    async fn user_stats_time_series(
        &self,
        actor: DashboardActor,
        request: DashboardUserStatsTimeSeriesRequest,
    ) -> DashboardResult<Vec<types::dashboard::DashboardUserStatsTimeSeriesPoint>> {
        ensure_admin(&actor)?;
        let window = user_stats_window(
            request.start_date.as_deref(),
            request.end_date.as_deref(),
            request.preset,
            request.tz_offset_minutes,
        )?;
        self.repository
            .user_stats_time_series(DashboardUserStatsTimeSeriesQuery {
                window,
                bucket: user_stats_bucket(request.granularity),
                user_id: clean_optional(request.user_id),
            })
            .await
    }

    async fn cost_forecast(
        &self,
        actor: DashboardActor,
        request: DashboardCostForecastRequest,
    ) -> DashboardResult<types::dashboard::DashboardCostForecastResponse> {
        ensure_admin(&actor)?;
        let window = cost_analysis_window(&request.range)?;
        self.repository
            .cost_forecast(DashboardCostForecastQuery {
                window,
                forecast_days: validated_forecast_days(request.forecast_days)?,
            })
            .await
    }

    async fn cost_savings(
        &self,
        actor: DashboardActor,
        request: DashboardCostSavingsRequest,
    ) -> DashboardResult<types::dashboard::DashboardCostSavingsResponse> {
        ensure_admin(&actor)?;
        self.repository
            .cost_savings(DashboardCostSavingsQuery {
                window: cost_analysis_window(&request.range)?,
            })
            .await
    }

    async fn api_key_leaderboard(
        &self,
        actor: DashboardActor,
        request: DashboardApiKeyLeaderboardRequest,
    ) -> DashboardResult<types::dashboard::DashboardApiKeyLeaderboardResponse> {
        ensure_admin(&actor)?;
        self.repository
            .api_key_leaderboard(DashboardApiKeyLeaderboardQuery {
                window: cost_analysis_window(&request.range)?,
                metric: request.metric,
                order: request.order,
                limit: validated_limit(request.limit)?,
                offset: request.offset,
                include_inactive: request.include_inactive,
                exclude_admin: request.exclude_admin,
            })
            .await
    }

    async fn provider_aggregation(
        &self,
        actor: DashboardActor,
        request: DashboardProviderAggregationRequest,
    ) -> DashboardResult<Vec<types::dashboard::DashboardProviderAggregationItem>> {
        ensure_admin(&actor)?;
        if request.group_by != "provider" {
            return Err(DashboardError::InvalidInput("group_by must be provider".into()));
        }
        self.repository
            .provider_aggregation(DashboardProviderAggregationQuery {
                window: cost_analysis_window(&request.range)?,
                limit: validated_limit(request.limit)?,
            })
            .await
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

fn user_stats_window(
    start_date: Option<&str>,
    end_date: Option<&str>,
    preset: Option<DashboardPreset>,
    offset_minutes: i32,
) -> DashboardResult<DashboardUserStatsWindow> {
    match (start_date, end_date) {
        (Some(start), Some(end)) => explicit_user_stats_window(start, end, offset_minutes),
        (None, None) => preset_user_stats_window(preset.unwrap_or(DEFAULT_USER_STATS_PRESET), offset_minutes),
        _ => Err(DashboardError::InvalidInput("start_date and end_date must be provided together".into())),
    }
}

fn cost_analysis_window(request: &types::dashboard::DashboardCostAnalysisRequest) -> DashboardResult<DashboardCostAnalysisWindow> {
    let now = time::OffsetDateTime::now_utc();
    let offset = utc_offset(request.tz_offset_minutes)?;
    let today = now.to_offset(offset).date();
    let (start_date, end_date) = match request.preset {
        DashboardCostAnalysisPreset::Today => (today, today),
        DashboardCostAnalysisPreset::Yesterday => {
            let yesterday = today
                .previous_day()
                .ok_or_else(|| DashboardError::InvalidInput("yesterday is out of range".into()))?;
            (yesterday, yesterday)
        }
        DashboardCostAnalysisPreset::Last7Days => preset_date_range(today, 7)?,
        DashboardCostAnalysisPreset::Last30Days => preset_date_range(today, 30)?,
        DashboardCostAnalysisPreset::Last90Days => preset_date_range(today, 90)?,
        DashboardCostAnalysisPreset::Custom => custom_cost_date_range(request.start_date.as_deref(), request.end_date.as_deref())?,
    };
    Ok(DashboardCostAnalysisWindow {
        start_date,
        end_date,
        started_at: local_date_start_utc(start_date, request.tz_offset_minutes)?,
        ended_at: local_date_start_utc(
            end_date
                .next_day()
                .ok_or_else(|| DashboardError::InvalidInput("end_date is out of range".into()))?,
            request.tz_offset_minutes,
        )?,
        tz_offset_minutes: request.tz_offset_minutes,
    })
}

fn preset_date_range(today: time::Date, days: i64) -> DashboardResult<(time::Date, time::Date)> {
    let start = today - time::Duration::days(days - ONE_DAY);
    Ok((start, today))
}

fn custom_cost_date_range(start_date: Option<&str>, end_date: Option<&str>) -> DashboardResult<(time::Date, time::Date)> {
    let start_date = parse_date(
        start_date.ok_or_else(|| DashboardError::InvalidInput("start_date is required for custom range".into()))?,
        "start_date",
    )?;
    let end_date = parse_date(
        end_date.ok_or_else(|| DashboardError::InvalidInput("end_date is required for custom range".into()))?,
        "end_date",
    )?;
    if start_date > end_date {
        return Err(DashboardError::InvalidInput("start_date must be before or equal to end_date".into()));
    }
    Ok((start_date, end_date))
}

fn validated_forecast_days(value: u32) -> DashboardResult<u32> {
    if value == 0 || value > MAX_FORECAST_DAYS {
        return Err(DashboardError::InvalidInput(format!("forecast_days must be between 1 and {MAX_FORECAST_DAYS}")));
    }
    Ok(value)
}

fn explicit_user_stats_window(start_date: &str, end_date: &str, offset_minutes: i32) -> DashboardResult<DashboardUserStatsWindow> {
    let start_date = parse_date(start_date, "start_date")?;
    let end_date = parse_date(end_date, "end_date")?;
    if start_date > end_date {
        return Err(DashboardError::InvalidInput("start_date must be before or equal to end_date".into()));
    }
    let started_at = local_date_start_utc(start_date, offset_minutes)?;
    let ended_at = local_date_start_utc(
        end_date
            .next_day()
            .ok_or_else(|| DashboardError::InvalidInput("end_date is out of range".into()))?,
        offset_minutes,
    )?;
    Ok(DashboardUserStatsWindow {
        start_date,
        end_date,
        started_at,
        ended_at,
    })
}

fn preset_user_stats_window(preset: DashboardPreset, offset_minutes: i32) -> DashboardResult<DashboardUserStatsWindow> {
    let overview = overview_windows(preset, offset_minutes)?;
    Ok(DashboardUserStatsWindow {
        start_date: local_date(overview.selected.started_at, offset_minutes)?,
        end_date: local_date(overview.selected.ended_at - time::Duration::days(ONE_DAY), offset_minutes)?,
        started_at: overview.selected.started_at,
        ended_at: overview.selected.ended_at,
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

fn local_date_start_utc(date: time::Date, offset_minutes: i32) -> DashboardResult<time::OffsetDateTime> {
    let offset = utc_offset(offset_minutes)?;
    date.with_hms(0, 0, 0)
        .map(|value| value.assume_offset(offset).to_offset(time::UtcOffset::UTC))
        .map_err(|error| DashboardError::InvalidInput(format!("invalid local date boundary: {error}")))
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

fn validated_limit(limit: u64) -> DashboardResult<u64> {
    if limit < MIN_PAGE_SIZE || limit > MAX_PAGE_SIZE {
        return Err(DashboardError::InvalidInput(format!(
            "limit must be between {MIN_PAGE_SIZE} and {MAX_PAGE_SIZE}"
        )));
    }
    Ok(limit)
}

fn clean_required(value: Option<&str>, field: &str) -> DashboardResult<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| DashboardError::InvalidInput(format!("{field} is required")))
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty())
}

fn parse_date(value: &str, field: &str) -> DashboardResult<time::Date> {
    time::Date::parse(value, &time::format_description::well_known::Iso8601::DEFAULT)
        .map_err(|error| DashboardError::InvalidInput(format!("{field} must use YYYY-MM-DD: {error}")))
}

fn is_admin(actor: &DashboardActor) -> bool {
    actor.role == ADMIN_ROLE
}

fn ensure_admin(actor: &DashboardActor) -> DashboardResult<()> {
    if !is_admin(actor) {
        return Err(DashboardError::Forbidden("only administrators can request user statistics".into()));
    }
    Ok(())
}

fn user_stats_bucket(granularity: DashboardUserStatsGranularity) -> DashboardUserStatsBucket {
    match granularity {
        DashboardUserStatsGranularity::Hour => DashboardUserStatsBucket::Hour,
        DashboardUserStatsGranularity::Day => DashboardUserStatsBucket::Day,
    }
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
