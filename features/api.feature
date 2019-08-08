Feature: divvun-api

  Background:
    Given I have loaded `se` grammar, speller, and hyphenator files

  Scenario: Retrieving available languages
    When I go to the endpoint `/languages`
    Then I get back a JSON object with available languages and their titles

  Scenario: Checking spelling for `se` language
    When I go to the endpoint `/speller/se` with appropriate data
    Then I get back a SpellerResponse with suggestions for each word

  Scenario: Checking grammar for `se` language
    When I go to the endpoint `/grammar/se` with appropriate data
    Then I get back a GramcheckResponse with `typo` and `double-space-before` error codes

  Scenario: Checking spelling for not loaded `en` language
    When I go to the endpoint `/speller/en` for not loaded language
    Then I get back an ApiError with the message `No speller available for language en`

  Scenario: Checking grammar for not loaded `en` language
    When I go to the endpoint `/grammar/en` for not loaded language
    Then I get back an ApiError with the message `No grammar checker available for language en`

  Scenario: Checking hyphenation for `se` language
    When I go to the endpoint `/hyphenation/se` with appropriate data
    Then I get back the correct HyphenationResponse

  Scenario: Checking GraphQL response for `se` language
    When I go to the endpoint `/graphql` with an appropriate GraphQL query
    Then I get back a JSON object with a Speller and Grammar, and Hyphenation response
