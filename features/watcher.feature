Feature: watcher

  Background:
    Given I have loaded `se` grammar, speller, and hyphenator files

  Scenario: Retrieving speller information for a newly added language
    Given I have the `smj.zhfst` file available
    When I load the `smj.zhfst` file into the `spelling` folder
    And I go to the speller endpoint for `smj` with appropriate data
    Then I get back a SpellerResponse with some suggestions

  Scenario: Retrieving grammar information for a newly added language
    Given I have the `smj.zcheck` file available
    When I load the `smj.zcheck` file into the `grammar` folder
    And I go to the grammar endpoint for `smj` with appropriate data
    Then I get back a GramcheckOutput detecting a typo

  Scenario: Retrieving speller information for a language unloaded at runtime
    Given I have the `se.zhfst` file available
    When I remove the `se.zhfst` file from the `spelling` folder
    And I go to the endpoint `/speller/se` for not loaded language
    Then I get back an ApiError with the message `No speller available for language se`
    And I put the removed `se.zhfst` file back into the `spelling` folder

  Scenario: Retrieving grammar information for a language unloaded at runtime
    Given I have the `se.zcheck` file available
    When I remove the `se.zcheck` file from the `grammar` folder
    And I go to the endpoint `/grammar/se` for not loaded language
    Then I get back an ApiError with the message `No grammar checker available for language se`
    And I put the removed `se.zcheck` file back into the `grammar` folder

  Scenario: Retrieving available languages for an ISO 639-3 language loaded at runtime
    Given I have the `smj.zcheck` file available
    When I load the `smj.zcheck` file into the `grammar` folder
    And I go to the endpoint `/languages`
    Then I get back a JSON object with the `smj` language
