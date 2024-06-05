#!/bin/bash

_master_completion() {
    local cur prev opts
    # all="${COMP_WORDS[*]}"
    # echo "all: ${all}"
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="--help --version -d --datapath -q --querypath --output -c --config -o --output"
    
    case "${prev}" in
        -d|--datapath)
            # Completion for files in MASTER_DATA_DIR
            local data_dir_files
            data_dir_files=$(/bin/ls "${MASTER_DATA_DIR:-}" 2>/dev/null)
            COMPREPLY=($(compgen -W "${data_dir_files}" -- ${cur}))
            return 0
        ;;
        -q|--querypath)
            # Completion for files in MASTER_DATA_DIR
            local data_dir_files
            data_dir_files=$(/bin/ls "${MASTER_QUERY_DIR/*:-}" 2>/dev/null)
            COMPREPLY=($(compgen -W "${data_dir_files}" -- ${cur}))
            return 0
        ;;
        -c|--config)
            # Completion for files in MASTER_DATA_DIR
            local data_dir_files
            data_dir_files=$(/bin/ls "${MASTER_CONFIG_DIR:-}" 2>/dev/null)
            COMPREPLY=($(compgen -W "${data_dir_files}" -- ${cur}))
            return 0
        ;;
        -o|--output)
            # Completion for files in MASTER_DATA_DIR
            local data_dir_files
            data_dir_files=$(/bin/ls "${MASTER_RESULT_DIR:-}" 2>/dev/null)
            COMPREPLY=($(compgen -W "${data_dir_files}" -- ${cur}))
            return 0
        ;;
        *)
        ;;
    esac
    
    COMPREPLY=($(compgen -W "${opts}" -- ${cur}))
    return 0
}

if [ -n "${ZSH_VERSION}" ]; then
    autoload -U +X bashcompinit && bashcompinit
fi

if [ -z "${MASTER_DATA_DIR}" ]; then
    echo "Set MASTER_DATA_DIR"
    return 1
fi
if [ -z "${MASTER_QUERY_DIR}" ]; then
    echo "Set MASTER_QUERY_DIR"
    return 1
fi
if [ -z "${MASTER_CONFIG_DIR}" ]; then
    echo "Set MASTER_CONFIG_DIR"
    return 1
fi
if [ -z "${MASTER_RESULT_DIR}" ]; then
    echo "Set MASTER_RESULT_DIR"
    return 1
fi

complete -F _master_completion dyft
complete -F _master_completion fresh
