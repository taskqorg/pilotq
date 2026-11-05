//! Human time forms shared by every surface (specq indicators § Time).
pub fn countdown(seconds: i64, ru: bool) -> String {
    let text = match seconds {
        ..=0 => "now".into(),
        86400.. => format!("{}d", (seconds - 1) / 86400 + 1),
        18000.. => format!("{}h", (seconds - 1) / 3600 + 1),
        _ => { let minutes = (seconds - 1) / 60 + 1; format!("{}:{:02}", minutes / 60, minutes % 60) }
    };
    if ru { crate::footer::russian_units(&text).replace("now", "сейчас") } else { text }
}

pub fn age(seconds: i64) -> String {
    match seconds {
        ..60 => "<1m".into(),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    }
}

pub fn exact(at: Option<i64>) -> String {
    at.and_then(|at| chrono::DateTime::from_timestamp(at, 0))
        .map(|at| at.format("%Y-%m-%d %H:%M UTC").to_string()).unwrap_or_else(|| "? unavailable".into())
}

pub fn moment(at: Option<i64>, now: i64) -> String {
    match at.filter(|at| chrono::DateTime::from_timestamp(*at, 0).is_some()) {
        Some(at) => format!("{} ago ({})", age(now.saturating_sub(at)), exact(Some(at))),
        None => "? unavailable".into(),
    }
}

pub fn source(at: Option<&str>, now: i64) -> String {
    moment(at.and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok()).map(|at| at.timestamp()), now)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #373 A6: rounding and cutovers must not hide minutes or invent healthy/known timestamps.
    #[test]
    fn boundaries_373() {
        for (seconds, expected) in [(-1,"now"),(0,"now"),(1,"0:01"),(60,"0:01"),(61,"0:02"),
            (17999,"5:00"),(18000,"5h"),(18001,"6h"),(86399,"24h"),(86400,"1d"),(86401,"2d")] {
            assert_eq!(countdown(seconds,false), expected);
        }
        for (seconds,expected) in [(0,"сейчас"),(1,"0:01"),(17999,"5:00"),(18000,"5 ч"),(18001,"6 ч"),(86400,"1 д")] {
            assert_eq!(countdown(seconds,true),expected);
        }
        for (seconds, expected) in [(-1,"<1m"),(0,"<1m"),(59,"<1m"),(60,"1m"),(3599,"59m"),
            (3600,"1h"),(86399,"23h"),(86400,"1d")] { assert_eq!(age(seconds),expected); }
        assert_eq!(source(Some("2026-11-05T14:30:01+02:00"),1793885401), "1h ago (2026-11-05 12:30 UTC)");
        for raw in [None,Some("unknown"),Some("2026-11-05 14:30 UTC")] { assert_eq!(source(raw,0),"? unavailable"); }
        assert_eq!(moment(Some(i64::MAX),0),"? unavailable");
        assert_eq!(exact(None),"? unavailable");
    }
}
