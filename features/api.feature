Feature: divvun-api

  Background:
    Given I have loaded `se` grammar and speller files

  Scenario: Retrieving available languages
    When I go to the endpoint `/languages`
    Then I get back a JSON object with available languages and their titles

  Scenario: Checking spelling for `se` language
    When I go to the endpoint `/speller/se` with appropriate data
    Then I get back a SpellerResponse with is_correct set to `false` and some suggestions

  Scenario: Checking grammar for `se` language
    When I go to the endpoint `/grammar/se` with appropriate data
    Then I get back a GrammarOutput with `typo` and `double-space-before` error codes

  Scenario: Checking spelling for not loaded `en` language
    When I go to the endpoint `/speller/en` for not loaded language
    Then I get back an ApiError with the message `No speller available for language en`

  Scenario: Checking grammar for not loaded `en` language
    When I go to the endpoint `/grammar/en` for not loaded language
    Then I get back an ApiError with the message `No grammar checker available for language en`

  Scenario: Checking GraphQL response for `se` language
    When I go to the endpoint `/graphql` with an appropriate GraphQL query
    Then I get back a a JSON object with both a Speller and Grammar response
