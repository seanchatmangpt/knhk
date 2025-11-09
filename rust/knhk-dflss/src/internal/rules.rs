//! Western Electric Rules for special cause detection

use crate::internal::chart::{ChartData, ShiftDirection, SpecialCause, TrendDirection};
use crate::internal::statistics::*;

pub fn check_western_electric_rules(data: &[ChartData]) -> Vec<SpecialCause> {
    let mut alerts = Vec::new();

    if data.len() < 9 {
        return alerts;
    }

    let recent = &data[data.len().saturating_sub(9)..];
    let latest = recent.last().unwrap();

    // Rule 1: Point beyond UCL or LCL
    if latest.value > latest.ucl || latest.value < latest.lcl {
        alerts.push(SpecialCause::OutOfControl {
            value: latest.value,
            ucl: latest.ucl,
            lcl: latest.lcl,
        });
    }

    // Rule 2: 9 consecutive points on same side of center line
    if recent.len() >= 9 {
        let above_cl = recent.iter().all(|d| d.value > d.cl);
        let below_cl = recent.iter().all(|d| d.value < d.cl);

        if above_cl {
            alerts.push(SpecialCause::Shift {
                direction: ShiftDirection::Above,
                count: 9,
            });
        } else if below_cl {
            alerts.push(SpecialCause::Shift {
                direction: ShiftDirection::Below,
                count: 9,
            });
        }
    }

    // Rule 3: 6 consecutive points increasing or decreasing
    if recent.len() >= 6 {
        let last_6 = &recent[recent.len() - 6..];
        let values: Vec<f64> = last_6.iter().map(|d| d.value).collect();

        let increasing = values.windows(2).all(|w| w[1] > w[0]);
        let decreasing = values.windows(2).all(|w| w[1] < w[0]);

        if increasing {
            alerts.push(SpecialCause::Trend {
                direction: TrendDirection::Increasing,
                count: 6,
            });
        } else if decreasing {
            alerts.push(SpecialCause::Trend {
                direction: TrendDirection::Decreasing,
                count: 6,
            });
        }
    }

    alerts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::chart::ChartData;

    fn create_chart_data(value: f64, ucl: f64, cl: f64, lcl: f64) -> ChartData {
        ChartData {
            timestamp: chrono::Utc::now().to_rfc3339(),
            value,
            ucl,
            cl,
            lcl,
            subgroup_data: None,
        }
    }

    #[test]
    fn test_rule_1_out_of_control() {
        let mut data = vec![];
        for i in 0..8 {
            data.push(create_chart_data(5.0 + i as f64, 10.0, 5.0, 0.0));
        }
        data.push(create_chart_data(11.0, 10.0, 5.0, 0.0)); // Out of control

        let alerts = check_western_electric_rules(&data);
        assert_eq!(alerts.len(), 1);
        assert!(matches!(alerts[0], SpecialCause::OutOfControl { .. }));
    }

    #[test]
    fn test_rule_2_shift() {
        let mut data = vec![];
        for _ in 0..9 {
            data.push(create_chart_data(6.0, 10.0, 5.0, 0.0)); // All above CL
        }

        let alerts = check_western_electric_rules(&data);
        assert_eq!(alerts.len(), 1);
        assert!(matches!(
            alerts[0],
            SpecialCause::Shift {
                direction: ShiftDirection::Above,
                ..
            }
        ));
    }

    #[test]
    fn test_rule_3_trend() {
        let mut data = vec![];
        for i in 0..5 {
            data.push(create_chart_data(i as f64, 10.0, 5.0, 0.0));
        }
        for i in 5..11 {
            data.push(create_chart_data(i as f64, 10.0, 5.0, 0.0)); // Increasing
        }

        let alerts = check_western_electric_rules(&data);
        assert!(alerts.iter().any(|a| matches!(
            a,
            SpecialCause::Trend {
                direction: TrendDirection::Increasing,
                ..
            }
        )));
    }
}
