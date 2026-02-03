use anyhow::Result;
use chrono::{Datelike, Duration, Local, NaiveDate};
use colored::Colorize;
use std::fmt::Display;

#[derive(Default, Debug)]
pub struct Deadline {
    date: NaiveDate,
}

impl Deadline {
    pub fn days_until(&self) -> String {
        let days_until = (self.date - Local::now().date_naive()).num_days();
        if days_until < 0 {
            format!("{} days ago", -days_until).red().to_string()
        } else {
            let mut res = format!("in {} days", days_until).to_string();
            if res == "in 0 days" {
                res = res.red().to_string()
            }
            res
        }
    }

    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim().to_lowercase();
        let today = Local::now().date_naive();

        let deadline = match input.as_str() {
            "today" => today,
            "tomorrow" | "tmr" => today + Duration::days(1),

            "monday" | "mon" => next_weekday(today, chrono::Weekday::Mon),
            "tuesday" | "tue" => next_weekday(today, chrono::Weekday::Tue),
            "wednesday" | "wed" => next_weekday(today, chrono::Weekday::Wed),
            "thursday" | "thu" => next_weekday(today, chrono::Weekday::Thu),
            "friday" | "fri" => next_weekday(today, chrono::Weekday::Fri),
            "saturday" | "sat" => next_weekday(today, chrono::Weekday::Sat),
            "sunday" | "sun" => next_weekday(today, chrono::Weekday::Sun),

            "week" | "1week" | "1w" => today + Duration::weeks(1),
            "2weeks" | "2w" => today + Duration::weeks(2),
            "month" | "1month" | "1m" => today + Duration::days(30),
            "3months" | "3m" => today + Duration::days(90),

            "eow" | "endofweek" => end_of_week(today),
            "eom" | "endofmonth" => end_of_month(today),
            "eoy" | "endofyear" => end_of_year(today),

            _ => {
                let rel_duration = parse_relative_duration(&input);
                match rel_duration {
                    Some(days) => today + Duration::days(days),
                    None => NaiveDate::parse_from_str(&input, "%Y-%m-%d")
                        .or_else(|_| NaiveDate::parse_from_str(&input, "%d/%m/%Y"))
                        .or_else(|_| NaiveDate::parse_from_str(&input, "%m-%d-%Y"))
                        .map_err(|_| {
                            anyhow::anyhow!(
                                "Invalid deadline format. Use: today, tomorrow, monday, +5d, 2026-02-10, etc. See `todo add --help` for more information."
                                )
                        })?,
                }
            }
        };
        Ok(Self { date: deadline })
    }
}

impl Display for Deadline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.date.format("%Y-%m-%d"))
    }
}

fn next_weekday(from: NaiveDate, target: chrono::Weekday) -> NaiveDate {
    let days_ahead =
        (target.num_days_from_monday() as i64 - from.weekday().num_days_from_monday() as i64 + 7)
            % 7;

    if days_ahead == 0 {
        from + Duration::days(7)
    } else {
        from + Duration::days(days_ahead)
    }
}

fn end_of_week(date: NaiveDate) -> NaiveDate {
    let days_until_sunday = (6 - date.weekday().num_days_from_monday()) as i64;
    date + Duration::days(days_until_sunday)
}

fn end_of_month(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(
        date.year(),
        date.month(),
        days_in_month(date.year(), date.month()),
    )
    .unwrap()
}

fn end_of_year(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), 12, 31).unwrap()
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or(NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
        .pred_opt()
        .unwrap()
        .day()
}

fn parse_relative_duration(input: &str) -> Option<i64> {
    let cleaned = input.replace("in ", "").replace("+", "").trim().to_string();

    if let Some(num_str) = cleaned
        .strip_suffix('d')
        .or_else(|| cleaned.strip_suffix("days"))
    {
        return num_str.trim().parse::<i64>().ok();
    }
    if let Some(num_str) = cleaned
        .strip_suffix('w')
        .or_else(|| cleaned.strip_suffix("weeks"))
    {
        return num_str.trim().parse::<i64>().ok().map(|n| n * 7);
    }
    if let Some(num_str) = cleaned
        .strip_suffix('m')
        .or_else(|| cleaned.strip_suffix("months"))
    {
        return num_str.trim().parse::<i64>().ok().map(|n| n * 30);
    }

    None
}
