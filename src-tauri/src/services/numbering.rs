use crate::{
    db::DatabaseState,
    errors::AppError,
    models::settings::{
        DocumentTypeKey, IssuedNumber, NumberPreview, NumberResetPolicy, NumberingSeriesSettings,
    },
};
use rusqlite::{Connection, OptionalExtension, Transaction};

pub fn default_series() -> NumberingSeriesSettings {
    NumberingSeriesSettings {
        document_type: DocumentTypeKey::invoice(),
        prefix: String::new(),
        format: "{counter}/{year}".into(),
        reset_policy: Some(NumberResetPolicy::Yearly),
        next_counter: 1,
    }
}
fn invalid(field: &str, message: &str) -> AppError {
    AppError::new("validation_error", "Sprawdź ustawienia numeracji.")
        .with_field_errors(serde_json::json!({field:message}))
}
fn supported(key: &DocumentTypeKey) -> Result<(), AppError> {
    if key.as_str() != "invoice" {
        return Err(invalid(
            "document_type",
            "Ten typ dokumentu nie jest obsługiwany.",
        ));
    }
    Ok(())
}
fn tokens(format: &str) -> Result<Vec<&str>, AppError> {
    let mut rest = format;
    let mut result = Vec::new();
    while !rest.is_empty() {
        let index = rest.find(['{', '}']);
        let Some(i) = index else { break };
        rest = &rest[i..];
        if !rest.starts_with('{') {
            return Err(invalid("format", "Nieprawidłowe nawiasy we wzorze."));
        }
        let end = rest
            .find('}')
            .ok_or_else(|| invalid("format", "Niezamknięty token."))?;
        let token = &rest[1..end];
        if !["prefix", "year", "month", "counter"].contains(&token) {
            return Err(invalid("format", "Nieznany token wzoru."));
        }
        result.push(token);
        rest = &rest[end + 1..];
    }
    Ok(result)
}
pub fn validate_series(s: &NumberingSeriesSettings) -> Result<(), AppError> {
    if s.format.is_empty() || s.format.len() > 200 {
        return Err(invalid("format", "Wzór musi mieć od 1 do 200 znaków."));
    }
    if s.prefix.len() > 50
        || s.prefix.contains(['{', '}'])
        || s.prefix.chars().any(char::is_control)
        || s.format.chars().any(char::is_control)
    {
        return Err(invalid(
            "prefix",
            "Prefiks do 50 znaków, bez nawiasów i znaków sterujących.",
        ));
    }
    let t = tokens(&s.format)?;
    if !t.contains(&"counter") {
        return Err(invalid("format", "Wzór musi zawierać {counter}."));
    }
    let policy = s.reset_policy.unwrap_or(NumberResetPolicy::Never);
    if policy != NumberResetPolicy::Never && !t.contains(&"year") {
        return Err(invalid("format", "Reset wymaga tokenu {year}."));
    }
    if policy == NumberResetPolicy::Monthly && !t.contains(&"month") {
        return Err(invalid(
            "format",
            "Reset miesięczny wymaga {year} i {month}.",
        ));
    }
    if s.next_counter < 1 || s.next_counter >= i64::MAX as u64 {
        return Err(invalid(
            "next_counter",
            "Licznik musi być dodatnią liczbą całkowitą w obsługiwanym zakresie.",
        ));
    }
    Ok(())
}
pub(crate) fn read_series(
    c: &Connection,
    key: &DocumentTypeKey,
) -> Result<(NumberingSeriesSettings, Option<String>), AppError> {
    let row=c.query_row("SELECT prefix,format,reset_policy,next_counter,counter_period FROM numbering_series WHERE document_type=?1",[key.as_str()],|r| {
        Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)? as u64,r.get::<_,Option<String>>(4)?))
    }).optional()?;
    match row {
        Some((prefix, format, policy, next_counter, period)) => {
            let reset_policy = serde_json::from_value(serde_json::Value::String(policy))
                .map_err(|_| invalid("reset_policy", "Nieprawidłowa polityka resetu."))?;
            Ok((
                NumberingSeriesSettings {
                    document_type: key.clone(),
                    prefix,
                    format,
                    reset_policy: Some(reset_policy),
                    next_counter,
                },
                period,
            ))
        }
        None => {
            let mut s = default_series();
            s.document_type = key.clone();
            Ok((s, None))
        }
    }
}
pub fn get_numbering_settings(
    state: &DatabaseState,
) -> Result<Vec<NumberingSeriesSettings>, AppError> {
    state.with_connection(|c| Ok(vec![read_series(c, &DocumentTypeKey::invoice())?.0]))
}
pub fn save_numbering_settings(
    state: &DatabaseState,
    input: NumberingSeriesSettings,
) -> Result<NumberingSeriesSettings, AppError> {
    supported(&input.document_type)?;
    validate_series(&input)?;
    state.with_connection(|c| {let tx=c.transaction()?;
        let (previous,period)=read_series(&tx,&input.document_type)?;
        let period=if previous.reset_policy==input.reset_policy && previous.next_counter==input.next_counter {period} else {None};
        let policy=serde_json::to_value(input.reset_policy.unwrap_or(NumberResetPolicy::Never)).unwrap();
        tx.execute("INSERT INTO numbering_series (document_type,prefix,format,reset_policy,next_counter,counter_period) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(document_type) DO UPDATE SET prefix=excluded.prefix,format=excluded.format,reset_policy=excluded.reset_policy,next_counter=excluded.next_counter,counter_period=excluded.counter_period",
            rusqlite::params![input.document_type.as_str(),input.prefix,input.format,policy.as_str(),input.next_counter as i64,period])?;
        tx.commit()?;Ok(input)
    })
}
fn candidate(
    c: &Connection,
    key: &DocumentTypeKey,
    year: u16,
    month: u8,
) -> Result<NumberPreview, AppError> {
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
        return Err(invalid("issue_date", "Nieprawidłowy rok lub miesiąc."));
    }
    let (s, last_period) = read_series(c, key)?;
    validate_series(&s)?;
    let period = match s.reset_policy.unwrap_or(NumberResetPolicy::Never) {
        NumberResetPolicy::Yearly => format!("{year:04}"),
        NumberResetPolicy::Monthly => format!("{year:04}-{month:02}"),
        NumberResetPolicy::Never => "never".into(),
    };
    let mut counter = if last_period.as_ref().is_some_and(|p| p != &period) {
        1
    } else {
        s.next_counter
    };
    loop {
        if counter >= i64::MAX as u64 {
            return Err(invalid("next_counter", "Wyczerpano zakres licznika."));
        }
        // Render parsed tokens once; text substituted for a prefix can never become a token.
        let formatted_number = s
            .format
            .replace("{year}", &format!("{year:04}"))
            .replace("{month}", &format!("{month:02}"))
            .replace("{counter}", &counter.to_string())
            .replace("{prefix}", &s.prefix);
        let used:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM issued_numbers WHERE document_type=?1 AND formatted_number=?2)",rusqlite::params![key.as_str(),formatted_number],|r|r.get(0))?;
        if !used {
            return Ok(NumberPreview {
                document_type: key.clone(),
                formatted_number,
                period_key: period,
                counter,
            });
        }
        counter += 1;
    }
}
pub fn preview_next_number(
    state: &DatabaseState,
    key: DocumentTypeKey,
) -> Result<NumberPreview, AppError> {
    supported(&key)?;
    state.with_connection(|c| {let tx=c.transaction()?;
        let (year,month)=tx.query_row("SELECT CAST(strftime('%Y','now','localtime') AS INTEGER), CAST(strftime('%m','now','localtime') AS INTEGER)",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
        candidate(&tx,&key,year,month)
    })
}
/// Caller must persist the document and its immutable snapshot in this same transaction.
pub fn allocate_number_in_transaction(
    tx: &Transaction<'_>,
    key: &DocumentTypeKey,
    year: u16,
    month: u8,
) -> Result<IssuedNumber, AppError> {
    let n = candidate(tx, key, year, month)?;
    let (s, _) = read_series(tx, key)?;
    tx.execute(
        "INSERT INTO issued_numbers(document_type,formatted_number) VALUES (?1,?2)",
        rusqlite::params![key.as_str(), n.formatted_number],
    )?;
    let policy = serde_json::to_value(s.reset_policy.unwrap_or(NumberResetPolicy::Never)).unwrap();
    tx.execute("INSERT INTO numbering_series(document_type,prefix,format,reset_policy,counter_period,next_counter) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(document_type) DO UPDATE SET counter_period=excluded.counter_period,next_counter=excluded.next_counter",
        rusqlite::params![key.as_str(),s.prefix,s.format,policy.as_str(),n.period_key,(n.counter+1) as i64])?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseState;
    use std::sync::Arc;

    fn database() -> (tempfile::TempDir, DatabaseState) {
        let dir = tempfile::tempdir().unwrap();
        let state = DatabaseState::open(dir.path().into()).unwrap();
        (dir, state)
    }
    fn series() -> NumberingSeriesSettings {
        NumberingSeriesSettings {
            document_type: DocumentTypeKey::invoice(),
            prefix: "FV".into(),
            format: "{prefix}/{counter}/{year}/{month}".into(),
            reset_policy: Some(NumberResetPolicy::Monthly),
            next_counter: 1,
        }
    }
    fn issue(state: &DatabaseState, year: u16, month: u8) -> IssuedNumber {
        state
            .with_connection(|c| {
                let tx = c.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
                let number =
                    allocate_number_in_transaction(&tx, &DocumentTypeKey::invoice(), year, month)?;
                tx.commit()?;
                Ok(number)
            })
            .unwrap()
    }
    #[test]
    fn renders_all_four_tokens() {
        let (_d, s) = database();
        save_numbering_settings(&s, series()).unwrap();
        assert_eq!(issue(&s, 2026, 11).formatted_number, "FV/1/2026/11");
    }
    #[test]
    fn rejects_unknown_or_unclosed_token() {
        for format in ["{counter}/{oops}", "{counter", "counter}", "", "FV"] {
            let mut input = series();
            input.format = format.into();
            assert!(validate_series(&input).is_err(), "{format}");
        }
    }
    #[test]
    fn yearly_reset_requires_year_token() {
        let mut x = series();
        x.reset_policy = Some(NumberResetPolicy::Yearly);
        x.format = "{counter}".into();
        assert!(validate_series(&x).is_err());
    }
    #[test]
    fn monthly_reset_requires_year_and_month() {
        for format in ["{counter}/{year}", "{counter}/{month}"] {
            let mut x = series();
            x.format = format.into();
            assert!(validate_series(&x).is_err());
        }
    }
    #[test]
    fn series_are_independent_by_document_type() {
        let (_d, s) = database();
        save_numbering_settings(&s, series()).unwrap();
        s.with_connection(|c| { c.execute("INSERT INTO numbering_series (document_type,prefix,format,reset_policy,next_counter) VALUES ('quote','Q','{prefix}/{counter}','never',7)",[])?;
            let tx=c.transaction()?; let q=allocate_number_in_transaction(&tx,&DocumentTypeKey::new("quote"),2026,11)?;
            assert_eq!(q.formatted_number,"Q/7"); tx.commit()?; Ok(()) }).unwrap();
        assert_eq!(issue(&s, 2026, 11).counter, 1);
    }
    #[test]
    fn preview_does_not_change_counter() {
        let (_d, s) = database();
        save_numbering_settings(&s, series()).unwrap();
        let before = fs_snapshot(&s);
        let a = preview_next_number(&s, DocumentTypeKey::invoice()).unwrap();
        assert_eq!(
            a,
            preview_next_number(&s, DocumentTypeKey::invoice()).unwrap()
        );
        assert_eq!(before, fs_snapshot(&s));
    }
    fn fs_snapshot(s: &DatabaseState) -> Vec<u8> {
        std::fs::read(s.data_dir().join("app.sqlite")).unwrap()
    }
    #[test]
    fn format_change_skips_historical_collision() {
        let (_d, s) = database();
        let mut x = series();
        x.reset_policy = Some(NumberResetPolicy::Never);
        x.format = "{prefix}/{counter}".into();
        save_numbering_settings(&s, x.clone()).unwrap();
        assert_eq!(issue(&s, 2026, 11).counter, 1);
        x.prefix = "OTHER".into();
        save_numbering_settings(&s, x.clone()).unwrap();
        issue(&s, 2026, 11);
        x.prefix = "FV".into();
        x.next_counter = 1;
        save_numbering_settings(&s, x).unwrap();
        assert_eq!(issue(&s, 2026, 11).counter, 2);
    }
    #[test]
    fn concurrent_allocations_commit_unique_numbers() {
        let (d, s) = database();
        save_numbering_settings(&s, series()).unwrap();
        let root = Arc::new(d.path().to_path_buf());
        let threads: Vec<_> = (0..6)
            .map(|_| {
                let root = root.clone();
                std::thread::spawn(move || {
                    let db = DatabaseState::open((*root).clone()).unwrap();
                    issue(&db, 2026, 11).counter
                })
            })
            .collect();
        let mut numbers: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        numbers.sort();
        assert_eq!(numbers, vec![1, 2, 3, 4, 5, 6]);
    }
    #[test]
    fn transaction_rollback_keeps_number_unissued() {
        let (_d, s) = database();
        save_numbering_settings(&s, series()).unwrap();
        s.with_connection(|c| {
            let tx = c.transaction()?;
            allocate_number_in_transaction(&tx, &DocumentTypeKey::invoice(), 2026, 11)?;
            Ok(())
        })
        .unwrap();
        assert_eq!(issue(&s, 2026, 11).counter, 1);
    }
    #[test]
    fn reset_boundaries_and_never_reset() {
        let (_d, s) = database();
        save_numbering_settings(&s, series()).unwrap();
        assert_eq!(issue(&s, 2026, 11).counter, 1);
        assert_eq!(issue(&s, 2026, 11).counter, 2);
        assert_eq!(issue(&s, 2026, 12).counter, 1);
        let mut x = series();
        x.reset_policy = Some(NumberResetPolicy::Yearly);
        save_numbering_settings(&s, x).unwrap();
        issue(&s, 2026, 12);
        assert_eq!(issue(&s, 2027, 1).counter, 1);
    }
    #[test]
    fn invalid_dates_counters_and_unsupported_commands_are_rejected() {
        let (_d, s) = database();
        let mut x = series();
        x.next_counter = 0;
        assert!(save_numbering_settings(&s, x).is_err());
        let mut x = series();
        x.prefix = "{counter}".into();
        assert!(save_numbering_settings(&s, x).is_err());
        assert!(preview_next_number(&s, DocumentTypeKey::new("vat_invoice")).is_err());
        for (y, m) in [(0, 1), (10000, 1), (2026, 0), (2026, 13)] {
            s.with_connection(|c| {
                let tx = c.transaction()?;
                assert!(
                    allocate_number_in_transaction(&tx, &DocumentTypeKey::invoice(), y, m).is_err()
                );
                Ok(())
            })
            .unwrap();
        }
    }
    #[test]
    fn explicitly_changed_counter_survives_a_reset_boundary() {
        let (_d, s) = database();
        save_numbering_settings(&s, series()).unwrap();
        issue(&s, 2026, 11);
        s.with_connection(|c| {
            c.execute("UPDATE numbering_series SET counter_period='1900-01'", [])?;
            Ok(())
        })
        .unwrap();
        let mut x = series();
        x.next_counter = 50;
        save_numbering_settings(&s, x).unwrap();
        assert_eq!(
            preview_next_number(&s, DocumentTypeKey::invoice())
                .unwrap()
                .counter,
            50
        );
    }
}
