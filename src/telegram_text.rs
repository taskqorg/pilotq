// Shared delivery rendering; the A40 fake audit uses the same exact wire content.
pub(crate) fn tidy(text: &str) -> String {
    fn label(u: &url::Url) -> String {
        u.host_str().unwrap_or("source").trim_start_matches("www.").to_string()
    }

    fn word(w: &str) -> String {
        let core = w.trim_start_matches(['(', '"', '*']);
        let start = w.len() - core.len();
        let core = core.trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '"', '*']);
        let (head, tail) = (&w[..start], &w[start + core.len()..]);
        if let Some(u) = url::Url::parse(core).ok().filter(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some()) {
            return format!("{head}[{}]({core}){tail}", label(&u));
        }
        let file = core.split(':').next().unwrap_or(core);
        // Dotted filenames are not limited to known extensions. Leave versions and abbreviations such as e.g. alone.
        let dotted = file.rsplit_once('.').is_some_and(|(_, ext)| ext.chars().any(char::is_alphabetic))
            && !matches!(file, "e.g" | "i.e");
        // A repository identity is not a local path; slash controls have no directory.
        let rooted = file.split_once('/').is_some_and(|(dir, name)|
            !dir.is_empty() && dir != "taskqorg" && !name.is_empty());
        let path = (dotted || rooted)
            && core.chars().all(|c| c.is_alphanumeric() || " _-./:".contains(c));
        if path { format!("{head}`{core}`{tail}") } else { w.to_string() }
    }
    let mut out = String::new();
    let mut fence = false;
    for line in text.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") { fence = !fence; }
        if fence || t.starts_with("```") || t.starts_with("~~~") || line.matches('`').count() % 2 == 1 { out += line; continue; }
        let (body, end) = line.strip_suffix('\n').map_or((line, ""), |b| (b, "\n"));
        let mut label_open = false;
        for (k, part) in body.split('`').enumerate() {
            if k > 0 { out.push('`'); }
            if k % 2 == 1 { out += part; continue; }
            let words: Vec<_> = part.split(' ').collect();
            let mut i = 0;
            while i < words.len() {
                if i > 0 { out.push(' '); }
                let w = words[i];
                // An unquoted path with spaces ends at its filename. Do not join prose after
                // an already complete filename, or consume written links/code as path text.
                let mut last = i;
                if !label_open && !w.contains(['[', ']']) && w.contains('/') && !w.contains("://")
                    && !w.rsplit('/').next().unwrap_or(w).contains('.') {
                    for (j, next) in words.iter().enumerate().skip(i + 1) {
                        if next.is_empty() || matches!(*next, "and" | "or" | "then" | "for" | "in" | "with" | "see") || next.contains(['[', ']', '/', ':', '!', '?']) { break; }
                        if next.trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '"', '*']).rsplit_once('.').is_some_and(|(_, ext)| ext.chars().any(char::is_alphabetic)) {
                            last = j;
                            break;
                        }
                    }
                }
                let candidate = words[i..=last].join(" ");
                // A word inside or next to a written link keeps its source.
                if label_open || w.contains(['[', ']']) { out += w; } else { out += &word(&candidate); }
                label_open = (label_open || w.contains('[')) && !w.contains(']');
                i = last + 1;
            }
        }
        out += end;
    }
    out
}

