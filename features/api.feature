Feature: divvun-api

  Background:
    Given I have loaded `se` grammar and speller files

  Scenario: Retrieving available languages
    When I go to the endpoint `/languages`
    Then I get back a JSON object with available languages and their titles

  Scenario: Checking spelling for `se` language
    When I go to the endpoint `/speller/se` with data
    Then I get back a JSON object with is_correct set to `false` and some suggestions