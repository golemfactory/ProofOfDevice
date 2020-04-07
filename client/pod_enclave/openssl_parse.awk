BEGIN {
    FS = ":"
    first = 1
    dump = ""
}

# parse hex dump
/    / {
    gsub(/ /, "", $0) # strip leading spaces
    for (i=1; i<=NF; i++) {
        if (first == 1) {
            first = 0
            if ($i == "00") # skip leading 0
                continue
        }
        dump = $i dump
    }
}

END {
    printf dump
}