/// A small safe Markdown subset for Telegram HTML. Raw HTML always stays literal; unsupported Markdown stays readable.
/// ponytail: headings, bullets, bold, italic, escapes, code and HTTP(S) links; a full Markdown parser only when the subset needs to grow.
pub(crate) fn html(text: &str) -> String {
    // #222: recognized speech is literal, collapsed in HTML, quoted in plain fallback.
    if text.starts_with("> ") {
        let (quote,answer)=text.split_once("\n\n").unwrap_or((text,""));
        if quote.lines().all(|line|line.starts_with("> ")) {
            let speech=quote.lines().map(|line|&line[2..]).collect::<Vec<_>>().join("\n");
            let speech=speech.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;");
            return format!("<blockquote expandable>{speech}</blockquote>{}{}",if answer.is_empty(){""}else{"\n\n"},html(answer));
        }
    }
    let text = &tidy(text);
    fn escape(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;") }
    fn inline(mut s: &str) -> String {
        let mut out = String::new();
        while !s.is_empty() {
            // a Markdown escape shows its punctuation, not the backslash
            if let Some(c) = s.strip_prefix('\\').and_then(|r| r.chars().next()).filter(char::is_ascii_punctuation) {
                out += &escape(&c.to_string());
                s = &s[2..];
                continue;
            }
            let pair = if s.starts_with("**") { Some(("**", "b")) } else if s.starts_with("__") { Some(("__", "b")) }
                else if s.starts_with('`') { Some(("`", "code")) } else if s.starts_with('*') { Some(("*", "i")) } else { None };
            if let Some((mark, tag)) = pair {
                // emphasis needs text touching both marks, so "2 * 3 * 4" stays literal
                if let Some(end) = s[mark.len()..].find(mark).filter(|&e| tag == "code" || e > 0 && !s[mark.len()..].starts_with(' ') && !s[..mark.len()+e].ends_with(' ')) {
                    let end = end + mark.len();
                    let inner = &s[mark.len()..end];
                    out += &format!("<{tag}>{}</{tag}>", if tag == "code" { escape(inner) } else { inline(inner) });
                    s = &s[end+mark.len()..];
                    continue;
                }
            }
            if s.starts_with('[') {
                if let Some(mid) = s.find("](") {
                    if let Some(end) = s[mid+2..].find(')') {
                        let end = mid+2+end;
                        let url = &s[mid+2..end];
                        if url::Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "http" | "https")) {
                            out += &format!("<a href=\"{}\">{}</a>", escape(url), escape(&s[1..mid]));
                            s = &s[end+1..];
                            continue;
                        }
                    }
                }
            }
            let c = s.chars().next().unwrap();
            out += &escape(&c.to_string());
            s = &s[c.len_utf8()..];
        }
        out
    }
    let mut out = String::new();
    let mut code = false;
    for line in text.split_inclusive('\n') {
        if line.starts_with("```") {
            if code { out += "</pre>"; } else { out += "<pre>"; }
            code = !code;
            continue;
        }
        if code { out += &escape(line); continue; }
        let (body, end) = line.strip_suffix('\n').map_or((line, ""), |b| (b, "\n"));
        let heading = body.trim_start_matches('#');
        let item = body.trim_start();
        let rule = body.trim();
        if heading.len() < body.len() && heading.starts_with(' ') {
            out += &format!("<b>{}</b>{end}", inline(heading.trim()));
        } else if rule.len() >= 3 && ['-', '*', '_'].iter().any(|&m| rule.chars().all(|c| c == m)) {
            // a horizontal rule is decoration; keep only its line break
            out += end;
        } else if let Some(rest) = ["- ", "* ", "+ "].iter().find_map(|m| item.strip_prefix(m)) {
            out += &format!("{}• {}{end}", &body[..body.len()-item.len()], inline(rest));
        } else { out += &inline(line); }
    }
    if code { out += "</pre>"; }
    out
}

/// Native rich Markdown also accepts HTML. Escape raw tags/entities outside code, once at the output boundary (§4).
/// A single line break inside a paragraph is a Markdown soft break that renders as a space; it becomes a hard break (#82).
pub(crate) fn markdown(text: &str) -> String {
    // Generated escaped HTML preserves literal speech and the expandable attribute in rich Markdown.
    if text.starts_with("> 🎤 ") {return html(text);}
    // a line that starts its own block never needs a hard break before it
    fn block(l: &str) -> bool {
        if l == "\u{a0}\n" { return false; } // #141: the footer spacer needs hard breaks on both sides
        let l=l.trim_start();
        let digits=l.chars().take_while(char::is_ascii_digit).count();
        l.trim().is_empty() || ["- ","* ","+ ","#",">","|","```","~~~"].iter().any(|m|l.starts_with(m))
            || (digits>0 && matches!(l[digits..].chars().next(),Some('.'|')')))
    }
    let text=&tidy(text);
    let mut out=String::new();
    let mut fence: Option<(char,usize)>=None;
    let mut inline=0;
    let lines: Vec<&str>=text.split_inclusive('\n').collect();
    for (i,line) in lines.iter().copied().enumerate() {
        let trimmed=line.trim_start();
        let mark=trimmed.chars().next().unwrap_or(' ');
        let count=trimmed.chars().take_while(|c|*c==mark).count();
        let indent=line.len()-trimmed.len();
        if matches!(mark,'`'|'~') && count>=3 && inline==0 && indent<=3 {
            if let Some((f,n))=fence {
                if mark==f && count>=n && trimmed[count..].trim().is_empty() { fence=None; }
            } else if mark!='`' || !trimmed[count..].contains('`') { fence=Some((mark,count)); }
            if fence.is_some() || trimmed[count..].trim().is_empty() { out.push_str(line); continue; }
        }
        if fence.is_some() { out.push_str(line); continue; }
        let mut chars=line.chars().peekable();
        while let Some(c)=chars.next() {
            if c=='\\' && inline==0 {
                out.push(c);
                if let Some(next)=chars.next() { out.push(next); }
            } else if c=='`' {
                let mut n=1;
                while chars.peek()==Some(&'`') { chars.next(); n+=1; }
                if inline==n { inline=0; } else if inline==0 {
                    let mut rest=chars.clone();
                    while let Some(c)=rest.next() {
                        if c=='`' {
                            let mut closing=1;
                            while rest.peek()==Some(&'`') { rest.next(); closing+=1; }
                            if closing==n { inline=n; break; }
                        }
                    }
                }
                out.extend(std::iter::repeat_n('`',n));
            } else if inline>0 { out.push(c); }
            else { match c { '<'=>out.push_str("&lt;"),'&'=>out.push_str("&amp;"),_=>out.push(c) } }
        }
        let body=line.trim_end_matches('\n');
        if inline==0 && line.ends_with('\n') && !trimmed.starts_with('#') && (!body.trim().is_empty() || body == "\u{a0}") && !body.ends_with([' ','\\'])
            && lines.get(i+1).is_some_and(|next|!block(next)) {
            out.pop();
            out.push_str("  \n");
        }
    }
    out
}

