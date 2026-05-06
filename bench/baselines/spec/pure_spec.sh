# shellcheck shell=bash
Describe 'pure.sh'
    BeforeEach 'setup_fixture 5 0 .env'
    AfterEach 'teardown_fixture'

    It 'finds query at PWD and exits 0'
        export HOME="/nonexistent-home-for-test"
        cd "$FIXTURE_PWD"
        When run script "$SHELLSPEC_PROJECT_ROOT/pure.sh" .env
        The status should be success
        The output should equal "$FIXTURE_PLANT"
    End

    It 'exits 1 with no stdout when no match'
        export HOME="/nonexistent-home-for-test"
        cd "$FIXTURE_PWD"
        When run script "$SHELLSPEC_PROJECT_ROOT/pure.sh" nonexistent
        The status should equal 1
        The output should equal ""
        The error should not equal ""
    End
End
