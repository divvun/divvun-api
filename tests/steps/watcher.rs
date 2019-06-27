use std::{fs, thread, time};

use divvun_api::language::speller::SpellerResponse;
use divvun_api::language::grammar::GramcheckOutput;

use crate::MyWorld;

steps!(MyWorld => {
    given regex r"^I have the `([^`]*)` file available$" (String) |world, file, _step| {
        let mut dir_path = world.config.data_file_dir.clone();
        dir_path.push(&file);
        assert_eq!(dir_path.exists(), true, "{} is not loaded", file);
    };

    when regex r"^I load the `([^`]*)` file into the `([^`]*)` folder$" (String, String) |world, file_name, dir, _step| {
        let mut file_path = world.config.data_file_dir.clone();
        file_path.push(file_name.clone());

        let mut load_path = world.config.data_file_dir.clone();
        load_path.push(dir);
        load_path.push(file_name);

        fs::copy(file_path, load_path).unwrap();

        let watcher_interval = world.config.watcher_interval_ms;
        let sleep_time = watcher_interval + 500;
        thread::sleep(time::Duration::from_millis(sleep_time));
    };

    when regex r"^I remove the `([^`]*)` file from the `([^`]*)` folder$" (String, String) |world, file_name, dir, _step| {
        let mut load_path = world.config.data_file_dir.clone();
        load_path.push(dir);
        load_path.push(file_name);

        fs::remove_file(load_path).unwrap();

        let watcher_interval = world.config.watcher_interval_ms;
        let sleep_time = watcher_interval + 500;
        thread::sleep(time::Duration::from_millis(sleep_time));
    };

    when "I go to the speller endpoint for `smj` with appropriate data" |world, _step| {
        let client = reqwest::Client::new();
        let url = format!("http://{}/speller/smj", &world.config.addr);

        let response: SpellerResponse = client.post(&url).json(&json!({"word": "bådnjåt"})).send().unwrap().json().unwrap();
        world.speller_response = Some(response);
    };

    then "I get back a SpellerResponse with is_correct set to true and some suggestions" |world, _step| {
        let response = &world.speller_response.clone().unwrap();
            assert_eq!(response.word, "bådnjåt");
            assert_eq!(response.is_correct, true);
            assert_eq!(response.suggestions, vec![
                "bådnjåt".to_owned(), "bådnjit".to_owned(), "bådnjut".to_owned(), "bådnjå".to_owned(), "bådnjål".to_owned()
            ]);

        let file_name = "smj.zhfst";
        let spelling_dir = "spelling";

        let mut speller_path = world.config.data_file_dir.clone();
        speller_path.push(spelling_dir);
        speller_path.push(file_name);

        fs::remove_file(speller_path).unwrap();
    };

    when "I go to the grammar endpoint for `smj` with appropriate data" |world, _step| {
        let client = reqwest::Client::new();
        let url = format!("http://{}/grammar/smj", &world.config.addr);

        let response: GramcheckOutput = client.post(&url).json(&json!({"text": "bådnjår"})).send().unwrap().json().unwrap();
        world.grammar_response = Some(response);
    };

    then "I get back a GramcheckOutput detecting a typo" |world, _step| {
        let response = &world.grammar_response.clone().unwrap();
            assert_eq!(response.text, "bådnjår");

            let errs = &response.errs;
            assert_eq!(errs.len(), 1);

            let err0 = &errs[0];
            assert_eq!(err0.error_text, "bådnjår");
            assert_eq!(err0.error_code, "typo");
            assert_eq!(err0.description, "typo");

        let file_name = "smj.zcheck";
        let grammar_dir = "grammar";

        let mut grammar_path = world.config.data_file_dir.clone();
        grammar_path.push(grammar_dir);
        grammar_path.push(file_name);

        fs::remove_file(grammar_path).unwrap();
    };

    then regex r"^I put the removed `([^`]*)` file back into the `([^`]*)` folder$" (String, String) |world, file_name, dir, _step| {
        let mut file_path = world.config.data_file_dir.clone();
        file_path.push(file_name.clone());

        let mut load_path = world.config.data_file_dir.clone();
        load_path.push(dir);
        load_path.push(file_name);

        fs::copy(file_path, load_path).unwrap();
    };
});
