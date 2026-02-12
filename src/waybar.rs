use crate::models::{UsageStats, WaybarOutput};

pub fn generate_output(stats: &UsageStats, format: &str) -> String {
    let percentage = stats.percentage as i32;
    let text = format.replace("{percentage}", &percentage.to_string());

    let tooltip = format_tooltip(stats);
    let class = get_css_class(stats.percentage);

    let output = WaybarOutput {
        text,
        tooltip,
        class,
    };

    serde_json::to_string(&output).unwrap_or_default()
}

fn format_tooltip(stats: &UsageStats) -> String {
    let mut tooltip = format!(
        "GitHub Copilot Usage\n{} / {} ({:.1}%)\nResets: {}",
        stats.total_used,
        stats.total_limit,
        stats.percentage,
        stats.reset_date.format("%B %d, %Y at %H:%M UTC")
    );

    if !stats.models.is_empty() {
        tooltip.push_str("\n\nPer-model usage:");
        for model in &stats.models {
            tooltip.push_str(&format!(
                "\n  {}: {:.0} ({:.1}%)",
                model.name, model.used, model.percentage
            ));
        }
    }

    if stats.estimated_cost > 0.0 {
        tooltip.push_str(&format!("\n\nEstimated cost: ${:.2}", stats.estimated_cost));
    }

    tooltip
}

fn get_css_class(percentage: f64) -> String {
    if percentage >= 90.0 {
        "copilot-critical".to_string()
    } else if percentage >= 75.0 {
        "copilot-warning".to_string()
    } else if percentage >= 50.0 {
        "copilot-normal".to_string()
    } else {
        "copilot-low".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_classes() {
        assert_eq!(get_css_class(95.0), "copilot-critical");
        assert_eq!(get_css_class(80.0), "copilot-warning");
        assert_eq!(get_css_class(60.0), "copilot-normal");
        assert_eq!(get_css_class(30.0), "copilot-low");
    }
}
