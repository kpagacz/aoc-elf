use std::collections::HashMap;

use anyhow::Context;
use chrono::TimeZone;

use crate::domain::ports::{AocClient, GetLeaderboard, InputCache, InputCacheError};
use crate::domain::{DurationString, Submission, SubmissionStatus};
use crate::infrastructure::aoc_api::aoc_client_impl::ResponseStatus;
use crate::infrastructure::aoc_api::AocApi;
use crate::infrastructure::{CliDisplay, FileInputCache};
use crate::infrastructure::{Configuration, HttpDescription};
use crate::submission_history::SubmissionHistory;

#[derive(Debug, Default)]
pub struct Driver {
    configuration: Configuration,
}

impl Driver {
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }

    pub fn input(&self, year: i32, day: i32) -> Result<String, anyhow::Error> {
        let is_already_released = self.is_input_released_yet(year, day, &chrono::Utc::now())?;
        if !is_already_released {
            anyhow::bail!("The input is not released yet");
        }

        match FileInputCache::load(year, day) {
            Ok(input) => return Ok(input),
            Err(e) => match e {
                InputCacheError::Empty(_) => {
                    eprintln!("Attempting to download it from the server...");
                }
                InputCacheError::Load(_) => {
                    eprintln!("Cache corrupted. Clear the cache and try again.");
                }
                _ => {
                    eprintln!("Failed to load the input from the cache: {}", e);
                }
            },
        };

        let http_client = AocApi::prepare_http_client(&self.configuration);
        let aoc_api = AocApi::new(http_client, self.configuration.clone());
        let input = aoc_api.get_input(&year, &day);
        if input.status == ResponseStatus::Ok {
            if FileInputCache::save(&input.body, year, day).is_err() {
                eprintln!("Failed saving the input to the cache");
            }
        } else {
            anyhow::bail!("{}", input.body);
        }
        Ok(input.body)
    }

    pub fn submit_answer(
        &self,
        year: i32,
        day: i32,
        part: crate::domain::RiddlePart,
        answer: String,
    ) -> Result<(), anyhow::Error> {
        let http_client = AocApi::prepare_http_client(&self.configuration);
        let aoc_api = AocApi::new(http_client, self.configuration.clone());

        let mut cache: Option<SubmissionHistory> = match SubmissionHistory::from_cache(&year, &day)
        {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("The application will not have any memory of this submission.");
                None
            }
        };

        let submission = Submission::new(part, answer, year, day);
        if let Some(ref cache) = cache {
            if let Some(submission_result) = cache.correct_submission(&submission.part) {
                eprintln!("🎉  You already submitted the correct answer for this part. Here is the result from last time...\n\n");
                println!("{}", submission_result.message);
                return Ok(());
            }

            if let Some(submission_result) = cache.get_result_for_submission(&submission) {
                eprintln!("♻️  You submitted this answer before and the result was...\n\n");
                println!("{}", submission_result.message);
                if let Some(wait_time) = cache.wait_time(&chrono::Utc::now(), &submission.part) {
                    eprintln!(
                        "\n🌡️  You still need to wait {} before another submission.",
                        DurationString::new(wait_time)
                    );
                }
                return Ok(());
            }

            if let Some(wait_time) = cache.wait_time(&chrono::Utc::now(), &submission.part) {
                eprintln!("🌡️  You wanted to submit an answer too soon. Please wait {} before submitting again.",
                DurationString::new(wait_time));
                return Ok(());
            }
        }

        let submission_result = aoc_api
            .submit_answer(submission)
            .context("Submitting the result was unsuccessful")?;
        eprintln!("Your submission result...\n\n");
        println!("{}", submission_result.message);
        if submission_result.status == SubmissionStatus::Correct
            || submission_result.status == SubmissionStatus::Incorrect
            || submission_result.status == SubmissionStatus::TooSoon
        {
            if let Some(ref mut cache) = cache {
                cache.add(submission_result);
                return Ok(cache.save_to_cache()?);
            } else {
                let mut cache = SubmissionHistory::new(year, day);
                cache.add(submission_result);
                return Ok(cache.save_to_cache()?);
            }
        }

        Ok(())
    }

    pub fn clear_cache(&self) -> Result<(), anyhow::Error> {
        FileInputCache::clear()?;
        SubmissionHistory::clear()?;
        Ok(())
    }

    /// Returns the description of the riddles
    pub fn get_description(&self, year: i32, day: i32) -> Result<String, anyhow::Error> {
        let http_client = AocApi::prepare_http_client(&self.configuration);
        let aoc_api = AocApi::new(http_client, self.configuration.clone());
        Ok(aoc_api
            .get_description::<HttpDescription>(&year, &day)?
            .cli_fmt(&self.configuration))
    }

    fn is_input_released_yet(
        &self,
        year: i32,
        day: i32,
        now: &chrono::DateTime<chrono::Utc>,
    ) -> Result<bool, anyhow::Error> {
        let input_release_time = match chrono::FixedOffset::west_opt(60 * 60 * 5)
            .unwrap()
            .with_ymd_and_hms(year as i32, 12, day as u32, 0, 0, 0)
            .single()
        {
            None => anyhow::bail!("Invalid date"),
            Some(time) => time,
        };

        Ok(now >= &input_release_time)
    }
    /// Lists the directories used by the application
    /// # Example
    /// ```
    /// use elv::Driver;
    /// let driver = Driver::default();
    /// driver.list_app_directories();
    /// ```
    pub fn list_app_directories(&self) -> Result<HashMap<&str, String>, anyhow::Error> {
        let mut directories = HashMap::new();
        if let Some(config_dir) = Configuration::get_project_directories()
            .config_dir()
            .to_str()
        {
            directories.insert("config", config_dir.to_owned());
        }
        if let Some(cache_dir) = Configuration::get_project_directories()
            .cache_dir()
            .to_str()
        {
            directories.insert("cache", cache_dir.to_owned());
        }
        Ok(directories)
    }

    pub fn get_leaderboard(&self, year: u16) -> Result<String, anyhow::Error> {
        let http_client = AocApi::prepare_http_client(&self.configuration);
        let aoc_client = AocApi::new(http_client, self.configuration.clone());
        let leaderboard = aoc_client.get_leaderboard(year)?;

        Ok(leaderboard.cli_fmt(&self.configuration))
    }

    pub fn update_config_value<T>(key: &str, value: T) -> Result<(), anyhow::Error>
    where
        T: Into<config::Value>,
    {
        Configuration::update_configuration_key(key, value)?;
        Ok(())
    }

    pub fn get_config_map() -> Result<config::Map<String, config::Value>, anyhow::Error> {
        Ok(Configuration::get_file_configuration_map()?)
    }

    pub fn set_config_key(key: String, value: String) -> Result<(), anyhow::Error> {
        Configuration::update_configuration_key(&key, value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_input_released_yet() {
        let driver = Driver::default();
        let now = chrono::Utc.with_ymd_and_hms(2022, 12, 1, 5, 0, 0).unwrap();
        for (year, day, expected) in &[
            (2019, 1, true),
            (2020, 1, true),
            (2021, 1, true),
            (2022, 1, true),
            (2022, 2, false),
            (2023, 1, false),
            (2024, 1, false),
        ] {
            assert_eq!(
                driver.is_input_released_yet(*year, *day, &now).unwrap(),
                *expected,
                "Input for {}-{} should be released: {}",
                year,
                day,
                expected
            );
        }
    }

    #[test]
    fn test_invalid_date_to_input() {
        let driver = Driver::default();
        let input = driver.input(0, 0);
        assert!(input.is_err());
        let error = input.err().unwrap();
        assert!(error.to_string() == "Invalid date");
    }
}
