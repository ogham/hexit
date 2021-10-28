_hexit()
{
    cur=${COMP_WORDS[COMP_CWORD]}
    prev=${COMP_WORDS[COMP_CWORD-1]}

    case "$prev" in
        -'?'|--help|-v|--version)
            return
            ;;

        -e|--expression|--limit|--prefix|--suffix|--separator|--verify-length|--verify-multiple)
            return
            ;;
    esac

    case "$cur" in
        -*)
            COMPREPLY=( $( compgen -W '$( _parse_help "$1" )' -- "$cur" ) )
            ;;

        *)
            _filedir
            ;;
    esac
} &&
complete -o filenames -o bashdefault -F _hexit hexit
