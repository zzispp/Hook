use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

pub(super) struct CostMetrics {
    pub upstream_total_cost: Decimal,
    pub profit: Decimal,
    pub profit_rate: f64,
}

pub(super) fn admin_cost_metrics(total_cost: Decimal, upstream_total_cost: Decimal, include_admin_costs: bool) -> CostMetrics {
    if !include_admin_costs {
        return CostMetrics {
            upstream_total_cost: Decimal::ZERO,
            profit: Decimal::ZERO,
            profit_rate: 0.0,
        };
    }
    let profit = profit(total_cost, upstream_total_cost);
    CostMetrics {
        upstream_total_cost,
        profit,
        profit_rate: profit_rate(total_cost, profit),
    }
}

fn profit(total_cost: Decimal, upstream_total_cost: Decimal) -> Decimal {
    total_cost - upstream_total_cost
}

fn profit_rate(total_cost: Decimal, profit: Decimal) -> f64 {
    if total_cost <= Decimal::ZERO {
        return 0.0;
    }
    (profit / total_cost).to_f64().unwrap_or(0.0)
}
