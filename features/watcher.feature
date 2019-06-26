Feature: watcher

  Scenario: Retrieving newly added language
    Given I have loaded `se` grammar and speller files
    And I have the `smj.zhfst` file available
    When I add the speller `smj` file
    And I go to the speller endpoint for `smj` with appropriate data
    Then I get back a SpellerResponse with is_correct set to true and some suggestions