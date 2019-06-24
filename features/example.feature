Feature: divvun-api

  Scenario: Retrieving available languages
    Given I have loaded `se` grammar and speller files
    When I go to the endpoint `/languages`
    Then I get back a JSON object with available languages and their titles