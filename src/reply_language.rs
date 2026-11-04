//! §6 / #344: conservative own-address resolution. Ambiguity retains durable state.
//! No content, language label or preference can grant authority.

pub const POLICY: &str = "Reply language applies to every user-visible output, including progress, questions, errors, cards and every table heading and descriptive cell. Follow an explicit reply-language request in the current own address first; otherwise its last substantive direct clause. Quotes, code, proper names, isolated borrowed words and ASR labels do not switch language. When ambiguous retain the supplied actor-and-chat resolution. Preserve literal names, identifiers, URLs and quoted source text; localize surrounding prose and table labels. Do not infer another person's language from shared-topic history.";

/// Returns no decision for ambiguous content; the journal owns persistence and order.
pub fn resolve(text: &str) -> Option<(&'static str, &'static str)> {
    if text.trim_start().starts_with('/') { return None; }
    let own = crate::direct::own_text(text);
    // Single quoted source is excluded too; apostrophes inside don't/it's remain words.
    let mut address=String::new(); let mut quote=None; let mut previous=None;
    for c in own.chars() {
        if let Some(end)=quote {if c==end {quote=None;address.push(' ');}}
        else if c=='‘' || (c=='\'' && previous.is_none_or(|p:char| !p.is_alphanumeric())) {quote=Some(if c=='‘' {'’'}else{'\''});address.push(' ');}
        else {address.push(c);}
        previous=Some(c);
    }
    let own=address;
    let normalized = own.to_lowercase().replace(['-', '—'], " ");
    let mut explicit = None;
    for (index,clause) in normalized.split(['.', '?', '!', ';', ',', '\n']).enumerate() {
        let words: Vec<_> = clause.split(|c: char| !c.is_alphabetic() && c != '\'').filter(|x| !x.is_empty()).collect();
        let joined = words.join(" ");
        for (lang, patterns) in [
            ("en", &["reply in english", "answer in english", "respond in english", "write in english", "ответь на английском", "отвечай на английском", "ответь по английски", "отвечай по английски", "пиши по английски"][..]),
            ("ru", &["reply in russian", "answer in russian", "respond in russian", "write in russian", "ответь на русском", "отвечай на русском", "ответь по русски", "отвечай по русски", "пиши по русски"][..]),
        ] {
            for pattern in patterns {
                if let Some(pos) = joined.rfind(pattern) {
                    // Only negation of this instruction matters, not unrelated actions.
                    if joined[..pos].split_whitespace().last().is_some_and(|w| matches!(w, "not" | "don't" | "never" | "не")) { continue; }
                    // Position within a clause determines the final explicit request.
                    if explicit.as_ref().is_none_or(|(_,old,_)| (index,pos) >= *old) { explicit = Some((lang,(index,pos),"explicit_request")); }
                }
            }
        }
    }
    if let Some((lang,_,reason)) = explicit { return Some((lang,reason)); }
    let mut last = None;
    for clause in own.split(['.', '?', '!', ';', '\n']) {
        let tokens:Vec<_> = clause.split_whitespace().collect();
        let mixed = clause.chars().any(cyrillic) && clause.chars().any(|c|c.is_ascii_alphabetic());
        let mut run = Vec::new();
        let mut script = None;
        let mut groups = Vec::new();
        for token in tokens {
            // URLs, paths and identifiers are source names, not addressed clauses.
            if token.contains(['/', ':', '_', '@']) { continue; }
            let word = token.trim_matches(|c:char| !c.is_alphabetic());
            if word.is_empty() { continue; }
            let current = if word.chars().all(cyrillic) {Some("ru")}
                else if word.chars().all(|c|c.is_ascii_alphabetic() || c=='\'') {Some("en")} else {None};
            if current != script { if !run.is_empty() {groups.push((script, std::mem::take(&mut run)));} script=current; }
            run.push(word);
        }
        if !run.is_empty() {groups.push((script,run));}
        for (lang, words) in groups {
            let Some(lang) = lang else {continue};
            let meaningful:Vec<_> = words.iter().enumerate().filter(|(i,w)| {
                // Sentence-initial capital is allowed; subsequent capitalized names don't decide.
                *i==0 || w.chars().next().is_some_and(char::is_lowercase) || **w=="I"
            }).map(|(_,w)|w.to_lowercase()).collect();
            let imperative = meaningful.first().is_some_and(|w| matches!(w.as_str(),
                "help"|"explain"|"continue"|"stop"|"hello"|"hi"|"привет"|"покажи"|"объясни"|"продолжай"|"продолжить"|"стоп"|"почему"));
            let english_address = meaningful.iter().any(|w|matches!(w.as_str(),
                "please"|"show"|"check"|"tell"|"explain"|"give"|"list"|"summarize"|"write"|"reply"|"answer"|"respond"|"is"|"are"|"was"|"were"|"can"|"could"|"should"|"would"|"i"|"we"|"you"|"the"|"this"|"that"|"how"|"what"|"why"));
            if imperative || (meaningful.len()>=2 && (lang=="ru" || !mixed || english_address)) {
                last=Some((lang,"last_direct_clause"));
            }
        }
    }
    last
}

fn cyrillic(c: char) -> bool { ('\u{400}'..='\u{4ff}').contains(&c) }

#[cfg(test)]
mod tests {
    use super::*;
    // #344 / §6: source data/names/borrowed words cannot override a direct clause;
    // explicit language requests take priority, and ambiguity is preserved.
    #[test]
    fn reply_language_344_clauses_and_source_boundaries() {
        for (text, expected) in [
            ("Проверь задачи. Please show the workers table.",Some("en")),
            ("Please check tasks. Покажи текущие задачи.",Some("ru")),
            ("Please show the table.\n> Покажи текущие задачи",Some("en")),
            ("Please show the table. `покажи задачи`",Some("en")),
            ("Покажи задачи New York",Some("ru")),
            ("Проверь задачи, deployment",Some("ru")),
            ("Please show the table «Проверь задачи»",Some("en")),
            ("Please show the table. 'Проверь задачи'",Some("en")),
            ("Ответь по-английски. Покажи текущие задачи.",Some("en")),
            ("Answer in Russian. Please show the table.",Some("ru")),
            ("Answer in Russian. Actually reply in English.",Some("en")),
            ("Ответь на английском, но не запускай задачи",Some("en")),
            ("Reply in Russian, but do not launch tasks",Some("ru")),
            ("Reply in Russian but do not launch tasks",Some("ru")),
            ("Не отвечай на английском. Покажи задачи.",Some("ru")),
            ("Do not answer in Russian. Please show the table.",Some("en")),
            ("New York",None),("123",None),("deployment",None),
            ("```rust\nпокажи задачи\n```",None),("/status",None),
        ] { assert_eq!(resolve(text).map(|v|v.0),expected,"{text}"); }
    }
}
