# Meta options
complete -c hexit -s 'v' -l 'version'         -d "Show version of hexit"
complete -c hexit -s '?' -l 'help'            -d "Show list of command-line options"

# Input options
complete -c hexit -s 'e' -l 'expression'      -d "Evaluate this string instead of reading a file"
complete -c hexit -s 'c' -l 'check-syntax'    -d "Check syntax without generating any output"

# Output options
complete -c hexit        -l 'prefix'          -d "String to print before a pair of hex characters" -x
complete -c hexit        -l 'suffix'          -d "String to print after a pair of hex characters" -x
complete -c hexit        -l 'separator'       -d "String to print between successive pairs of hex characters" -x
complete -c hexit -s 'l' -l 'lowercase'       -d "If you like your letters minuscule"
complete -c hexit -s 'r' -l 'raw'             -d "Print bytes without any formatting at all"
complete -c hexit -s 'o' -l 'output'          -d "Write output to the given file, rather than to stdout" -x
complete -c hexit        -l 'limit'           -d "Limit the output from getting too large" -x

# Verification options

complete -c hexit        -l 'verify-length'   -d "Verify that an exact number of bytes is written" -x
complete -c hexit        -l 'verify-multiple' -d "Verify that a multiple of a number of bytes is written" -x
