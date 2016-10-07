#!/bin/bash

out_dir="$2"
mkdir -p "$out_dir"

list_deps() {
    ldd $1 | \
        sed -e '/=>/s/.*=> //' -e 's/\t*//' -e 's/ \(.*\)//' 
}

copy_deps() {
    for i in $1; do
        if [ -e "$i" ]; then
            dep_name=$(basename "$i")
            if [ ! -e "$2/$dep_name" ]; then
                cp -L "$i" "$2/$dep_name"
                chmod 755 "$2/$dep_name"
            fi
        fi
    done
}

get_deps() {
    copy_deps "$(list_deps $1)" "$2"
}

get_deps "$1" "$2"
