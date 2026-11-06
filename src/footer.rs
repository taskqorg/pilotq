//! #119 / §12: turn-local, numeric-only runtime observations. No account cache or secrets.
use serde_json::Value;

#[derive(Default)]
pub struct Footer {
    context: Option<(u64, i64)>,
    quota: Option<(Vec<agentq_runtime::agent::Window>, i64)>,
    context_stale: bool,
}

impl Footer {
    pub fn context(&mut self, v: &Value, at: i64) {
        let valid = v["used"].as_u64().zip(v["window"].as_u64())
            .filter(|(_, window)| *window > 0)
            .and_then(|(used, window)| used.checked_mul(100).map(|n| n / window));
        if let Some(percent) = valid {
            self.context = Some((percent, at));
            self.context_stale = false;
        } else {
            self.context_stale = true;
        }
    }

    pub fn quota(&mut self, v: &Value, at: i64) {
        self.quota = serde_json::from_value(v.clone()).ok().map(|windows| (windows, at));
    }

    pub fn render(&self, model: Option<&str>, requested: Option<&str>, at: i64) -> String {
        // Only a model slug is displayed; protocol diagnostics and arbitrary strings cannot become markup.
        let safe = |s: &&str| !s.is_empty() && s.len() <= 128 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._:/-".contains(&b));
        let model = match model.filter(safe) {
            Some(m) => m.rsplit('/').next().unwrap_or(m).to_string(),
            None => requested.filter(safe).map(|m| format!("{m} (requested)")).unwrap_or_else(|| "? model unavailable".into()),
        };
        let stale = |yes, seen| if yes { format!(" · stale {} ({})", crate::time::age(at.saturating_sub(seen)), crate::time::exact(Some(seen))) } else { String::new() };
        let context = match self.context {
            Some((p, seen)) => {
                let face = match p { 0..=39 => "🙂", 40..=69 => "😐", 70..=79 => "😕", 80..=89 => "😣", _ => "🥵" };
                format!("{face} {p}%{}", stale(self.context_stale, seen))
            }
            None => "? context unavailable".into(),
        };
        let quota = self.quota.as_ref().filter(|(windows, _)| !windows.is_empty()).map(|(windows, _)| windows.iter().map(|w| {
            let label = w.window.as_deref().filter(|s| !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)));
            let label = label.map_or(String::new(), |s| format!("{s} "));
            let valid = (w.unknown.is_none() && !label.is_empty()).then_some(()).and(w.used).filter(|u| u.is_finite() && (0.0..=1.0).contains(u))
                .zip(w.resets_at.and_then(|n| i64::try_from(n).ok()).filter(|reset| *reset > at));
            match valid {
                Some((used, reset)) => {
                    let p = ((1.0 - used) * 100.0).round() as u64;
                    let moon = match p { 0..=12 => "🌑", 13..=37 => "🌘", 38..=62 => "🌗", 63..=87 => "🌖", _ => "🌕" };
                    let countdown = crate::time::countdown(reset.saturating_sub(at),false);
                    format!("{label}{moon} {p}% ↻ {countdown} ({})", crate::time::exact(Some(reset)))
                }
                None => format!("{label}? unavailable"),
            }
        }).collect::<Vec<_>>().join("; ")).unwrap_or_else(|| "? unavailable".into());
        format!("{model} · {context} · {quota}")
    }
}

// Runtime observations have numeric duration tokens; leave model slugs intact.
pub fn russian_units(text: &str) -> String {
    text.split_inclusive(char::is_whitespace).map(|part| {
        let token=part.trim_end();
        let suffix=token.chars().last().unwrap_or(' ');
        let number=&token[..token.len().saturating_sub(suffix.len_utf8())];
        let digits=number.strip_prefix('<').unwrap_or(number);
        let unit=match suffix {'m'=>"мин",'h'=>"ч",'d'=>"д",_=>return part.to_string()};
        if digits.is_empty() || !digits.bytes().all(|b|b.is_ascii_digit()) {return part.to_string();}
        format!("{number} {unit}{}", &part[token.len()..])
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // #383 A5: every quota window reaches /status; an incomplete window never reads healthy.
    #[test]
    fn all_quota_windows_383() {
        let mut f = Footer::default();
        f.quota(&json!([
            {"window":"300m","used":0.25,"resets_at":999,"event_id":null,"unknown":null},
            {"window":"10080m","used":0.5,"resets_at":999,"event_id":null,"unknown":null},
            {"window":"seven_day_sonnet","used":null,"resets_at":999,"event_id":null,"unknown":"partial-snapshot"},
            {"window":null,"used":0.01,"resets_at":999,"event_id":null,"unknown":null}
        ]), 0);
        let text = f.render(Some("model"), None, 0);
        assert!(text.contains("300m") && text.contains("75%"), "{text}");
        assert!(text.contains("10080m") && text.contains("50%"), "{text}");
        assert!(text.contains("seven_day_sonnet ? unavailable"), "{text}");
        assert!(!text.contains("99%"),"an unnamed window remains unknown: {text}");
    }

    // §12 / #119: guard every indicator boundary, reset rounding and bad data.
    // Invalid/expired readings must never masquerade as a real zero or current value.
    #[test]
    fn u119_indicators_and_invalid_observations() {
        let mut f = Footer::default();
        assert_eq!(f.render(None, None, 0), "? model unavailable · ? context unavailable · ? unavailable");
        for (p, face) in [(0,"🙂"),(39,"🙂"),(40,"😐"),(69,"😐"),(70,"😕"),(79,"😕"),(80,"😣"),(89,"😣"),(90,"🥵"),(101,"🥵")] {
            f.context(&json!({"used":p,"window":100}), 0);
            assert!(f.render(Some("provider/model"), None, 0).starts_with(&format!("model · {face} {p}%")));
        }
        for (p, moon) in [(0,"🌑"),(12,"🌑"),(13,"🌘"),(37,"🌘"),(38,"🌗"),(62,"🌗"),(63,"🌖"),(87,"🌖"),(88,"🌕"),(100,"🌕")] {
            for (reset,label) in [(1,"0:01"),(3599,"1:00"),(3600,"1:00"),(3601,"1:01"),(17999,"5:00"),(18000,"5h"),(18001,"6h"),(86399,"24h"),(86400,"1d"),(86401,"2d")] {
                f.quota(&json!([{"window":"10080m","used":(100-p) as f64 /100.0,"resets_at":reset,"event_id":null,"unknown":null}]), 0);
                assert!(f.render(Some("model"),None,0).contains(&format!("{moon} {p}% ↻ {label}")));
            }
        }
        f.context(&json!({"used":null,"window":0}),120);
        f.quota(&json!([{"window":null,"used":1.01,"resets_at":999999,"event_id":null,"unknown":null}]),120);
        assert!(f.render(None,Some("selected"),120).contains("stale 2m"));
        assert!(f.render(None,Some("selected"),120).starts_with("selected (requested)"));
        assert!(f.render(None,None,86402).ends_with("? unavailable"));
        let mut empty=Footer::default();
        for v in [json!({}),json!([]),json!([{"window":"300m","used":null,"resets_at":999,"event_id":null,"unknown":"partial-snapshot"}])] {
            empty.quota(&v,0);
            assert!(empty.render(None,None,0).ends_with("? unavailable"));
        }
    }
}
