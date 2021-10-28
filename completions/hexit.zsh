#compdef hexit

__hexit() {
    _arguments \
        "(- 1 *)"{-v,--version}"[Show version of hexit]" \
        "(- 1 *)"{-\?,--help}"[Show list of command-line options]" \
        {-e,--expression}"[Evaluate this string instead of reading a file]:(input):" \
        {-c,--check-syntax}"[Check syntax without generating any output]" \
        --prefix"[String to print before a pair of hex characters]:(string):" \
        --suffix"[String to print after a pair of hex characters]:(string):" \
        --separator"[String to print between successive pairs of hex characters]:(string):" \
        {-l,--lowercase}"[If you like your letters minuscule]" \
        {-r,--raw}"[Print bytes without any formatting at all]" \
        {-o,--output}"[Write output to the given file, rather than stdout]:(path):_files" \
        --limit"[Limit the output from getting too large]:(number)" \
        --verify-length"[Verify that an exact number of bytes is printed]:(number):" \
        --verify-multiple"[Verify that a multiple of a number of bytes is printed]:(number):" \
        '*:filename:_files'
}

__hexit
