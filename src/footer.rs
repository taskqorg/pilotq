//! #119 / §12: turn-local, numeric-only runtime observations. No account cache or secrets.
use serde_json::Value;

#[derive(Default)]
pub struct Footer {
    context: Option<(u64, i64)>,
    quota: Option<(u64, i64, i64)>,
    context_stale: bool,
    quota_stale: bool,
}

impl Footer {
    pub fn context(&mut self, v: &Value, at: i64) {
        let valid = v["last"]["inputTokens"].as_u64().zip(v["modelContextWindow"].as_u64())
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
        // Never substitute a model-specific or credit bucket for ordinary Codex quota.
        if v["limitId"].as_str().is_some_and(|s| s != "codex") { return; }
        let valid = [&v["primary"], &v["secondary"]].into_iter()
            .find(|w| w["windowDurationMins"].as_i64() == Some(10080))
            .and_then(|w| w["usedPercent"].as_u64().filter(|p| *p <= 100)
                .zip(w["resetsAt"].as_i64().filter(|t| *t > at)));
        if let Some((used, reset)) = valid {
            self.quota = Some((100 - used, reset, at));
            self.quota_stale = false;
        } else {
            self.quota_stale = true;
        }
    }

    pub fn render(&self, model: Option<&str>, requested: Option<&str>, at: i64) -> String {
        // Only a model slug is displayed; protocol diagnostics and arbitrary strings cannot become markup.
        let safe = |s: &&str| !s.is_empty() && s.len() <= 128 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._:/-".contains(&b));
        let model = match model.filter(safe) {
            Some(m) => m.rsplit('/').next().unwrap_or(m).to_string(),
            None => requested.filter(safe).map(|m| format!("{m} (requested)")).unwrap_or_else(|| "? model unavailable".into()),
        };
        let stale = |yes, seen| if yes { format!(" · stale {}m", at.saturating_sub(seen).max(0) / 60) } else { String::new() };
        let context = match self.context {
            Some((p, seen)) => {
                let face = match p { 0..=39 => "🙂", 40..=69 => "😐", 70..=79 => "😕", 80..=89 => "😣", _ => "🥵" };
                format!("{face} {p}%{}", stale(self.context_stale, seen))
            }
            None => "? context unavailable".into(),
        };
        let quota = match self.quota.filter(|(_, reset, _)| *reset > at) {
            Some((p, reset, seen)) => {
                let moon = match p { 0..=12 => "🌑", 13..=37 => "🌘", 38..=62 => "🌗", 63..=87 => "🌖", _ => "🌕" };
                let seconds = reset.saturating_sub(at) as u64;
                let reset = if seconds >= 86400 { format!("{}d", seconds.div_ceil(86400)) }
                    else if seconds >= 3600 { format!("{}h", seconds.div_ceil(3600)) } else { "<1h".into() };
                format!("{moon} {p}% ↻ {reset}{}", stale(self.quota_stale, seen))
            }
            None => "? unavailable".into(),
        };
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

    // §12 / #119: guard every indicator boundary, reset rounding and bad data.
    // Invalid/expired readings must never masquerade as a real zero or current value.
    #[test]
    fn u119_indicators_and_invalid_observations() {
        let mut f = Footer::default();
        assert_eq!(f.render(None, None, 0), "? model unavailable · ? context unavailable · ? unavailable");
        for (p, face) in [(0,"🙂"),(39,"🙂"),(40,"😐"),(69,"😐"),(70,"😕"),(79,"😕"),(80,"😣"),(89,"😣"),(90,"🥵"),(101,"🥵")] {
            f.context(&json!({"last":{"inputTokens":p},"modelContextWindow":100}), 0);
            assert!(f.render(Some("provider/model"), None, 0).starts_with(&format!("model · {face} {p}%")));
        }
        for (p, moon) in [(0,"🌑"),(12,"🌑"),(13,"🌘"),(37,"🌘"),(38,"🌗"),(62,"🌗"),(63,"🌖"),(87,"🌖"),(88,"🌕"),(100,"🌕")] {
            for (reset,label) in [(3599,"<1h"),(3600,"1h"),(3601,"2h"),(86399,"24h"),(86400,"1d"),(86401,"2d")] {
                f.quota(&json!({"limitId":"codex","primary":{"usedPercent":100-p,"windowDurationMins":10080,"resetsAt":reset}}), 0);
                assert!(f.render(Some("model"),None,0).contains(&format!("{moon} {p}% ↻ {label}")));
            }
        }
        f.context(&json!({"last":{"inputTokens":-1},"modelContextWindow":0}),120);
        f.quota(&json!({"secondary":{"usedPercent":101,"windowDurationMins":10080,"resetsAt":999999}}),120);
        assert!(f.render(None,Some("selected"),120).contains("stale 2m"));
        assert!(f.render(None,Some("selected"),120).starts_with("selected (requested)"));
        assert!(f.render(None,None,86402).ends_with("? unavailable"));
        let mut empty=Footer::default();
        for v in [json!({}),json!({"primary":{"usedPercent":0,"windowDurationMins":300,"resetsAt":999}}),json!({"limitId":"other","secondary":{"usedPercent":0,"windowDurationMins":10080,"resetsAt":999}})] {
            empty.quota(&v,0);
            assert!(empty.render(None,None,0).ends_with("? unavailable"));
        }
    }
}
