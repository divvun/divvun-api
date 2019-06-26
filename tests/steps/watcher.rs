use std::{fs, thread, time};

use divvun_api::language::speller::SpellerResponse;

use crate::MyWorld;

steps!(MyWorld => {
    given regex r"^I have the `([^`]*)` file available$" (String) |world, file, _step| {
        let mut dir_path = world.config.data_file_dir.clone();
        dir_path.push(&file);
        assert_eq!(dir_path.exists(), true, "{} is not loaded", file);
    };

    when "I add the speller `smj` file" |world, _step| {
        let file_name = "smj.zhfst";
        let spelling_dir = "spelling";

        let mut file_path = world.config.data_file_dir.clone();
        file_path.push(file_name);

        let mut speller_path = world.config.data_file_dir.clone();
        speller_path.push(spelling_dir);
        speller_path.push(file_name);

        fs::copy(file_path, speller_path).unwrap();

        // The watcher watches every second
        thread::sleep(time::Duration::from_secs(2));
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
});
